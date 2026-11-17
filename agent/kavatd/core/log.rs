//! The log: one line of JSON for each call, in a file the person can read.
//!
//! It is ~/.local/state/kavat/agent/log.jsonl. A line says when, who asked, the tool and
//! its arguments, the level, whether the person confirmed it, the command that ran, how it
//! ended, the first lines of what it wrote, and how to undo it when the tool says how.

use serde_json::Value;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// How much of what a command wrote stays in the log.
const LINES: usize = 12;
const BYTES: usize = 2000;

pub struct Log {
    path: PathBuf,
    // one line is written at a time, whole
    writing: Mutex<()>,
}

/// The day of a count of days since 1970, as year, month and day.
fn day(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let (era, of_era) = (shifted / 146_097, shifted % 146_097);
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    (year_of_era + era * 400 + u64::from(month <= 2), month, of_year - (153 * shifted_month + 2) / 5 + 1)
}

/// A moment as the log writes it, in UTC: 2026-11-16T09:30:00Z.
pub fn stamp(moment: SystemTime) -> String {
    let seconds = moment.duration_since(UNIX_EPOCH).map_or(0, |since| since.as_secs());
    let (year, month, day) = day(seconds / 86_400);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", seconds % 86_400 / 3600, seconds % 3600 / 60, seconds % 60)
}

/// The first lines of what a command wrote.
pub fn first_lines(text: &str) -> String {
    let mut kept: String = text.lines().take(LINES).collect::<Vec<_>>().join("\n");
    if kept.len() > BYTES {
        let end = (0..=BYTES).rev().find(|at| kept.is_char_boundary(*at)).unwrap_or(0);
        kept.truncate(end);
    }
    kept
}

impl Log {
    pub fn at(path: PathBuf) -> Log {
        Log { path, writing: Mutex::new(()) }
    }

    /// Where the log is when nothing else is said.
    pub fn default_path() -> Result<PathBuf, String> {
        let state = match (std::env::var_os("XDG_STATE_HOME"), std::env::var_os("HOME")) {
            (Some(state), _) if !state.is_empty() => PathBuf::from(state),
            (_, Some(home)) if !home.is_empty() => PathBuf::from(home).join(".local/state"),
            _ => return Err("neither XDG_STATE_HOME nor HOME says where the log goes".into()),
        };
        Ok(state.join("kavat/agent/log.jsonl"))
    }

    fn open(&self) -> Result<std::fs::File, String> {
        let problem = |error: std::io::Error| format!("{}: {error}", self.path.display());
        if let Some(folder) = self.path.parent() {
            std::fs::DirBuilder::new().recursive(true).mode(0o700).create(folder).map_err(problem)?;
        }
        // what was asked of the agent is the person's own business
        std::fs::OpenOptions::new().create(true).append(true).mode(0o600).open(&self.path).map_err(problem)
    }

    /// Whether a line can be written. It is asked before a tool runs: what cannot be
    /// written down is not done.
    pub fn ready(&self) -> Result<(), String> {
        self.open().map(|_| ())
    }

    pub fn append(&self, entry: &Value) -> Result<(), String> {
        let _writing = self.writing.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut file = self.open()?;
        file.write_all(format!("{entry}\n").as_bytes()).map_err(|error| format!("{}: {error}", self.path.display()))
    }

    /// The last `count` lines, the oldest first. A line that is not JSON is left out.
    pub fn last(&self, count: usize) -> Result<Vec<Value>, String> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(format!("{}: {error}", self.path.display())),
        };
        let lines: Vec<&str> = text.lines().collect();
        Ok(lines[lines.len().saturating_sub(count)..].iter().filter_map(|line| serde_json::from_str(line).ok()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_moment_is_written_as_a_date_and_a_time() {
        assert_eq!(stamp(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(stamp(UNIX_EPOCH + Duration::from_secs(1_794_821_400)), "2026-11-16T09:30:00Z");
        assert_eq!(stamp(UNIX_EPOCH + Duration::from_secs(1_709_251_199)), "2024-02-29T23:59:59Z");
    }

    #[test]
    fn only_the_first_lines_of_an_output_are_kept() {
        let long: String = (1..=100).map(|number| format!("line {number}\n")).collect();
        assert_eq!(first_lines(&long).lines().count(), LINES);
        assert!(first_lines(&"é".repeat(5000)).len() <= BYTES);
        assert_eq!(first_lines("one\n"), "one");
    }
}
