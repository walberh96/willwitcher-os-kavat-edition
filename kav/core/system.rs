//! Where kav's files are, and running the system's own tools: pacman, git, snapper.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Where things are. Each one can be moved with a variable, which is how the tests give kav
/// a small world of its own.
pub struct Paths {
    /// the list that comes with the system, and the one of this machine, which has the last word
    pub lists: Vec<PathBuf>,
    /// where the AUR's repositories are fetched to
    pub cache: PathBuf,
    /// what people asked to have reviewed
    pub requests: PathBuf,
    /// the clean system that AUR packages are built in
    pub chroot: PathBuf,
    /// the AUR
    pub aur: String,
}

impl Paths {
    pub fn from_environment() -> Paths {
        let variable = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from);
        let home = variable("HOME").unwrap_or_else(|| "/root".into());
        let cache = variable("XDG_CACHE_HOME").unwrap_or_else(|| home.join(".cache")).join("kav");
        let state = variable("XDG_STATE_HOME").unwrap_or_else(|| home.join(".local/state")).join("kav");
        Paths {
            lists: vec![
                variable("KAV_DATA").unwrap_or_else(|| "/usr/share/kav".into()).join("allowlist.toml"),
                variable("KAV_MACHINE").unwrap_or_else(|| "/etc/kav".into()).join("allowlist.toml"),
            ],
            cache,
            requests: state.join("requests"),
            chroot: "/var/lib/kav/chroot".into(),
            aur: std::env::var("KAV_AUR").unwrap_or_else(|_| "https://aur.archlinux.org".into()),
        }
    }

    /// The list of this machine: where an approval is written.
    pub fn machine_list(&self) -> &PathBuf {
        &self.lists[1]
    }
}

pub fn is_root() -> bool {
    std::fs::metadata("/proc/self").is_ok_and(|me| std::os::unix::fs::MetadataExt::uid(&me) == 0)
}

/// A command that needs root: as it is for root, through sudo for anyone else.
pub fn as_root(program: &str) -> Command {
    if is_root() {
        return Command::new(program);
    }
    let mut command = Command::new("sudo");
    command.arg(program);
    command
}

/// Runs a command in the open, for the person to see and answer. Ok when it succeeded.
pub fn run(command: &mut Command) -> Result<(), String> {
    let name = command.get_program().to_string_lossy().into_owned();
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{name} ended with {}", status.code().map_or("a signal".to_string(), |code| format!("status {code}")))),
        Err(error) => Err(format!("{name}: {error}")),
    }
}

/// Runs a command quietly and gives what it printed, or None when it failed.
pub fn read(command: &mut Command) -> Option<String> {
    let output = command.stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Today, as 2026-11-13, by the clock of the system and in UTC.
pub fn today() -> String {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs());
    date_of(seconds / 86400)
}

/// The date of a day counted from 1970-01-01.
fn date_of(days: u64) -> String {
    // the civil calendar from a day count, after Howard Hinnant's algorithm
    let shifted = days as i64 + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_count_becomes_its_date() {
        assert_eq!(date_of(0), "1970-01-01");
        assert_eq!(date_of(19_782), "2024-02-29");
        assert_eq!(date_of(20_770), "2026-11-13");
    }
}
