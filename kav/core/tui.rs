//! kav with no arguments, in a terminal: the same things as the commands, on one screen.
//!
//! Nothing here changes the system by itself. What a key asks for is done by running kav's
//! own command for it, in the open, with the screen given back to the terminal meanwhile:
//! the person sees the same questions and the same refusals as on the command line.

use crate::allowlist::Allowlist;
use crate::doctor;
use crate::system::{Paths, as_root, read};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
/                Browse: type what to look for; Enter looks
Enter            Browse: the full record of the package
i                Browse: install the package
x                Browse: remove the package
u                Dashboard, Updates: bring the whole system up to date
7                Settings: where kav keeps its lists
r                AUR: read the package, and approve it or not
Enter            Snapshots: read the list, which asks for your password
z                Snapshots: go back to before the last change of packages
f                Doctor: mend what the line under the cursor found
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
}

/// The numbers and the lists of the dashboard.
#[derive(Default)]
struct Board {
    updates: usize,
    from_kavat: usize,
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
        if !self.query.is_empty() {
            self.search();
        } else {
            self.rows[1] = vec![Row { name: String::new(), text: Line::from("/ and what to look for, then Enter.") }];
        }
        self.read_dashboard();
        self.rows[6] = crate::settings::SETTINGS
            .iter()
            .map(|setting| {
                let now = crate::settings::get(setting.name);
                Row {
                    name: format!("{} {}", setting.name, crate::settings::next(setting, now)),
                    text: Line::from(vec![
                        Span::raw(format!("{:<34}", setting.what)),
                        now.map_or(dim("root's to read; Enter changes it"), |value| self.bold(value.to_string())),
                    ]),
                }
            })
            .collect();
        self.rows[6].extend([
            Row { name: String::new(), text: Line::from("") },
            Row { name: String::new(), text: Line::from(vec![Span::raw(format!("{:<34}", "The reviewed list of the system")), dim(self.paths.lists[0].display().to_string())]) },
            Row { name: String::new(), text: Line::from(vec![Span::raw(format!("{:<34}", "The reviewed list of this machine")), dim(self.paths.lists[1].display().to_string())]) },
            Row { name: String::new(), text: Line::from(vec![Span::raw(format!("{:<34}", "What was asked to be reviewed")), dim(self.paths.requests.display().to_string())]) },
            Row { name: String::new(), text: Line::from(vec![Span::raw(format!("{:<34}", "Where AUR packages are built")), dim(self.paths.chroot.display().to_string())]) },
            Row { name: String::new(), text: Line::from("") },
            Row { name: String::new(), text: Line::from(dim("Enter on one of the first two changes it to its next value. The rest cannot be changed from here yet.")) },
        ]);
        for tab in 0..TABS.len() {
            let last = self.rows[tab].len().saturating_sub(1);
            let at = self.cursor[tab].selected().unwrap_or(0).min(last);
            self.cursor[tab].select(Some(at));
        }
    }

    fn read_updates(&mut self) {
        // pacman's own lists, as they were last fetched: kav update fetches them again
        self.rows[2] = lines_of(Command::new("pacman").arg("-Qu"))
            .into_iter()
            .filter_map(|line| {
                let mut words = line.split_whitespace();
                let (name, from, _, to) = (words.next()?, words.next()?, words.next()?, words.next()?);
                Some(Row {
                    name: name.to_string(),
                    text: Line::from(vec![Span::raw(format!("{name:<34}")), dim(format!("{from}  ->  ")), Span::raw(to.to_string())]),
                })
            })
            .collect();
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
        let list = read(as_root("snapper").args(["--no-headers", "-c", "root", "list", "--columns", "number,type,date,description"]));
        let mut rows: Vec<Row> = list
            .unwrap_or_default()
            .lines()
            .rev()
            .filter(|line| !line.trim_start().starts_with('0'))
            .map(|line| Row { name: String::new(), text: Line::from(line.trim().to_string()) })
            .collect();
        if rows.is_empty() {
            rows.push(Row { name: String::new(), text: Line::from("There are no snapshots to list.") });
        }
        self.rows[4] = rows;
    }

    fn read_dashboard(&mut self) {
        let updates = lines_of(Command::new("pacman").arg("-Qu"));
        let kavat = lines_of(Command::new("pacman").args(["-Sl", "kavat"]));
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
            updates: updates.len(),
            from_kavat: updates.iter().filter(|line| line.split_whitespace().next().is_some_and(|name| kavat.iter().any(|listed| listed.split_whitespace().nth(1) == Some(name)))).count(),
            waiting: asked.len(),
            waiting_first: asked.first().map(|row| row.name.clone()).unwrap_or_default(),
            installed: lines_of(Command::new("pacman").arg("-Qq")).len(),
            explicit: lines_of(Command::new("pacman").arg("-Qeq")).len(),
            foreign: lines_of(Command::new("pacman").arg("-Qmq")).len(),
            snapshot: self.snapshots_read.then(|| self.rows[4].first().map(|row| row.text.to_string())).flatten().and_then(|line| {
                let mut cells = line.split('│').map(str::trim);
                Some((cells.next()?.to_string(), cells.nth(1)?.to_string()))
            }),
            health,
            activity: activity(),
        };
        self.rows[0].clear();
    }

    /// The dashboard, as the design draws it: four numbers, the health, what was done last
    /// and the news.
    fn draw_dashboard(&self, frame: &mut Frame, area: Rect) {
        let [cards, middle, news] = Layout::vertical([Constraint::Length(5), Constraint::Min(3), Constraint::Length(3)]).areas(area);
        let cells: [Rect; 4] = Layout::horizontal([Constraint::Ratio(1, 4); 4]).areas(cards);
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
            format!("{} Arch · {} [kavat]", board.updates - board.from_kavat, board.from_kavat), "3 see them · u update all"), cells[0]);
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

    /// Look for what was typed, in pacman's repositories and on the reviewed list.
    fn search(&mut self) {
        let words: Vec<&str> = self.query.split_whitespace().collect();
        let mut rows = Vec::new();
        let found = read(Command::new("pacman").arg("-Ss").args(&words)).unwrap_or_default();
        for line in found.lines().filter(|line| !line.starts_with(' ')) {
            let mut parts = line.split_whitespace();
            let (Some(place), Some(version)) = (parts.next(), parts.next()) else { continue };
            let Some((repository, name)) = place.split_once('/') else { continue };
            let installed = if line.contains("[installed") { "installed" } else { "" };
            rows.push(Row {
                name: name.to_string(),
                text: Line::from(vec![Span::raw(format!("{name:<34}")), dim(format!("{repository:<8} {version:<22}")), Span::raw(installed)]),
            });
        }
        if let Ok(list) = Allowlist::load(&self.paths.lists) {
            for entry in list.entries().filter(|entry| words.iter().all(|word| entry.name.contains(word))) {
                rows.push(Row { name: entry.name.clone(), text: Line::from(vec![Span::raw(format!("{:<34}", entry.name)), dim("the AUR, on the reviewed list")]) });
            }
        }
        if rows.is_empty() {
            rows.push(Row { name: String::new(), text: Line::from(format!("Nothing is called {}.", self.query)) });
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
        for (index, name) in TABS.iter().enumerate() {
            let label = format!(" {} {name} ", index + 1);
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

        if self.tab == 0 {
            self.draw_dashboard(frame, body);
        } else {
            let title = match self.tab {
                1 if self.typing => format!(" Browse · looking for: {}▏ ", self.query),
                1 if !self.query.is_empty() => format!(" Browse · {} ", self.query),
                2 => format!(" Updates · {} ", self.rows[2].len()),
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
            1 => vec![("↵", "record"), ("i", "install"), ("x", "remove")],
            2 => vec![("u", "update all")],
            3 => vec![("r", "review")],
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
        ratatui::restore();
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
        println!("\nEnter goes back to kav.");
        let _ = std::io::stdin().read_line(&mut String::new());
        *terminal = ratatui::init();
        self.read_all();
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
            KeyCode::Enter if self.tab == 6 => {
                // the row's name is the setting and the value it would take
                if let Some((name, value)) = self.selected().and_then(|row| row.split_once(' ')) {
                    let what = crate::settings::SETTINGS.iter().find(|s| s.name == name).map_or(name, |s| s.what);
                    self.asked = Some((format!("{what}: {value}"), vec!["set".into(), name.into(), value.into()]));
                }
            }
            KeyCode::Enter if self.tab == 4 => {
                // sudo may ask for the password: it needs the terminal as it is
                ratatui::restore();
                let allowed = as_root("true").status().is_ok_and(|status| status.success());
                *terminal = ratatui::init();
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

pub fn run(paths: Paths) -> Result<(), String> {
    let mut app = App::new(paths);
    let mut terminal = ratatui::init();
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
            Ok(_) => {}
            Err(error) => break Err(error.to_string()),
        }
    };
    ratatui::restore();
    ended
}
