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

    /// kav's own settings on this machine: the few that no other tool has a file for.
    pub fn settings(&self) -> PathBuf {
        self.lists[1].with_file_name("kav.toml")
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
    // sudo's own prompt does not say what the password is for
    command.args(["-p", "Your password, %u, to change the system: "]).arg(program);
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

/// Runs a command in the open, as run does, and also gives what it wrote to its errors:
/// that is where pacman says what stopped it. Its questions go there too, so each piece is
/// passed on as it comes.
pub fn run_heard(command: &mut Command) -> (Result<(), String>, String) {
    use std::io::{Read, Write};
    let name = command.get_program().to_string_lossy().into_owned();
    let mut child = match command.stderr(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(error) => return (Err(format!("{name}: {error}")), String::new()),
    };
    let mut heard = Vec::new();
    if let Some(mut errors) = child.stderr.take() {
        let mut piece = [0u8; 4096];
        while let Ok(count) = errors.read(&mut piece) {
            if count == 0 {
                break;
            }
            let _ = std::io::stderr().write_all(&piece[..count]);
            let _ = std::io::stderr().flush();
            heard.extend_from_slice(&piece[..count]);
        }
    }
    let ended = match child.wait() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{name} ended with {}", status.code().map_or("a signal".to_string(), |code| format!("status {code}")))),
        Err(error) => Err(format!("{name}: {error}")),
    };
    (ended, String::from_utf8_lossy(&heard).into_owned())
}

/// Whether a program is installed where commands are looked for.
pub fn has(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|folders| std::env::split_paths(&folders).any(|folder| folder.join(program).is_file()))
}

/// Writes a file that is root's, whole, making its folder when it is not there.
pub fn write_as_root(file: &str, text: &str) -> Result<(), String> {
    use std::io::Write;
    let mut child = as_root("install").args(["-D", "-m", "644", "/dev/stdin", file]).stdin(Stdio::piped()).spawn().map_err(|error| format!("{file}: {error}"))?;
    if let Some(mut input) = child.stdin.take() {
        input.write_all(text.as_bytes()).map_err(|error| format!("{file}: {error}"))?;
    }
    match child.wait() {
        Ok(status) if status.success() => Ok(()),
        _ => Err(format!("{file} could not be written")),
    }
}

/// Runs kav itself as root, for a step that writes what is root's.
pub fn myself_as_root(arguments: &[&str]) -> Result<(), String> {
    let program = std::env::current_exe().map_err(|error| error.to_string())?;
    run(as_root(&program.to_string_lossy()).args(arguments))
}

/// Whether a person is there to be asked.
pub fn someone_there() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

/// Asks a question that takes y for yes; anything else is no.
pub fn yes(question: &str) -> bool {
    use std::io::Write;
    print!("{question} [y/N] ");
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    let _ = std::io::stdin().read_line(&mut answer);
    answer.trim().eq_ignore_ascii_case("y")
}

/// Now, as 20261113-101500, in UTC: a name for what is kept from one moment.
pub fn stamp() -> String {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs());
    let (day, rest) = (date_of(seconds / 86400).replace('-', ""), seconds % 86400);
    format!("{day}-{:02}{:02}{:02}", rest / 3600, rest % 3600 / 60, rest % 60)
}

/// A size as people read it: 3.4 MiB.
pub fn size(bytes: u64) -> String {
    let mut amount = bytes as f64;
    for unit in ["B", "KiB", "MiB", "GiB"] {
        if amount < 1024.0 || unit == "GiB" {
            return if unit == "B" { format!("{amount:.0} {unit}") } else { format!("{amount:.1} {unit}") };
        }
        amount /= 1024.0;
    }
    unreachable!()
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

    #[test]
    fn a_size_is_written_as_people_read_it() {
        assert_eq!(size(512), "512 B");
        assert_eq!(size(1536), "1.5 KiB");
        assert_eq!(size(3 * 1024 * 1024), "3.0 MiB");
        assert_eq!(size(5 * 1024 * 1024 * 1024), "5.0 GiB");
    }
}
