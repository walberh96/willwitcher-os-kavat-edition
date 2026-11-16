//! kav with no arguments, in a terminal: the same things as the commands, on one screen.
//!
//! Nothing here changes the system by itself. What a key asks for is done by running kav's
//! own command for it, in the open, with the screen given back to the terminal meanwhile:
//! the person sees the same questions and the same refusals as on the command line.

use crate::allowlist::Allowlist;
use crate::browse::{self, Filter};
use crate::doctor;
use crate::snapshots::{self, Snapshot};
use crate::system::{Paths, as_root, read, size};
use crate::updates::{self, Source, Update};
use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::process::Command;

const TABS: [&str; 7] = ["Dashboard", "Browse", "Updates", "AUR", "Snapshots", "Doctor", "Settings"];

const HELP: &str = "\
1 to 7, Tab      go to a tab
Up, Down, j, k   move in a list
the mouse        a click goes to a tab, a card or a line; the wheel moves in a list
/                Browse: type what to look for; Enter looks
f                Browse: show all, the installed, the explicit, the orphans, the AUR's
Enter            Browse: the full record of the package
i                Browse: install the package
x                Browse: remove the package
u                Dashboard, Updates: bring the whole system up to date
h                Updates: hold a package of the AUR back, or let it go
r                AUR: read the package, and approve it or not
Enter            Snapshots: read the list, which asks for your password;
                 then, what the change of the snapshot did, package by package
s                Snapshots: measure the room the snapshot takes, which takes a moment
c                Snapshots: compare the snapshot with the system now
b                Snapshots: start from the snapshot the next time the machine starts,
                 once; nothing restarts by itself
p                Snapshots: pin the snapshot, so it is not cleaned away, or unpin it
d                Snapshots: delete the snapshot
z                Snapshots: go back to before the last change of packages
f                Doctor: mend what the line under the cursor found
Enter            Settings: change the setting to its next value; the mirror, to the
                 mirrors of a country
F5               read everything again
q, Esc           leave";

/// One line of a list: what is shown, and the package or thing it stands for.
struct Row {
    name: String,
    text: Line<'static>,
}

struct App {
    paths: Paths,
    accent: Color,
    tab: usize,
    cursor: [ListState; 7],
    rows: [Vec<Row>; 7],
    /// what the dashboard shows, read when everything is
    board: Board,
    /// what Browse was asked for, and whether it is being typed
    query: String,
    typing: bool,
    /// the record of a package, or the help, over everything else
    sheet: Option<(String, String)>,
    /// what the last thing done ended in
    said: String,
    /// something that changes the system, waiting for a yes: what it is, in words, and
    /// kav's own command for it
    asked: Option<(String, Vec<String>)>,
    snapshots_read: bool,
    /// which packages Browse shows
    filter: Filter,
    /// what an update would bring, and the snapshots, behind their lines
    pending: Vec<Update>,
    snapshots: Vec<Snapshot>,
    /// the room the snapshots that were measured take, by their numbers
    measured: std::collections::BTreeMap<u32, u64>,
    /// where things were drawn last, for the mouse: each tab's columns, the list, and the
    /// dashboard's cards
    tab_spots: Vec<(u16, u16)>,
    body: Rect,
    cards: [Rect; 4],
}

/// The numbers and the lists of the dashboard.
#[derive(Default)]
struct Board {
    updates: usize,
    from_kavat: usize,
    from_aur: usize,
    waiting: usize,
    waiting_first: String,
    installed: usize,
    explicit: usize,
    foreign: usize,
    /// the newest snapshot, as snapper lists it, once the snapshots were read
    snapshot: Option<(String, String)>,
    health: Vec<Line<'static>>,
    activity: Vec<Line<'static>>,
}

/// The last changes of packages, newest first, from pacman's own log: when, what was done
/// and to what.
fn activity() -> Vec<Line<'static>> {
    let log = std::fs::read_to_string("/var/log/pacman.log").unwrap_or_default();
    let mut changes: Vec<(String, &'static str, Vec<String>)> = Vec::new();
    let mut open = false;
    for line in log.lines() {
        let Some((stamp, rest)) = line.strip_prefix('[').and_then(|line| line.split_once("] ")) else { continue };
        if rest.starts_with("[ALPM] transaction started") {
            changes.push((stamp.replace('T', " ").chars().take(16).collect(), "", Vec::new()));
            open = true;
        } else if rest.starts_with("[ALPM] transaction completed") {
            open = false;
        } else if open {
            let Some(change) = changes.last_mut() else { continue };
            for (word, verb) in [("installed ", "install"), ("upgraded ", "update"), ("removed ", "remove"), ("reinstalled ", "install"), ("downgraded ", "update")] {
                if let Some(what) = rest.strip_prefix("[ALPM] ").and_then(|rest| rest.strip_prefix(word)) {
                    if change.1.is_empty() {
                        change.1 = verb;
                    }
                    change.2.push(what.split_whitespace().next().unwrap_or("").to_string());
                }
            }
        }
    }
    let mut lines = Vec::new();
    for (when, verb, names) in changes.into_iter().rev().filter(|change| !change.2.is_empty()).take(12) {
        let what = match names.len() {
            1 => names[0].clone(),
            count if verb == "update" => format!("{count} packages"),
            count => format!("{} +{} more", names[0], count - 1),
        };
        let colour = match verb {
            "remove" => Color::Red,
            "update" => Color::Cyan,
            _ => Color::Blue,
        };
        lines.push(Line::from(vec![dim(format!("{when}  ")), Span::styled(format!("{verb:<9}"), Style::default().fg(colour)), Span::raw(what)]));
    }
    if lines.is_empty() {
        lines.push(Line::from(dim("No change of packages is on record yet.")));
    }
    lines
}

/// The accent of the system's theme, when the theme has written its colours; yellow otherwise.
fn accent() -> Color {
    let state = std::env::var("XDG_STATE_HOME").ok().filter(|value| !value.is_empty());
    let state = state.or_else(|| std::env::var("HOME").ok().map(|home| format!("{home}/.local/state")));
    let colours = state.and_then(|state| std::fs::read_to_string(format!("{state}/kavat/theme/current/nvim/palette.lua")).ok());
    let hex = colours.as_deref().and_then(|text| {
        let line = text.lines().find(|line| line.trim_start().starts_with("accent "))?;
        let start = line.find('#')?;
        line.get(start + 1..start + 7)
    });
    match hex.and_then(|hex| u32::from_str_radix(hex, 16).ok()) {
        Some(colour) => Color::Rgb((colour >> 16) as u8, (colour >> 8) as u8, colour as u8),
        None => Color::Yellow,
    }
}

fn lines_of(command: &mut Command) -> Vec<String> {
    read(command).unwrap_or_default().lines().map(str::to_string).collect()
}

fn dim(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().add_modifier(Modifier::DIM))
}

impl App {
    fn new(paths: Paths) -> App {
        let mut app = App {
            paths,
            accent: accent(),
            tab: 0,
            cursor: Default::default(),
            rows: Default::default(),
            query: String::new(),
            typing: false,
            sheet: None,
            said: String::new(),
            asked: None,
            snapshots_read: false,
            filter: Filter::All,
            pending: Vec::new(),
            snapshots: Vec::new(),
            measured: Default::default(),
            tab_spots: Vec::new(),
            body: Rect::default(),
            cards: [Rect::default(); 4],
            board: Board::default(),
        };
        app.read_all();
        app
    }

    fn bold(&self, text: impl Into<String>) -> Span<'static> {
        Span::styled(text.into(), Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
    }

    fn read_all(&mut self) {
        self.read_updates();
        self.read_aur();
        self.read_doctor();
        if self.snapshots_read {
            self.read_snapshots();
        } else {
            self.rows[4] = vec![Row { name: String::new(), text: Line::from("Enter reads the snapshots; it asks for your password.") }];
        }
        self.search();
        self.read_dashboard();
        self.read_settings();
        for tab in 0..TABS.len() {
            let last = self.rows[tab].len().saturating_sub(1);
            let at = self.cursor[tab].selected().unwrap_or(0).min(last);
            self.cursor[tab].select(Some(at));
        }
    }

    fn read_settings(&mut self) {
        use crate::settings::{LOCKED, SETTINGS, get, mirror, next, shown};
        let wide = |text: &str| Span::raw(format!("{text:<68}"));
        let fixed = |what: &str, value: String| Row { name: String::new(), text: Line::from(vec![wide(what), dim(value)]) };
        let empty = || Row { name: String::new(), text: Line::from("") };
        let mut rows: Vec<Row> = SETTINGS
            .iter()
            .map(|setting| {
                let now = get(&self.paths, setting.name);
                Row {
                    name: format!("{} {}", setting.name, next(setting, now.as_deref())),
                    text: Line::from(vec![wide(setting.what), now.map_or(dim("root's to read; Enter changes it"), |value| self.bold(shown(setting, &value)))]),
                }
            })
            .collect();
        rows.push(Row { name: "mirror".into(), text: Line::from(vec![wide("The mirror in use"), self.bold(mirror().unwrap_or_else(|| "none is set".into()))]) });
        rows.push(empty());
        rows.extend(LOCKED.iter().map(|(what, value)| fixed(what, format!("{value} · locked"))));
        rows.push(empty());
        rows.extend([
            fixed("The reviewed list of the system", self.paths.lists[0].display().to_string()),
            fixed("The reviewed list of this machine", self.paths.lists[1].display().to_string()),
            fixed("kav's own settings", self.paths.settings().display().to_string()),
            fixed("What was asked to be reviewed", self.paths.requests.display().to_string()),
            fixed("Where AUR packages are built", self.paths.chroot.display().to_string()),
        ]);
        rows.push(empty());
        rows.push(Row { name: String::new(), text: Line::from(dim("Enter on one of the first five changes it to its next value; on the mirror, it asks for a country. What is locked protects the system.")) });
        self.rows[6] = rows;
    }

    fn read_updates(&mut self) {
        // pacman's lists as they were last fetched, by an update or by the look for updates
        self.pending = updates::pending(&self.paths);
        let mut rows = Vec::new();
        for source in [Source::Arch, Source::Kavat, Source::Aur] {
            let these: Vec<&Update> = self.pending.iter().filter(|update| update.source == source).collect();
            if these.is_empty() {
                continue;
            }
            rows.push(Row { name: String::new(), text: Line::from(self.bold(format!("{} · {}", source.title(), these.len()))) });
            for update in these {
                let sizes = match source {
                    Source::Aur => format!("{:>24}", "built here"),
                    _ => format!("{:>11} {:>12}", size(update.download), updates::signed(update.change)),
                };
                let mut text = vec![Span::raw(format!("  {:<30}", update.name)), dim(format!("{:<18} -> ", update.from)), Span::raw(format!("{:<18}", update.to)), dim(sizes)];
                if update.restart {
                    text.push(Span::styled("  restart", Style::default().fg(Color::Yellow)));
                }
                if update.held {
                    text.push(Span::styled("  held back", Style::default().fg(Color::Yellow)));
                }
                rows.push(Row { name: update.name.clone(), text: Line::from(text) });
            }
        }
        if rows.is_empty() {
            rows.push(Row { name: String::new(), text: Line::from("Nothing waits, by the lists pacman has. kav update fetches them again.") });
        }
        self.rows[2] = rows;
    }

    fn read_aur(&mut self) {
        let mut rows = Vec::new();
        let foreign = lines_of(Command::new("pacman").arg("-Qmq"));
        match Allowlist::load(&self.paths.lists) {
            Ok(list) => {
                for entry in list.entries() {
                    let installed = if foreign.contains(&entry.name) { "installed" } else { "" };
                    rows.push(Row {
                        name: entry.name.clone(),
                        text: Line::from(vec![
                            Span::raw(format!("{:<34}", entry.name)),
                            dim(format!("{}  reviewed {}  ", &entry.commit[..12.min(entry.commit.len())], entry.reviewed)),
                            Span::raw(installed),
                        ]),
                    });
                }
            }
            Err(problem) => rows.push(Row { name: String::new(), text: Line::from(problem) }),
        }
        for name in std::fs::read_to_string(&self.paths.requests).unwrap_or_default().lines() {
            if !rows.iter().any(|row| row.name == name) {
                rows.push(Row { name: name.to_string(), text: Line::from(vec![Span::raw(format!("{name:<34}")), dim("asked for, not reviewed yet")]) });
            }
        }
        if rows.is_empty() {
            rows.push(Row { name: String::new(), text: Line::from("The reviewed list is empty, and nothing is asked for.") });
        }
        self.rows[3] = rows;
    }

    fn read_doctor(&mut self) {
        let mut rows = Vec::new();
        for (index, (what, problems)) in doctor::findings().into_iter().enumerate() {
            let mark = if problems.is_empty() {
                Span::styled("ok    ", Style::default().fg(Color::Green))
            } else {
                Span::styled("FAIL  ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            };
            // a line that can be mended carries kav's own words for it: f runs kav fix with them
            let fix = doctor::FIXES.get(index).copied().flatten().filter(|_| !problems.is_empty());
            let whole = fix.filter(|fix| !matches!(*fix, "service" | "merge")).unwrap_or_default();
            let hint = if whole.is_empty() { Span::raw("") } else { dim("   f mends it") };
            rows.push(Row { name: whole.to_string(), text: Line::from(vec![mark, Span::raw(what), hint]) });
            for problem in problems {
                let one = match fix {
                    Some(fix @ ("service" | "merge")) => format!("{fix} {problem}"),
                    _ => String::new(),
                };
                let hint = if one.is_empty() { Span::raw("") } else { dim("   f mends it") };
                rows.push(Row { name: one, text: Line::from(vec![Span::raw("      "), dim(problem), hint]) });
            }
        }
        self.rows[5] = rows;
    }

    fn read_snapshots(&mut self) {
        // the newest first
        self.snapshots = snapshots::list().unwrap_or_default();
        self.snapshots.reverse();
        for snapshot in &mut self.snapshots {
            snapshot.takes = self.measured.get(&snapshot.number).copied();
        }
        let mut rows: Vec<Row> = self.snapshots.iter().map(|snapshot| Row { name: snapshot.number.to_string(), text: Line::from(snapshots::line(snapshot)) }).collect();
        if rows.is_empty() {
            rows.push(Row { name: String::new(), text: Line::from("There are no snapshots to list.") });
        }
        self.rows[4] = rows;
    }

    fn read_dashboard(&mut self) {
        let moving: Vec<&Update> = self.pending.iter().filter(|update| !update.held).collect();
        let asked: Vec<&Row> = self.rows[3].iter().filter(|row| !row.name.is_empty() && row.text.to_string().contains("asked for")).collect();
        let mut health = Vec::new();
        for (what, problems) in doctor::findings() {
            health.push(match problems.first() {
                None => Line::from(vec![Span::styled("✓ ", Style::default().fg(Color::Green)), Span::raw(what)]),
                Some(first) => {
                    let more = if problems.len() > 1 { format!("  +{} more", problems.len() - 1) } else { String::new() };
                    Line::from(vec![Span::styled("! ", Style::default().fg(Color::Yellow)), Span::styled(format!("{first}{more}"), Style::default().fg(Color::Yellow))])
                }
            });
        }
        self.board = Board {
            updates: moving.len(),
            from_kavat: moving.iter().filter(|update| update.source == Source::Kavat).count(),
            from_aur: moving.iter().filter(|update| update.source == Source::Aur).count(),
            waiting: asked.len(),
            waiting_first: asked.first().map(|row| row.name.clone()).unwrap_or_default(),
            installed: lines_of(Command::new("pacman").arg("-Qq")).len(),
            explicit: lines_of(Command::new("pacman").arg("-Qeq")).len(),
            foreign: lines_of(Command::new("pacman").arg("-Qmq")).len(),
            snapshot: self.snapshots.first().map(|snapshot| (snapshot.number.to_string(), snapshot.date.chars().take(16).collect())),
            health,
            activity: activity(),
        };
        self.rows[0].clear();
    }

    /// The dashboard, as the design draws it: four numbers, the health, what was done last
    /// and the news.
    fn draw_dashboard(&mut self, frame: &mut Frame, area: Rect) {
        let [cards, middle, news] = Layout::vertical([Constraint::Length(5), Constraint::Min(3), Constraint::Length(3)]).areas(area);
        let cells: [Rect; 4] = Layout::horizontal([Constraint::Ratio(1, 4); 4]).areas(cards);
        self.cards = cells;
        let board = &self.board;
        let card = |title: &str, number: String, about: String, key: &str| {
            Paragraph::new(vec![
                Line::from(Span::styled(number, Style::default().add_modifier(Modifier::BOLD))),
                Line::from(about),
                Line::from(dim(key.to_string())),
            ])
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().add_modifier(Modifier::DIM)).title(self.bold(format!(" {title} "))))
        };
        frame.render_widget(card("Updates", board.updates.to_string(),
            format!("{} Arch · {} [kavat] · {} AUR", board.updates - board.from_kavat - board.from_aur, board.from_kavat, board.from_aur), "3 see them · u update all"), cells[0]);
        frame.render_widget(card("AUR reviews", board.waiting.to_string(),
            if board.waiting == 0 { "nothing is asked for".into() } else { format!("{} asked for", board.waiting_first) }, "4 open review"), cells[1]);
        let (number, when) = board.snapshot.clone().unwrap_or_else(|| ("–".into(), "not read yet: it asks for your password".into()));
        frame.render_widget(card("Last snapshot", if number == "–" { number } else { format!("#{number}") }, when, "5 open timeline"), cells[2]);
        frame.render_widget(card("Installed", board.installed.to_string(),
            format!("{} explicit · {} from AUR", board.explicit, board.foreign), "2 browse"), cells[3]);

        let [left, right] = Layout::horizontal([Constraint::Ratio(1, 2); 2]).areas(middle);
        let pane = |title: &str| Block::default().borders(Borders::ALL).border_style(Style::default().add_modifier(Modifier::DIM)).title(self.bold(format!(" {title} ")));
        frame.render_widget(Paragraph::new(board.health.clone()).wrap(Wrap { trim: false }).block(pane("Health")), left);
        frame.render_widget(Paragraph::new(board.activity.clone()).block(pane("Recent activity")), right);
        frame.render_widget(
            Paragraph::new(Line::from(dim("Arch's news is not read yet: kav update will show what needs a hand."))).block(pane("Arch news")),
            news,
        );
    }

    /// Look for what was typed, in pacman's repositories and on the reviewed list, and keep
    /// what the filter keeps. With nothing typed, a filter shows all that is its own.
    fn search(&mut self) {
        let words: Vec<&str> = self.query.split_whitespace().collect();
        let among = self.filter.packages();
        let list = Allowlist::load(&self.paths.lists).unwrap_or_default();
        let mut rows = Vec::new();
        if words.is_empty() {
            for (name, version) in &among {
                let place = if self.filter == Filter::Aur && list.get(name).is_none() { "not on the reviewed list" } else { "" };
                rows.push(Row { name: name.clone(), text: Line::from(vec![Span::raw(format!("{name:<34}")), dim(format!("{version:<31}")), Span::raw("installed  "), dim(place)]) });
            }
        } else {
            let found = read(Command::new("pacman").arg("-Ss").args(&words)).unwrap_or_default();
            for line in found.lines().filter(|line| !line.starts_with(' ')) {
                let mut parts = line.split_whitespace();
                let (Some(place), Some(version)) = (parts.next(), parts.next()) else { continue };
                let Some((repository, name)) = place.split_once('/') else { continue };
                if !browse::keep(self.filter, name, &among) {
                    continue;
                }
                let installed = if line.contains("[installed") { "installed" } else { "" };
                rows.push(Row {
                    name: name.to_string(),
                    text: Line::from(vec![Span::raw(format!("{name:<34}")), dim(format!("{repository:<8} {version:<22}")), Span::raw(installed)]),
                });
            }
            // what is installed and in no list pacman has: one from the AUR, or any of them
            // on a system that has not fetched its lists yet
            let local = read(Command::new("pacman").arg("-Qs").args(&words)).unwrap_or_default();
            for line in local.lines().filter(|line| !line.starts_with(' ')) {
                let mut parts = line.split_whitespace();
                let (Some(place), Some(version)) = (parts.next(), parts.next()) else { continue };
                let name = place.rsplit('/').next().unwrap_or(place);
                if rows.iter().any(|row| row.name == name) || !browse::keep(self.filter, name, &among) {
                    continue;
                }
                rows.push(Row {
                    name: name.to_string(),
                    text: Line::from(vec![Span::raw(format!("{name:<34}")), dim(format!("{:<8} {version:<22}", "")), Span::raw("installed")]),
                });
            }
        }
        // the reviewed list is the AUR's: it shows with no filter, and under the AUR's
        if matches!(self.filter, Filter::All | Filter::Aur) && (!words.is_empty() || self.filter == Filter::Aur) {
            let shown: Vec<String> = rows.iter().map(|row| row.name.clone()).collect();
            for entry in list.entries().filter(|entry| words.iter().all(|word| entry.name.contains(word)) && !shown.contains(&entry.name)) {
                rows.push(Row { name: entry.name.clone(), text: Line::from(vec![Span::raw(format!("{:<34}", entry.name)), dim("the AUR, on the reviewed list")]) });
            }
        }
        if rows.is_empty() {
            rows.push(Row {
                name: String::new(),
                text: Line::from(match (words.is_empty(), self.filter) {
                    (true, Filter::All) => "/ and what to look for, then Enter. f shows the installed, the explicit, the orphans or the AUR's.".to_string(),
                    (true, _) => "No package is like that.".to_string(),
                    (false, Filter::All) => format!("Nothing is called {}.", self.query),
                    (false, filter) => format!("Nothing called {} is among: {}.", self.query, filter.title()),
                }),
            });
        }
        self.rows[1] = rows;
        self.cursor[1].select(Some(0));
    }

    fn selected(&self) -> Option<&str> {
        let row = self.rows[self.tab].get(self.cursor[self.tab].selected()?)?;
        (!row.name.is_empty()).then_some(row.name.as_str())
    }

    fn move_by(&mut self, by: isize) {
        let count = self.rows[self.tab].len();
        if count == 0 {
            return;
        }
        let at = self.cursor[self.tab].selected().unwrap_or(0) as isize;
        self.cursor[self.tab].select(Some((at + by).clamp(0, count as isize - 1) as usize));
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [top, body, bottom] = Layout::vertical([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

        // the tabs, in one line: the one in view is filled with the accent
        let mut header = vec![self.bold(" kav "), Span::raw(" ")];
        let mut column = 6;
        self.tab_spots.clear();
        for (index, name) in TABS.iter().enumerate() {
            let label = format!(" {} {name} ", index + 1);
            self.tab_spots.push((column, column + label.len() as u16));
            column += label.len() as u16 + 1;
            header.push(if index == self.tab {
                Span::styled(label, Style::default().bg(self.accent).fg(Color::Black).add_modifier(Modifier::BOLD))
            } else {
                dim(label)
            });
            header.push(Span::raw(" "));
        }
        let who = format!("{}@{} ", std::env::var("USER").unwrap_or_default(), std::fs::read_to_string("/etc/hostname").unwrap_or_default().trim());
        let [tabs, right] = Layout::horizontal([Constraint::Min(1), Constraint::Length(who.len() as u16)]).areas(top);
        frame.render_widget(Paragraph::new(Line::from(header)), tabs);
        frame.render_widget(Paragraph::new(Line::from(dim(who))), right);

        self.body = body;
        if self.tab == 0 {
            self.draw_dashboard(frame, body);
        } else {
            let showing = if self.filter == Filter::All { String::new() } else { format!(" · showing: {}", self.filter.title()) };
            let title = match self.tab {
                1 if self.typing => format!(" Browse{showing} · looking for: {}▏ ", self.query),
                1 if !self.query.is_empty() => format!(" Browse{showing} · {} ", self.query),
                1 => format!(" Browse{showing} "),
                2 => format!(" Updates · {} ", updates::total(&self.pending)),
                tab => format!(" {} ", TABS[tab]),
            };
            let items: Vec<ListItem> = self.rows[self.tab].iter().map(|row| ListItem::new(row.text.clone())).collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().add_modifier(Modifier::DIM)).title(self.bold(title)))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            frame.render_stateful_widget(list, body, &mut self.cursor[self.tab]);
        }

        // the keys that do something here, each in its own box
        let mut keys: Vec<(&str, &str)> = vec![("1–7", "tabs"), ("/", "search")];
        keys.extend(match self.tab {
            1 => vec![("f", "filter"), ("↵", "record"), ("i", "install"), ("x", "remove")],
            2 => vec![("u", "update all"), ("h", "hold an AUR package back")],
            3 => vec![("r", "review")],
            4 if self.snapshots_read => vec![("↵", "its packages"), ("s", "size"), ("c", "compare"), ("b", "boot it next"), ("p", "pin"), ("d", "delete"), ("z", "undo the last change")],
            4 => vec![("↵", "read the list"), ("z", "undo the last change")],
            5 => vec![("f", "mend the line under the cursor")],
            6 => vec![("↵", "change to the next value")],
            0 => vec![("u", "update all")],
            _ => vec![],
        });
        keys.extend([("?", "help"), ("q", "quit")]);
        let mut bar = vec![Span::raw(" ")];
        if self.said.is_empty() {
            for (key, what) in keys {
                bar.push(Span::styled(format!(" {key} "), Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)));
                bar.push(dim(format!(" {what}   ")));
            }
        } else {
            bar.push(Span::raw(self.said.clone()));
        }
        frame.render_widget(Paragraph::new(Line::from(bar)), bottom);

        if let Some((what, arguments)) = &self.asked {
            let area = centred(frame.area(), 60, 40);
            frame.render_widget(Clear, area);
            let text = vec![
                Line::from(""),
                Line::from(Span::styled(format!("  {what}"), Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(format!("  It runs  kav {}  in this terminal.", arguments.join(" "))),
                Line::from("  It changes the system, so it asks for your password there:"),
                Line::from("  type it at the line that says so, and it goes on."),
                Line::from(""),
                Line::from(vec![Span::raw("  "), self.bold("Enter"), Span::raw("  go on      "), self.bold("Esc"), Span::raw("  leave it")]),
            ];
            let question = Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(self.accent)).title(self.bold(" Before it starts ")));
            frame.render_widget(question, area);
        }

        if let Some((title, text)) = &self.sheet {
            let area = centred(frame.area(), 80, 80);
            frame.render_widget(Clear, area);
            let sheet = Paragraph::new(text.clone())
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title(format!(" {title} · any key closes ")));
            frame.render_widget(sheet, area);
        }
    }

    /// Give the screen back to the terminal, run one of kav's own commands in the open, and
    /// take the screen again once the person has read what it said.
    fn hand_over(&mut self, terminal: &mut DefaultTerminal, arguments: &[&str]) {
        screen_off();
        println!("\n  kav {}\n", arguments.join(" "));
        println!("  If it asks for your password, type it and press Enter: nothing shows while you type.\n");
        let me = std::env::current_exe().unwrap_or_else(|_| "kav".into());
        let ended = Command::new(me)
            .args(arguments)
            .env("SUDO_PROMPT", "  Your password, %u, to change the system's packages: ")
            .status();
        self.said = match ended {
            Ok(status) if status.success() => format!("kav {} is done", arguments.join(" ")),
            _ => format!("kav {} did not finish", arguments.join(" ")),
        };
        // what changed the system may have changed what each snapshot alone holds
        if arguments.get(1) != Some(&"compare") {
            self.measured.clear();
        }
        println!("\nEnter goes back to kav.");
        let _ = std::io::stdin().read_line(&mut String::new());
        *terminal = screen_on();
        self.read_all();
    }

    /// The mouse: a click goes to a tab, to a card of the dashboard or to a line of a list,
    /// and the wheel moves in the list. It does nothing a key would have to confirm.
    fn mouse(&mut self, mouse: MouseEvent) {
        if self.asked.is_some() {
            return;
        }
        let inside = |area: Rect| mouse.column >= area.x && mouse.column < area.x + area.width && mouse.row >= area.y && mouse.row < area.y + area.height;
        match mouse.kind {
            MouseEventKind::ScrollDown if self.sheet.is_none() => self.move_by(3),
            MouseEventKind::ScrollUp if self.sheet.is_none() => self.move_by(-3),
            MouseEventKind::Down(MouseButton::Left) => {
                self.said.clear();
                self.typing = false;
                if self.sheet.take().is_some() {
                    return;
                }
                if mouse.row == 0 {
                    if let Some(tab) = self.tab_spots.iter().position(|(from, to)| (*from..*to).contains(&mouse.column)) {
                        self.tab = tab;
                    }
                } else if self.tab == 0 {
                    // each card opens the tab it counts
                    if let Some(card) = self.cards.iter().position(|card| inside(*card)) {
                        self.tab = [2, 3, 4, 1][card];
                    }
                } else if inside(self.body) && mouse.row > self.body.y {
                    let line = self.cursor[self.tab].offset() + (mouse.row - self.body.y - 1) as usize;
                    if line < self.rows[self.tab].len() {
                        self.cursor[self.tab].select(Some(line));
                    }
                }
            }
            _ => {}
        }
    }

    /// One key. False when it is time to leave.
    fn key(&mut self, terminal: &mut DefaultTerminal, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.said.clear();
        if self.sheet.take().is_some() {
            return true;
        }
        if let Some((_, arguments)) = self.asked.take() {
            if code == KeyCode::Enter {
                let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
                self.hand_over(terminal, &arguments);
            }
            return true;
        }
        if self.typing {
            match code {
                KeyCode::Enter => {
                    self.typing = false;
                    if !self.query.trim().is_empty() {
                        self.search();
                    }
                }
                KeyCode::Esc => self.typing = false,
                KeyCode::Backspace => {
                    self.query.pop();
                }
                KeyCode::Char(letter) => self.query.push(letter),
                _ => {}
            }
            return true;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return false,
            KeyCode::Char(digit @ '1'..='7') => self.tab = digit as usize - '1' as usize,
            KeyCode::Tab | KeyCode::Right => self.tab = (self.tab + 1) % TABS.len(),
            KeyCode::BackTab | KeyCode::Left => self.tab = (self.tab + TABS.len() - 1) % TABS.len(),
            KeyCode::Down | KeyCode::Char('j') => self.move_by(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_by(-1),
            KeyCode::PageDown => self.move_by(15),
            KeyCode::PageUp => self.move_by(-15),
            KeyCode::Home => self.move_by(-(self.rows[self.tab].len() as isize)),
            KeyCode::End => self.move_by(self.rows[self.tab].len() as isize),
            KeyCode::Char('?') => self.sheet = Some(("Help".into(), HELP.into())),
            KeyCode::F(5) => self.read_all(),
            KeyCode::Char('/') => {
                self.tab = 1;
                self.typing = true;
                self.query.clear();
            }
            KeyCode::Char('f') if self.tab == 1 => {
                self.filter = self.filter.next();
                self.search();
            }
            KeyCode::Enter if self.tab == 1 => {
                if let Some(name) = self.selected().map(str::to_string) {
                    let record = read(Command::new("pacman").args(["-Qi", &name]))
                        .or_else(|| read(Command::new("pacman").args(["-Si", &name])))
                        .unwrap_or_else(|| "It is not in pacman's repositories: kav info says where it would come from.".into());
                    self.sheet = Some((name, record));
                }
            }
            KeyCode::Char('i') if self.tab == 1 => {
                if let Some(name) = self.selected().map(str::to_string) {
                    self.asked = Some((format!("Install {name}"), vec!["install".into(), name]));
                }
            }
            KeyCode::Char('x') if self.tab == 1 => {
                if let Some(name) = self.selected().map(str::to_string) {
                    self.asked = Some((format!("Remove {name}"), vec!["remove".into(), name]));
                }
            }
            KeyCode::Char('u') if self.tab == 2 || self.tab == 0 => {
                self.asked = Some(("Bring the whole system up to date".into(), vec!["update".into()]));
            }
            KeyCode::Char('h') if self.tab == 2 => {
                let update = self.selected().and_then(|name| self.pending.iter().find(|update| update.name == name));
                match update.map(|update| (update.source, update.held, update.name.clone())) {
                    Some((Source::Aur, true, name)) => self.asked = Some((format!("Let {name} go: kav update builds its reviewed commit"), vec!["unhold".into(), name])),
                    Some((Source::Aur, false, name)) => self.asked = Some((format!("Hold {name} back: kav update leaves it as it is"), vec!["hold".into(), name])),
                    Some(_) => self.said = "Only a package of the AUR can be held back: Arch's are brought up to date all together.".into(),
                    None => {}
                }
            }
            KeyCode::Char('r') if self.tab == 3 => {
                if let Some(name) = self.selected().map(str::to_string) {
                    self.hand_over(terminal, &["review", &name]);
                }
            }
            KeyCode::Char('f') if self.tab == 5 => {
                if let Some(words) = self.selected().map(str::to_string) {
                    let mut arguments = vec!["fix".to_string()];
                    arguments.extend(words.split_whitespace().map(str::to_string));
                    self.asked = Some((format!("Mend: {words}"), arguments));
                }
            }
            KeyCode::Enter if self.tab == 6 && self.selected() == Some("mirror") => {
                self.asked = Some(("The mirrors of a country, or the worldwide ones: it asks which in the terminal".into(), vec!["set".into(), "mirror".into()]));
            }
            KeyCode::Enter if self.tab == 6 => {
                // the row's name is the setting and the value it would take
                if let Some((name, value)) = self.selected().and_then(|row| row.split_once(' ')) {
                    let what = crate::settings::SETTINGS.iter().find(|s| s.name == name).map_or(name, |s| s.what);
                    self.asked = Some((format!("{what}: {value}"), vec!["set".into(), name.into(), value.into()]));
                }
            }
            KeyCode::Enter if self.tab == 4 && self.snapshots_read => {
                let chosen = self.selected().and_then(|number| self.snapshots.iter().find(|snapshot| snapshot.number.to_string() == number));
                if let Some(snapshot) = chosen {
                    let mut text = format!("{}, of {}\n{}\n\n", snapshot.kind, snapshot.date, snapshot.description);
                    text.push_str(&match snapshot.packages.is_empty() {
                        true => "pacman's log has no change of packages for it.".to_string(),
                        false => snapshot.packages.join("\n"),
                    });
                    self.sheet = Some((format!("Snapshot {}{}", snapshot.number, if snapshot.pinned() { " · pinned" } else { "" }), text));
                }
            }
            KeyCode::Char('s') if self.tab == 4 && self.snapshots_read => {
                if let Some(number) = self.selected().and_then(|number| number.parse().ok()) {
                    // sudo may ask for the password again: it needs the terminal as it is
                    screen_off();
                    println!("\n  Measuring snapshot {number}: btrfs reads it whole, which takes a moment.");
                    let measured = snapshots::sizes(&[number]);
                    *terminal = screen_on();
                    match measured.as_ref().and_then(|measured| measured.get(&number)) {
                        Some(bytes) => {
                            self.measured.insert(number, *bytes);
                            self.said = format!("Snapshot {number} alone takes {}: what deleting it gives back", size(*bytes));
                            self.read_snapshots();
                        }
                        None => self.said = format!("Snapshot {number} could not be measured"),
                    }
                }
            }
            KeyCode::Char('c') if self.tab == 4 => {
                if let Some(number) = self.selected().map(str::to_string) {
                    self.hand_over(terminal, &["snapshots", "compare", &number]);
                }
            }
            KeyCode::Char('b') if self.tab == 4 => {
                if let Some(number) = self.selected().map(str::to_string) {
                    self.asked = Some((format!("Start from snapshot {number} the next time, once: nothing restarts by itself"), vec!["snapshots".into(), "boot".into(), number]));
                }
            }
            KeyCode::Char('d') if self.tab == 4 => {
                if let Some(number) = self.selected().map(str::to_string) {
                    self.asked = Some((format!("Delete snapshot {number}: the system cannot go back to it after"), vec!["snapshots".into(), "delete".into(), number, "--yes".into()]));
                }
            }
            KeyCode::Char('p') if self.tab == 4 => {
                let chosen = self.selected().and_then(|number| self.snapshots.iter().find(|snapshot| snapshot.number.to_string() == number));
                if let Some((number, pinned)) = chosen.map(|snapshot| (snapshot.number.to_string(), snapshot.pinned())) {
                    self.asked = Some(match pinned {
                        true => (format!("Unpin snapshot {number}: it goes when there are too many"), vec!["snapshots".into(), "unpin".into(), number]),
                        false => (format!("Pin snapshot {number}: it stays until it is deleted by hand"), vec!["snapshots".into(), "pin".into(), number]),
                    });
                }
            }
            KeyCode::Enter if self.tab == 4 => {
                // sudo may ask for the password: it needs the terminal as it is
                screen_off();
                let allowed = as_root("true").status().is_ok_and(|status| status.success());
                *terminal = screen_on();
                if allowed {
                    self.snapshots_read = true;
                    self.read_snapshots();
                    self.cursor[4].select(Some(0));
                } else {
                    self.said = "The snapshots are root's to read, and the password was not given.".into();
                }
            }
            KeyCode::Char('z') if self.tab == 4 => {
                self.asked = Some(("Go back to before the last change of packages".into(), vec!["undo".into()]));
            }
            _ => {}
        }
        true
    }
}

/// A rectangle in the middle of another, as a percentage of each side.
fn centred(area: Rect, width: u16, height: u16) -> Rect {
    let [_, row, _] = Layout::vertical([Constraint::Percentage((100 - height) / 2), Constraint::Percentage(height), Constraint::Min(0)]).areas(area);
    let [_, cell, _] = Layout::horizontal([Constraint::Percentage((100 - width) / 2), Constraint::Percentage(width), Constraint::Min(0)]).areas(row);
    cell
}

/// Takes the terminal for the full screen, with the mouse.
fn screen_on() -> DefaultTerminal {
    let terminal = ratatui::init();
    let _ = ratatui::crossterm::execute!(std::io::stdout(), EnableMouseCapture);
    terminal
}

/// Gives the terminal back as it was.
fn screen_off() {
    let _ = ratatui::crossterm::execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
}

pub fn run(paths: Paths) -> Result<(), String> {
    let mut app = App::new(paths);
    let mut terminal = screen_on();
    let ended = loop {
        if let Err(error) = terminal.draw(|frame| app.draw(frame)) {
            break Err(error.to_string());
        }
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                if !app.key(&mut terminal, key.code, key.modifiers) {
                    break Ok(());
                }
            }
            Ok(Event::Mouse(mouse)) => app.mouse(mouse),
            Ok(_) => {}
            Err(error) => break Err(error.to_string()),
        }
    };
    screen_off();
    ended
}
