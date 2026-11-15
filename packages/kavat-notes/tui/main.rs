//! kavat-notes-tui: the notes on one screen, as the design draws it.
//!
//! It reads the notes as nb keeps them, Markdown files in ~/Notes, one folder to a
//! notebook, and shows them by what they are for: what is due today, the inbox, every note,
//! the tasks and the reminders, and the albums of sites the browser keeps. It changes nothing
//! by itself: each key that changes a note or an album runs kavat-notes for it, which is what
//! keeps the history and sets the reminders.

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const TABS: [&str; 6] = ["Today", "Inbox", "Notes", "To-do", "Reminders", "Albums"];
/// The tab of the albums, by its place
const ALBUMS: usize = 5;

const HELP: &str = "\
1 to 6, Tab      go to a tab
Up, Down, j, k   move in the list
b, B             the next notebook on the left, or the one before; the lists show only its notes
Enter            open the note in the editor
a                add a note: one line, as in the quick note; it goes to the notebook chosen
n                a note from a template: meeting, bug, idea, daily and yours
D                today's daily note
Space            mark a task as done, or as open again
t                make the note a task, or a task a note again
r                set a reminder: \"tomorrow 9:00\", \"in 2 hours\"; empty takes it off
d                the day a task is due: \"friday\", \"2026-12-01\"
o                open the folder the note points to, in Yazi
/                show only the notes that have what is typed
F5               read the notes again
q, Esc           leave

In Albums
b, B             the next album on the left, or the one before
Enter            open the site in the browser
O                open every site of the album
R                the site has been read: it goes to the album's archive
x                take the site out of the album";

/// One note, as much of it as the screen shows.
#[derive(Clone, Default)]
struct Note {
    path: PathBuf,
    notebook: String,
    title: String,
    /// None for a plain note; Some(true) for a task that is done
    done: Option<bool>,
    /// "2026-11-13 09:00"
    remind: String,
    repeat: String,
    /// "2026-11-13"
    due: String,
    tags: Vec<String>,
    tasks: Vec<(bool, String)>,
    related: Vec<String>,
    /// seconds since 1970, for the order of the lists
    changed: u64,
}

impl Note {
    /// When the note asks for attention: its reminder, or the end of the day it is due.
    fn when(&self) -> String {
        if !self.remind.is_empty() {
            self.remind.clone()
        } else if !self.due.is_empty() {
            format!("{} 23:59", self.due)
        } else {
            String::new()
        }
    }

    fn open(&self) -> bool {
        self.done != Some(true)
    }
}

fn read_note(path: &Path, notebook: &str) -> Option<Note> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut note = Note { path: path.to_path_buf(), notebook: notebook.to_string(), ..Note::default() };
    note.changed = std::fs::metadata(path).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let mut lines = text.lines().peekable();
    if lines.peek() == Some(&"---") {
        lines.next();
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
            if let Some(value) = line.strip_prefix("remind: ") {
                note.remind = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("repeat: ") {
                note.repeat = value.trim().to_string();
            }
        }
    }
    let mut section = String::new();
    for line in lines {
        if let Some(heading) = line.strip_prefix("# ") {
            if note.title.is_empty() {
                let (done, title) = match heading {
                    _ if heading.starts_with("[ ] ") => (Some(false), &heading[4..]),
                    _ if heading.starts_with("[x] ") || heading.starts_with("[X] ") => (Some(true), &heading[4..]),
                    _ => (None, heading),
                };
                note.done = done;
                note.title = title.to_string();
            }
        } else if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim().to_lowercase();
        } else if section == "due" && note.due.is_empty() && line.len() >= 10 && line.as_bytes()[4] == b'-' {
            note.due = line[..10].to_string();
        } else if section == "tasks" && line.starts_with("- [") {
            note.tasks.push((line.starts_with("- [x]") || line.starts_with("- [X]"), line.get(6..).unwrap_or("").to_string()));
        } else if section == "related" && line.starts_with("- ") {
            note.related.push(line[2..].to_string());
        } else if section.is_empty() || section == "tags" {
            for word in line.split_whitespace().filter(|word| word.len() > 1 && word.starts_with('#') && !word[1..].starts_with('#')) {
                note.tags.push(word.to_string());
            }
        }
    }
    if note.title.is_empty() {
        note.title = path.file_stem()?.to_string_lossy().into_owned();
    }
    Some(note)
}

/// One site of an album: a bookmark as nb writes it, and its thumbnail when it has one.
#[derive(Clone, Default)]
struct Site {
    path: PathBuf,
    title: String,
    address: String,
    thumbnail: Option<PathBuf>,
}

struct Album {
    name: String,
    sites: Vec<Site>,
}

/// The albums, which are the folders of home/albums: read-later first, which is always
/// there, and the rest by name. What an album has archived is not read.
fn read_albums(folder: &Path) -> Vec<Album> {
    let inside = folder.join("home/albums");
    let mut names: Vec<String> = std::fs::read_dir(&inside).into_iter().flatten().filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.') && name != "read-later")
        .collect();
    names.sort();
    names.insert(0, "read-later".into());
    names.into_iter().map(|name| {
        let mut sites: Vec<Site> = std::fs::read_dir(inside.join(&name)).into_iter().flatten().filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.to_string_lossy().ends_with(".bookmark.md"))
            .filter_map(|path| read_site(&path))
            .collect();
        sites.sort_by(|a, b| a.path.cmp(&b.path));
        Album { name, sites }
    }).collect()
}

fn read_site(path: &Path) -> Option<Site> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut site = Site { path: path.to_path_buf(), ..Site::default() };
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            if site.title.is_empty() {
                site.title = title.trim().to_string();
            }
        } else if let Some(address) = line.strip_prefix('<').and_then(|rest| rest.strip_suffix('>')) {
            if site.address.is_empty() {
                site.address = address.to_string();
            }
        }
    }
    if site.title.is_empty() {
        site.title = site.address.clone();
    }
    let thumbnail = PathBuf::from(path.to_string_lossy().replace(".bookmark.md", ".jpg"));
    site.thumbnail = thumbnail.is_file().then_some(thumbnail);
    Some(site)
}

/// The notebooks, which are the folders of the notes, by name.
fn read_books(folder: &Path) -> Vec<String> {
    let mut books: Vec<String> = std::fs::read_dir(folder).into_iter().flatten().filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect();
    books.sort();
    books
}

fn read_notes(folder: &Path) -> Vec<Note> {
    let mut notes = Vec::new();
    // the sites of the albums are not notes: they have their own tab
    let albums = folder.join("home/albums");
    for notebook in std::fs::read_dir(folder).into_iter().flatten().filter_map(Result::ok) {
        let name = notebook.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || !notebook.path().is_dir() {
            continue;
        }
        let mut folders = vec![notebook.path()];
        while let Some(inside) = folders.pop() {
            for entry in std::fs::read_dir(&inside).into_iter().flatten().filter_map(Result::ok) {
                let path = entry.path();
                let hidden = entry.file_name().to_string_lossy().starts_with('.');
                if path.is_dir() && !hidden {
                    if path != albums {
                        folders.push(path);
                    }
                } else if !hidden && path.extension().is_some_and(|ending| ending == "md") {
                    notes.extend(read_note(&path, &name));
                }
            }
        }
    }
    notes.sort_by(|a, b| b.changed.cmp(&a.changed));
    notes
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

/// Now, as "2026-11-13 09:00", by the clock and the zone of the system.
fn now() -> String {
    let output = Command::new("date").arg("+%Y-%m-%d %H:%M").stderr(Stdio::null()).output();
    output.ok().map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string()).unwrap_or_default()
}

fn dim(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().add_modifier(Modifier::DIM))
}

/// One line of the list: a note, by its place in the notes, a site, by its place in the
/// album in view, or a heading that is neither.
enum Row {
    Heading(&'static str, Color),
    Note(usize),
    Site(usize),
}

/// What a line typed at the bottom is for.
#[derive(Clone, Copy, PartialEq)]
enum Asking {
    Add,
    Remind,
    Due,
    Filter,
    /// the title of a note made from the template chosen
    Title,
}

struct App {
    folder: PathBuf,
    notes: Vec<Note>,
    books: Vec<String>,
    /// the notebook the lists show, when one is chosen
    book: Option<String>,
    albums: Vec<Album>,
    /// the album in view, by its place
    album: usize,
    /// the templates offered, and the one under the cursor, while they are
    choosing: Option<(Vec<String>, usize)>,
    template: String,
    accent: Color,
    tab: usize,
    rows: Vec<Row>,
    cursor: ListState,
    filter: String,
    asking: Option<Asking>,
    typed: String,
    help: bool,
    said: String,
    now: String,
}

impl App {
    fn new() -> App {
        let home = std::env::var("HOME").unwrap_or_default();
        let folder = std::env::var("NB_DIR").ok().filter(|value| !value.is_empty()).map_or_else(|| PathBuf::from(home).join("Notes"), PathBuf::from);
        let mut app = App {
            folder,
            notes: Vec::new(),
            books: Vec::new(),
            book: None,
            albums: Vec::new(),
            album: 0,
            choosing: None,
            template: String::new(),
            accent: accent(),
            tab: 0,
            rows: Vec::new(),
            cursor: ListState::default(),
            filter: String::new(),
            asking: None,
            typed: String::new(),
            help: false,
            said: String::new(),
            now: String::new(),
        };
        app.read();
        app
    }

    fn bold(&self, text: impl Into<String>) -> Span<'static> {
        Span::styled(text.into(), Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
    }

    fn read(&mut self) {
        self.notes = read_notes(&self.folder);
        self.books = read_books(&self.folder);
        if self.book.as_ref().is_some_and(|book| !self.books.contains(book)) {
            self.book = None;
        }
        self.albums = read_albums(&self.folder);
        self.album = self.album.min(self.albums.len().saturating_sub(1));
        self.now = now();
        self.list();
    }

    fn shown(&self, note: &Note) -> bool {
        // the inbox is a notebook itself: its tab shows it whichever is chosen
        if self.tab != 1 && self.book.as_ref().is_some_and(|book| *book != note.notebook) {
            return false;
        }
        let filter = self.filter.to_lowercase();
        filter.is_empty() || note.title.to_lowercase().contains(&filter) || note.tags.iter().any(|tag| tag.to_lowercase().contains(&filter))
    }

    /// Make the list of the tab in view, keeping the cursor on a note.
    fn list(&mut self) {
        let today = self.now.get(..10).unwrap_or("").to_string();
        let mut rows = Vec::new();
        let with = |keep: &dyn Fn(&Note) -> bool| -> Vec<usize> {
            self.notes.iter().enumerate().filter(|(_, note)| self.shown(note) && keep(note)).map(|(index, _)| index).collect()
        };
        let by_time = |mut found: Vec<usize>| -> Vec<usize> {
            found.sort_by_key(|&index| self.notes[index].when());
            found
        };
        match self.tab {
            0 => {
                let dated = |note: &Note| note.open() && !note.when().is_empty();
                // a reminder is overdue once its time has gone; a task, once its day has
                let overdue = |note: &Note| if note.remind.is_empty() { note.due.as_str() < today.as_str() } else { note.remind.as_str() <= self.now.as_str() };
                let groups: [(&'static str, Color, Vec<usize>); 3] = [
                    ("OVERDUE", Color::Red, by_time(with(&|note| dated(note) && overdue(note)))),
                    ("TODAY", Color::Reset, by_time(with(&|note| dated(note) && !overdue(note) && note.when().starts_with(&today)))),
                    ("UPCOMING", Color::Reset, by_time(with(&|note| dated(note) && !overdue(note) && !note.when().starts_with(&today)))),
                ];
                for (name, colour, found) in groups {
                    if !found.is_empty() {
                        rows.push(Row::Heading(name, colour));
                        rows.extend(found.into_iter().map(Row::Note));
                    }
                }
                let recent: Vec<usize> = with(&|note| note.done.is_none() && note.when().is_empty()).into_iter().take(6).collect();
                if !recent.is_empty() {
                    rows.push(Row::Heading("RECENT NOTES", Color::Reset));
                    rows.extend(recent.into_iter().map(Row::Note));
                }
            }
            1 => rows.extend(with(&|note| note.notebook == "inbox").into_iter().map(Row::Note)),
            2 => rows.extend(with(&|_| true).into_iter().map(Row::Note)),
            3 => {
                let mut tasks = with(&|note| note.done.is_some());
                tasks.sort_by_key(|&index| (!self.notes[index].open(), self.notes[index].when().is_empty(), self.notes[index].when()));
                rows.extend(tasks.into_iter().map(Row::Note));
            }
            4 => rows.extend(by_time(with(&|note| !note.remind.is_empty())).into_iter().map(Row::Note)),
            _ => {
                let filter = self.filter.to_lowercase();
                let sites = self.albums.get(self.album).map_or(&[][..], |album| &album.sites[..]);
                let found = sites.iter().enumerate().filter(|(_, site)| {
                    filter.is_empty() || site.title.to_lowercase().contains(&filter) || site.address.to_lowercase().contains(&filter)
                });
                rows.extend(found.map(|(index, _)| Row::Site(index)));
            }
        }
        self.rows = rows;
        let lands = |row: Option<&Row>| matches!(row, Some(Row::Note(_) | Row::Site(_)));
        let first = self.rows.iter().position(|row| lands(Some(row)));
        let at = self.cursor.selected().filter(|&at| lands(self.rows.get(at))).or(first);
        self.cursor.select(at);
    }

    /// The site under the cursor, in the tab of the albums.
    fn site(&self) -> Option<&Site> {
        match self.rows.get(self.cursor.selected()?)? {
            Row::Site(index) => self.albums.get(self.album)?.sites.get(*index),
            _ => None,
        }
    }

    /// Choose the next line of the left column, or the one before: an album in the tab of
    /// the albums, and a notebook in the others, where the first line is all of them.
    fn choose(&mut self, by: isize) {
        if self.tab == ALBUMS {
            let count = self.albums.len().max(1) as isize;
            self.album = (self.album as isize + by).rem_euclid(count) as usize;
        } else {
            let count = self.books.len() as isize + 1;
            let at = self.book.as_ref().and_then(|book| self.books.iter().position(|name| name == book)).map_or(0, |at| at as isize + 1);
            let at = (at + by).rem_euclid(count);
            self.book = (at > 0).then(|| self.books[at as usize - 1].clone());
        }
        self.cursor.select(None);
        self.list();
    }

    fn selected(&self) -> Option<&Note> {
        match self.rows.get(self.cursor.selected()?)? {
            Row::Note(index) => self.notes.get(*index),
            _ => None,
        }
    }

    fn move_by(&mut self, by: isize) {
        let count = self.rows.len() as isize;
        let mut at = self.cursor.selected().map_or(-1, |at| at as isize);
        // over the headings, which are not notes
        loop {
            at += by.signum();
            if at < 0 || at >= count {
                return;
            }
            if matches!(self.rows[at as usize], Row::Note(_) | Row::Site(_)) {
                self.cursor.select(Some(at as usize));
                return;
            }
        }
    }

    /// A day and a time as the list shows them: the time for today, the day otherwise.
    fn short(&self, when: &str) -> String {
        if when.is_empty() {
            return String::new();
        }
        let today = self.now.get(..10).unwrap_or("");
        let (day, time) = when.split_at(10.min(when.len()));
        if day == today {
            time.trim().to_string()
        } else {
            let output = Command::new("date").args(["-d", day, "+%a %-d %b"]).stderr(Stdio::null()).output();
            output.ok().map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string()).unwrap_or_else(|| day.to_string())
        }
    }

    fn line_of(&self, note: &Note, width: usize) -> Line<'static> {
        let mark = match note.done {
            Some(true) => "[x] ",
            Some(false) => "[ ] ",
            None => " ·  ",
        };
        let mut right = String::new();
        if !note.remind.is_empty() {
            right.push_str(&format!("⏰ {}", self.short(&note.remind)));
            if !note.repeat.is_empty() {
                right.push_str(&format!("  ↻ {}", note.repeat));
            }
        } else if !note.due.is_empty() {
            right.push_str(&self.short(&note.when()));
        } else {
            right.push_str(&note.notebook);
        }
        let room = width.saturating_sub(mark.chars().count() + right.chars().count() + 3);
        let title: String = note.title.chars().take(room).collect();
        let pad = room.saturating_sub(title.chars().count()) + 1;
        let style = if note.done == Some(true) { Style::default().add_modifier(Modifier::DIM) } else { Style::default() };
        Line::from(vec![Span::styled(format!("{mark}{title}"), style), Span::raw(" ".repeat(pad)), dim(right)])
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [top, body, bottom] = Layout::vertical([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

        let mut header = vec![self.bold(" notes "), Span::raw(" ")];
        for (index, name) in TABS.iter().enumerate() {
            let label = format!(" {} {name} ", index + 1);
            header.push(if index == self.tab {
                Span::styled(label, Style::default().bg(self.accent).fg(Color::Black).add_modifier(Modifier::BOLD))
            } else {
                dim(label)
            });
            header.push(Span::raw(" "));
        }
        let place = format!("{} · {} notes ", self.folder.display().to_string().replace(&std::env::var("HOME").unwrap_or_default(), "~"), self.notes.len());
        let [tabs, right] = Layout::horizontal([Constraint::Min(1), Constraint::Length(place.chars().count() as u16)]).areas(top);
        frame.render_widget(Paragraph::new(Line::from(header)), tabs);
        frame.render_widget(Paragraph::new(Line::from(dim(place))), right);

        let [left, middle, detail] = Layout::horizontal([Constraint::Length(26), Constraint::Min(30), Constraint::Percentage(38)]).areas(body);
        let accent = self.accent;
        let pane = move |title: String, lit: bool| {
            let border = if lit { Style::default().fg(accent) } else { Style::default().add_modifier(Modifier::DIM) };
            let title = Span::styled(format!(" {title} "), Style::default().fg(accent).add_modifier(Modifier::BOLD));
            Block::default().borders(Borders::ALL).border_style(border).title(title)
        };

        // one line of the left column: a name and how many it holds, lit when it is the one chosen
        let choice = |name: &str, count: usize, chosen: bool| {
            let name: String = name.chars().take(16).collect();
            if chosen {
                Line::from(Span::styled(format!("{name:<16}{count:>5} "), Style::default().bg(accent).fg(Color::Black)))
            } else {
                Line::from(vec![Span::raw(format!("{name:<16}")), dim(format!("{count:>5}"))])
            }
        };
        if self.tab == ALBUMS {
            // the albums with how many sites each holds
            let mut side: Vec<Line> = self.albums.iter().enumerate().map(|(index, album)| choice(&album.name, album.sites.len(), index == self.album)).collect();
            side.push(Line::from(""));
            side.push(Line::from(dim("b, B choose an album")));
            frame.render_widget(Paragraph::new(side).block(pane("Albums".into(), false)), left);
        } else {
            // the notebooks with how many notes each holds, and the tags that are used most
            let mut held: BTreeMap<&str, usize> = BTreeMap::new();
            let mut tags: BTreeMap<&str, usize> = BTreeMap::new();
            for note in &self.notes {
                *held.entry(&note.notebook).or_default() += 1;
                for tag in &note.tags {
                    *tags.entry(tag).or_default() += 1;
                }
            }
            let mut side = vec![choice("All", self.notes.len(), self.book.is_none())];
            side.extend(self.books.iter().map(|name| choice(name, held.get(name.as_str()).copied().unwrap_or(0), self.book.as_ref() == Some(name))));
            let mut used: Vec<(&str, usize)> = tags.into_iter().collect();
            used.sort_by(|a, b| b.1.cmp(&a.1));
            if !used.is_empty() {
                side.push(Line::from(""));
                side.push(Line::from(dim("TAGS")));
                side.extend(used.iter().take(12).map(|(tag, _)| Line::from(Span::styled(tag.to_string(), Style::default().fg(Color::Blue)))));
            }
            frame.render_widget(Paragraph::new(side).block(pane("Notebooks".into(), false)), left);
        }

        let width = middle.width.saturating_sub(4) as usize;
        let mut items: Vec<ListItem> = Vec::new();
        for row in &self.rows {
            items.push(match row {
                Row::Heading(name, colour) => ListItem::new(Line::from(Span::styled(*name, Style::default().fg(*colour).add_modifier(Modifier::DIM)))),
                Row::Note(index) => ListItem::new(self.line_of(&self.notes[*index], width)),
                Row::Site(index) => {
                    // the title, and as much of the address as there is room for
                    let site = &self.albums[self.album].sites[*index];
                    let address: String = site.address.chars().take(width / 2).collect();
                    let room = width.saturating_sub(address.chars().count() + 7);
                    let title: String = site.title.chars().take(room).collect();
                    let pad = room.saturating_sub(title.chars().count()) + 1;
                    let mark = if site.thumbnail.is_some() { " ▣  " } else { " ·  " };
                    ListItem::new(Line::from(vec![Span::raw(format!("{mark}{title}")), Span::raw(" ".repeat(pad)), dim(address)]))
                }
            });
        }
        if items.is_empty() {
            let nothing = match self.tab {
                0 => "Nothing is due, and there are no notes yet. a adds one.",
                1 => "The inbox is empty. Super+N saves a note here from anywhere.",
                3 => "There are no tasks. t makes a note a task.",
                4 => "There are no reminders. r sets one on a note.",
                ALBUMS => "This album has no sites. The browser keeps them here.",
                _ => "There are no notes yet. a adds one.",
            };
            items.push(ListItem::new(Line::from(dim(nothing))));
        }
        let mut title = match (self.tab, &self.book) {
            (ALBUMS, _) => self.albums.get(self.album).map_or("Albums".to_string(), |album| format!("Albums · {}", album.name)),
            (tab, Some(book)) if tab != 1 => format!("{} · {book}", TABS[tab]),
            (tab, _) => TABS[tab].to_string(),
        };
        if !self.filter.is_empty() {
            title = format!("{title} · {}", self.filter);
        }
        let list = List::new(items).block(pane(title, true)).highlight_style(Style::default().bg(self.accent).fg(Color::Black));
        frame.render_stateful_widget(list, middle, &mut self.cursor);

        // what the note under the cursor says
        let mut about: Vec<Line> = Vec::new();
        let mut name = if self.tab == ALBUMS { "Site" } else { "Note" }.to_string();
        if let Some(note) = self.selected() {
            name = note.title.clone();
            let field = |label: &str, value: String| Line::from(vec![dim(format!("{label:<10}")), Span::raw(value)]);
            if !note.due.is_empty() {
                about.push(field("Due", self.short(&format!("{} 00:00", note.due))));
            }
            if !note.remind.is_empty() {
                let again = if note.repeat.is_empty() { String::new() } else { format!(" · repeats {}", note.repeat) };
                about.push(field("Remind", format!("{}{again}", note.remind)));
            }
            about.push(Line::from(vec![dim(format!("{:<10}", "Notebook")), Span::raw(format!("{} ", note.notebook)), Span::styled(note.tags.join(" "), Style::default().fg(Color::Blue))]));
            if let Some(folder) = note.related.iter().find_map(|link| folder_of(link)) {
                about.push(Line::from(vec![dim(format!("{:<10}", "Folder")), self.bold(folder), dim("  o opens it in Yazi")]));
            }
            if !note.tasks.is_empty() {
                about.push(Line::from(""));
                about.push(Line::from(dim("TASKS")));
                for (done, task) in &note.tasks {
                    about.push(if *done { Line::from(dim(format!("[x] {task}"))) } else { Line::from(format!("[ ] {task}")) });
                }
            }
            if !note.related.is_empty() {
                about.push(Line::from(""));
                about.push(Line::from(dim("RELATED")));
                about.extend(note.related.iter().map(|link| Line::from(Span::styled(link.clone(), Style::default().fg(Color::Blue)))));
            }
        } else if let Some(site) = self.site() {
            name = site.title.clone();
            let short = |path: &Path| path.strip_prefix(&self.folder).unwrap_or(path).display().to_string();
            about.push(Line::from(vec![dim(format!("{:<10}", "Address")), Span::styled(site.address.clone(), Style::default().fg(Color::Blue))]));
            about.push(Line::from(vec![dim(format!("{:<10}", "File")), Span::raw(short(&site.path))]));
            about.push(Line::from(vec![dim(format!("{:<10}", "Picture")), match &site.thumbnail {
                Some(thumbnail) => Span::raw(short(thumbnail)),
                None => dim("none"),
            }]));
            about.push(Line::from(""));
            about.push(Line::from(dim("↵ opens it in the browser · O the whole album")));
            about.push(Line::from(dim("R it has been read · x takes it out")));
        }
        frame.render_widget(Paragraph::new(about).wrap(Wrap { trim: false }).block(pane(name, false)), detail);

        // the keys, or the line being typed, or what the last key ended in
        let mut bar = vec![Span::raw(" ")];
        if let Some(asking) = self.asking {
            let label = match asking {
                Asking::Add => "Add",
                Asking::Remind => "Remind",
                Asking::Due => "Due",
                Asking::Filter => "Show only",
                Asking::Title => "Title",
            };
            bar.push(self.bold(format!("{label}: ")));
            bar.push(Span::raw(format!("{}▏", self.typed)));
        } else if !self.said.is_empty() {
            bar.push(Span::raw(self.said.clone()));
        } else {
            let keys: &[(&str, &str)] = if self.tab == ALBUMS {
                &[("↵", "open in the browser"), ("O", "open them all"), ("R", "read"), ("x", "take out"), ("b", "album"), ("/", "search"), ("?", "help")]
            } else {
                &[("↵", "open in Neovim"), ("a", "add"), ("n", "template"), ("t", "to-do"), ("r", "remind"), ("d", "due date"), ("Space", "done"), ("b", "notebook"), ("/", "search"), ("?", "help")]
            };
            for (key, what) in keys {
                bar.push(Span::styled(format!(" {key} "), Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)));
                bar.push(dim(format!(" {what}  ")));
            }
        }
        frame.render_widget(Paragraph::new(Line::from(bar)), bottom);

        if self.help {
            let area = centred(frame.area(), 70, 80);
            frame.render_widget(Clear, area);
            frame.render_widget(Paragraph::new(HELP).block(Block::default().borders(Borders::ALL).title(" Help · any key closes ")), area);
        }
        if let Some((templates, at)) = &self.choosing {
            let area = centred(frame.area(), 40, 50);
            let items: Vec<ListItem> = templates.iter().map(|name| ListItem::new(format!(" {name}"))).collect();
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" A note from a template · ↵ takes it, Esc closes "))
                .highlight_style(Style::default().bg(self.accent).fg(Color::Black));
            frame.render_widget(Clear, area);
            frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(*at)));
        }
    }

    /// Ask kavat-notes for something it answers with a line, like the file of a note it made.
    fn ask(&mut self, arguments: &[&str]) -> Option<String> {
        let output = Command::new("kavat-notes").args(arguments).stdin(Stdio::null()).output();
        match output {
            Ok(output) if output.status.success() => Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
            Ok(output) => {
                self.said = String::from_utf8_lossy(&output.stderr).lines().last().unwrap_or("kavat-notes could not do it").to_string();
                None
            }
            Err(error) => {
                self.said = format!("kavat-notes: {error}");
                None
            }
        }
    }

    /// add or new and what follows it, with the notebook chosen, when there is one, in between.
    fn to_book(&self, command: &str, rest: &[&str]) -> Vec<String> {
        let mut arguments = vec![command.to_string()];
        if let Some(book) = &self.book {
            arguments.extend(["--to".to_string(), book.clone()]);
        }
        arguments.extend(rest.iter().filter(|word| !word.is_empty()).map(|word| word.to_string()));
        arguments
    }

    /// Open a note in the editor, which takes the terminal and gives it back.
    fn edit(&mut self, terminal: &mut DefaultTerminal, path: &str) {
        ratatui::restore();
        let editor = std::env::var("EDITOR").ok().filter(|value| !value.is_empty()).unwrap_or_else(|| "nvim".into());
        let _ = Command::new(editor).arg(path).status();
        *terminal = ratatui::init();
        self.change(&["sync"]);
    }

    /// Run kavat-notes for a change, quietly, and say what it said when it could not.
    fn change(&mut self, arguments: &[&str]) {
        let output = Command::new("kavat-notes").args(arguments).stdin(Stdio::null()).output();
        self.said = match output {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stderr).lines().last().unwrap_or("").to_string(),
            Ok(output) => String::from_utf8_lossy(&output.stderr).lines().last().unwrap_or("kavat-notes could not do it").to_string(),
            Err(error) => format!("kavat-notes: {error}"),
        };
        self.read();
    }

    /// The line typed at the bottom, once Enter ends it.
    fn answer(&mut self, terminal: &mut DefaultTerminal, asking: Asking) {
        let typed = std::mem::take(&mut self.typed);
        if asking == Asking::Title {
            // the note is made from the template, and opened to be written
            let template = std::mem::take(&mut self.template);
            let arguments = self.to_book("new", &[&template, typed.trim()]);
            let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
            match self.ask(&arguments) {
                Some(path) if !path.is_empty() => self.edit(terminal, &path),
                _ => self.read(),
            }
            return;
        }
        let path = self.selected().map(|note| note.path.to_string_lossy().into_owned());
        match (asking, path) {
            (Asking::Filter, _) => {
                self.filter = typed.trim().to_string();
                self.list();
            }
            (Asking::Add, _) if !typed.trim().is_empty() => {
                let arguments = self.to_book("add", &[typed.trim()]);
                self.change(&arguments.iter().map(String::as_str).collect::<Vec<&str>>());
            }
            (Asking::Remind, Some(path)) if typed.trim().is_empty() => self.change(&["remind", "--off", &path]),
            (Asking::Remind, Some(path)) => self.change(&["remind", &path, typed.trim()]),
            (Asking::Due, Some(path)) if !typed.trim().is_empty() => self.change(&["due", &path, typed.trim()]),
            _ => {}
        }
    }

    /// One key. False when it is time to leave.
    fn key(&mut self, terminal: &mut DefaultTerminal, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if self.help {
            self.help = false;
            return true;
        }
        if let Some(asking) = self.asking {
            match code {
                KeyCode::Enter => {
                    self.asking = None;
                    self.answer(terminal, asking);
                }
                KeyCode::Esc => {
                    self.asking = None;
                    self.typed.clear();
                }
                KeyCode::Backspace => {
                    self.typed.pop();
                }
                KeyCode::Char(letter) => self.typed.push(letter),
                _ => {}
            }
            return true;
        }
        if let Some((templates, at)) = &mut self.choosing {
            match code {
                KeyCode::Down | KeyCode::Char('j') => *at = (*at + 1) % templates.len(),
                KeyCode::Up | KeyCode::Char('k') => *at = (*at + templates.len() - 1) % templates.len(),
                KeyCode::Enter => {
                    self.template = templates[*at].clone();
                    self.choosing = None;
                    self.asking = Some(Asking::Title);
                }
                _ => self.choosing = None,
            }
            return true;
        }
        self.said.clear();
        let path = self.selected().map(|note| note.path.to_string_lossy().into_owned());
        if self.tab == ALBUMS && self.album_key(code) {
            return true;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return false,
            KeyCode::Char(digit @ '1'..='6') => {
                self.tab = digit as usize - '1' as usize;
                self.cursor.select(None);
                self.list();
            }
            KeyCode::Tab | KeyCode::Right => {
                self.tab = (self.tab + 1) % TABS.len();
                self.cursor.select(None);
                self.list();
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.tab = (self.tab + TABS.len() - 1) % TABS.len();
                self.cursor.select(None);
                self.list();
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_by(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_by(-1),
            KeyCode::Char('?') => self.help = true,
            KeyCode::F(5) => self.read(),
            KeyCode::Char('a') => self.asking = Some(Asking::Add),
            KeyCode::Char('b') => self.choose(1),
            KeyCode::Char('B') => self.choose(-1),
            KeyCode::Char('n') => {
                if let Some(names) = self.ask(&["new"]) {
                    let templates: Vec<String> = names.lines().map(String::from).collect();
                    if templates.is_empty() {
                        self.said = "There are no templates: they are files in ~/.config/kavat/notes/templates.".into();
                    } else {
                        self.choosing = Some((templates, 0));
                    }
                }
            }
            KeyCode::Char('D') => {
                if let Some(path) = self.ask(&["daily"]).filter(|path| !path.is_empty()) {
                    self.edit(terminal, &path);
                }
            }
            KeyCode::Char('/') => {
                self.asking = Some(Asking::Filter);
                self.typed = self.filter.clone();
            }
            KeyCode::Char('r') if path.is_some() => self.asking = Some(Asking::Remind),
            KeyCode::Char('d') if path.is_some() => self.asking = Some(Asking::Due),
            KeyCode::Char(' ') => {
                if let Some(path) = path {
                    self.change(&["done", &path]);
                }
            }
            KeyCode::Char('t') => {
                if let Some(path) = path {
                    self.change(&["todo", &path]);
                }
            }
            KeyCode::Char('o') => {
                let folder = self.selected().and_then(|note| note.related.iter().find_map(|link| folder_of(link)));
                match folder {
                    Some(folder) => {
                        let home = std::env::var("HOME").unwrap_or_default();
                        let _ = Command::new("setsid").args(["-f", "kitty", "yazi", &folder.replacen('~', &home, 1)])
                            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    }
                    None => self.said = "This note points to no folder: a line like - [[~/Projects/kav]] under ## Related does.".into(),
                }
            }
            KeyCode::Enter => {
                if let Some(path) = path {
                    self.edit(terminal, &path);
                }
            }
            _ => {}
        }
        true
    }

    /// A key of the tab of the albums. False when the key is not one of its own.
    fn album_key(&mut self, code: KeyCode) -> bool {
        let Some(album) = self.albums.get(self.album).map(|album| album.name.clone()) else {
            return false;
        };
        let site = self.site().cloned();
        match (code, site) {
            (KeyCode::Enter, Some(site)) => {
                let opened = Command::new("setsid").args(["-f", "kavat-browser", &site.address])
                    .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
                self.said = match opened {
                    Ok(status) if status.success() => format!("Opened in the browser: {}", site.address),
                    _ => "The browser, kavat-browser, could not be opened.".into(),
                };
            }
            (KeyCode::Char('O'), _) => {
                self.change(&["album", "open", &album]);
                if self.said.is_empty() {
                    self.said = format!("Opened in the browser: every site of {album}");
                }
            }
            (KeyCode::Char('R'), Some(site)) => {
                self.change(&["album", "read", &site.path.to_string_lossy()]);
                if self.said.is_empty() {
                    self.said = format!("Read, and archived: {}", site.title);
                }
            }
            (KeyCode::Char('x'), Some(site)) => {
                self.change(&["album", "remove", &album, &site.path.to_string_lossy()]);
                if self.said.is_empty() {
                    self.said = format!("Taken out of {album}: {}", site.title);
                }
            }
            (KeyCode::Enter | KeyCode::Char('R' | 'x' | ' ' | 't' | 'r' | 'd' | 'o'), None) => {}
            _ => return false,
        }
        true
    }
}

/// The folder a link names, when it names one: [[~/Projects/kav]] or a bare path.
fn folder_of(link: &str) -> Option<String> {
    let inside = link.trim().trim_start_matches("[[").trim_end_matches("]]");
    (inside.starts_with("~/") || inside.starts_with('/')).then(|| inside.to_string())
}

/// A rectangle in the middle of another, as a percentage of each side.
fn centred(area: Rect, width: u16, height: u16) -> Rect {
    let [_, row, _] = Layout::vertical([Constraint::Percentage((100 - height) / 2), Constraint::Percentage(height), Constraint::Min(0)]).areas(area);
    let [_, cell, _] = Layout::horizontal([Constraint::Percentage((100 - width) / 2), Constraint::Percentage(width), Constraint::Min(0)]).areas(row);
    cell
}

fn main() {
    let mut app = App::new();
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
    if let Err(problem) = ended {
        eprintln!("kavat-notes-tui: {problem}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_is_read_with_its_reminder_its_day_its_tags_and_its_lists() {
        let folder = std::env::temp_dir().join(format!("kavat-notes-tui-test-{}", std::process::id()));
        std::fs::create_dir_all(folder.join("kav")).unwrap();
        let path = folder.join("kav/review.todo.md");
        std::fs::write(&path, "---\nremind: 2026-11-13 09:00\nrepeat: weekly\n---\n# [ ] Review the kav PR\n\n#kav #review\n\n## Due\n\n2026-11-13\n\n## Tasks\n\n- [x] Read the diff\n- [ ] Merge\n\n## Related\n\n- [[~/Projects/kav]]\n").unwrap();
        let notes = read_notes(&folder);
        std::fs::remove_dir_all(&folder).unwrap();
        assert_eq!(notes.len(), 1);
        let note = &notes[0];
        assert_eq!((note.notebook.as_str(), note.title.as_str(), note.done), ("kav", "Review the kav PR", Some(false)));
        assert_eq!((note.remind.as_str(), note.repeat.as_str(), note.due.as_str()), ("2026-11-13 09:00", "weekly", "2026-11-13"));
        assert_eq!(note.tags, ["#kav", "#review"]);
        assert_eq!(note.tasks, [(true, "Read the diff".to_string()), (false, "Merge".to_string())]);
        assert_eq!(folder_of(&note.related[0]).as_deref(), Some("~/Projects/kav"));
    }

    #[test]
    fn an_album_is_read_with_its_sites_and_they_are_not_notes() {
        let folder = std::env::temp_dir().join(format!("kavat-notes-tui-albums-{}", std::process::id()));
        let album = folder.join("home/albums/rust");
        std::fs::create_dir_all(album.join("archive")).unwrap();
        std::fs::write(album.join("20261115100000-docs.bookmark.md"), "# Rust docs\n\n<https://docs.rs/>\n\n#album\n").unwrap();
        std::fs::write(album.join("20261115100000-docs.jpg"), "picture").unwrap();
        std::fs::write(album.join("20261115100001-book.bookmark.md"), "# The book\n\n<https://doc.rust-lang.org/book/>\n\n#album\n").unwrap();
        std::fs::write(album.join("archive/20261115090000-old.bookmark.md"), "# Old\n\n<https://old.example/>\n\n#album\n").unwrap();
        std::fs::write(folder.join("home/plain.md"), "# A note\n").unwrap();
        let (albums, notes, books) = (read_albums(&folder), read_notes(&folder), read_books(&folder));
        std::fs::remove_dir_all(&folder).unwrap();
        assert_eq!(albums.iter().map(|album| (album.name.as_str(), album.sites.len())).collect::<Vec<_>>(), [("read-later", 0), ("rust", 2)]);
        let site = &albums[1].sites[0];
        assert_eq!((site.title.as_str(), site.address.as_str(), site.thumbnail.is_some()), ("Rust docs", "https://docs.rs/", true));
        assert!(albums[1].sites[1].thumbnail.is_none());
        assert_eq!(notes.iter().map(|note| note.title.as_str()).collect::<Vec<_>>(), ["A note"]);
        assert_eq!(books, ["home"]);
    }
}
