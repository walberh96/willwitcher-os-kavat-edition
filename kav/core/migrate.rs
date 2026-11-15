//! Migrations: what a new version of the system has to change that pacman does not manage,
//! like a settings file of the person's or a service of theirs. Each is a script that runs
//! once, after pacman and in the order of its number. One that fails stops the ones after it.
//!
//! The system's are in /usr/share/kavat/migrations and run as root; what ran is kept in
//! /var/lib/kavat/migrations, inside the system, so going back to a snapshot also goes back
//! on which of them ran. The person's are in the folder user/ there and run as the person;
//! what ran is kept in ~/.local/state/kavat/migrations.

use crate::system::{is_root, myself_as_root, run};
use std::path::{Path, PathBuf};
use std::process::Command;

const SCRIPTS: &str = "/usr/share/kavat/migrations";
const RAN: &str = "/var/lib/kavat/migrations";

/// The two queues: the system's and the person's. Each is where its scripts are and where
/// it keeps which of them ran.
fn queues() -> [(&'static str, PathBuf, PathBuf); 2] {
    let variable = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from);
    let scripts = variable("KAV_MIGRATIONS").unwrap_or_else(|| SCRIPTS.into());
    let state = variable("XDG_STATE_HOME").unwrap_or_else(|| variable("HOME").unwrap_or_else(|| "/root".into()).join(".local/state"));
    [("system", scripts.clone(), RAN.into()), ("user", scripts.join("user"), state.join("kavat/migrations"))]
}

/// Whether a file is a migration: four digits, a dash, a name, and .sh.
fn is_migration(name: &str) -> bool {
    let number = name.len() > 8 && name.as_bytes()[..4].iter().all(u8::is_ascii_digit) && name.as_bytes()[4] == b'-';
    number && name.ends_with(".sh") && name.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

/// The migrations in a folder, in the order they run.
fn scripts_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder).into_iter().flatten().filter_map(Result::ok).map(|entry| entry.file_name().to_string_lossy().into_owned()).filter(|name| is_migration(name)).collect();
    names.sort();
    names
}

/// The migrations that have not run, in order. A system, or a person, that is new has no
/// record of any: everything there is then is older than they are, and counts as run.
fn waiting(scripts: &Path, ran: &Path) -> Vec<String> {
    if !ran.is_dir() {
        return Vec::new();
    }
    scripts_in(scripts).into_iter().filter(|name| !ran.join(name).exists()).collect()
}

/// Starts the record of what ran, when there is none: with everything there is now.
fn begin(scripts: &Path, ran: &Path) -> Result<(), String> {
    if ran.is_dir() {
        return Ok(());
    }
    let problem = |error: std::io::Error| format!("{}: {error}", ran.display());
    std::fs::create_dir_all(ran).map_err(problem)?;
    for name in scripts_in(scripts) {
        std::fs::write(ran.join(name), "").map_err(problem)?;
    }
    Ok(())
}

/// Runs what waits in one queue, in order, as whoever kav runs as. The first that fails
/// stops the rest, and is named.
fn run_queue(scripts: &Path, ran: &Path) -> Result<usize, String> {
    begin(scripts, ran)?;
    let all = waiting(scripts, ran);
    for (at, name) in all.iter().enumerate() {
        println!("kav: migration {name}");
        if let Err(problem) = run(Command::new("bash").arg(scripts.join(name))) {
            let after = all.len() - at - 1;
            let rest = if after == 0 { String::new() } else { format!(" The {after} after it wait for it.") };
            return Err(format!("the migration {name} failed: {problem}.{rest}\n     It is {}; `kav migrate` tries it again.", scripts.join(name).display()));
        }
        std::fs::write(ran.join(name), "").map_err(|error| format!("{}: {error}", ran.join(name).display()))?;
    }
    Ok(all.len())
}

/// `kav migrate`: runs what waits, the system's as root and then the person's.
/// `kav migrate --list` shows them all, and which ran.
pub fn command(arguments: &[String]) -> Result<(), String> {
    let [(_, system, system_ran), (_, user, user_ran)] = queues();
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["--list"] => {
            for (whose, scripts, ran) in queues() {
                let waiting = waiting(&scripts, &ran);
                for name in scripts_in(&scripts) {
                    let state = match (ran.is_dir(), waiting.contains(&name)) {
                        (false, _) => "older than this system",
                        (true, true) => "waits",
                        (true, false) => "ran",
                    };
                    println!("{whose:<7} {name:<40} {state}");
                }
            }
            if scripts_in(&system).is_empty() && scripts_in(&user).is_empty() {
                println!("There are no migrations: no version of the system has needed one yet.");
            }
            Ok(())
        }
        // the step that runs as root
        ["--system"] => {
            if !is_root() {
                return Err("the system's migrations run as root: kav migrate does it".into());
            }
            run_queue(&system, &system_ran).map(|_| ())
        }
        [] => {
            // root is asked for only when the system has something waiting, or no record yet
            let mut count = waiting(&system, &system_ran).len();
            if is_root() {
                run_queue(&system, &system_ran)?;
            } else if count > 0 || !system_ran.is_dir() {
                myself_as_root(&["migrate", "--system"])?;
            }
            // the person's are for someone with a desktop: root has none
            if !is_root() {
                count += run_queue(&user, &user_ran)?;
            }
            if count == 0 {
                println!("kav: no migration waits.");
            }
            Ok(())
        }
        _ => Err("kav migrate takes nothing, or --list".into()),
    }
}

/// After pacman, in kav update: the same as kav migrate, and quiet when nothing waits.
pub fn after_update() -> Result<(), String> {
    let [(_, system, system_ran), (_, user, user_ran)] = queues();
    if is_root() {
        run_queue(&system, &system_ran)?;
    } else {
        if !waiting(&system, &system_ran).is_empty() || !system_ran.is_dir() {
            myself_as_root(&["migrate", "--system"])?;
        }
        run_queue(&user, &user_ran)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("kav-test-{}-migrate-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        (root.join("scripts"), root.join("ran"))
    }

    #[test]
    fn only_numbered_scripts_are_migrations() {
        assert!(is_migration("0001-move-the-settings.sh"));
        assert!(!is_migration("README.md") && !is_migration("1-short.sh") && !is_migration("0001-a b.sh") && !is_migration("0001-.sh.bak"));
    }

    #[test]
    fn what_is_there_before_the_first_time_counts_as_run() {
        let (scripts, ran) = world("first");
        std::fs::write(scripts.join("0001-old.sh"), "exit 1\n").unwrap();
        assert_eq!(run_queue(&scripts, &ran).unwrap(), 0);
        assert!(ran.join("0001-old.sh").exists());
    }

    #[test]
    fn each_runs_once_and_in_order_and_a_failure_stops_the_rest() {
        let (scripts, ran) = world("order");
        assert_eq!(run_queue(&scripts, &ran).unwrap(), 0);
        let out = scripts.join("out");
        let script = |name: &str, ends: u8| std::fs::write(scripts.join(name), format!("echo {name} >> {}\nexit {ends}\n", out.display())).unwrap();
        script("0002-second.sh", 0);
        script("0001-first.sh", 0);
        script("notes.sh", 0);
        assert_eq!(waiting(&scripts, &ran), ["0001-first.sh", "0002-second.sh"]);
        assert_eq!(run_queue(&scripts, &ran).unwrap(), 2);
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "0001-first.sh\n0002-second.sh\n");
        assert_eq!(run_queue(&scripts, &ran).unwrap(), 0);

        script("0003-breaks.sh", 1);
        script("0004-after.sh", 0);
        let problem = run_queue(&scripts, &ran).unwrap_err();
        assert!(problem.contains("the migration 0003-breaks.sh failed") && problem.contains("The 1 after it wait"), "{problem}");
        assert_eq!(waiting(&scripts, &ran), ["0003-breaks.sh", "0004-after.sh"]);
        // mended, it runs, and then the one that waited for it
        script("0003-breaks.sh", 0);
        assert_eq!(run_queue(&scripts, &ran).unwrap(), 2);
        assert!(std::fs::read_to_string(&out).unwrap().ends_with("0003-breaks.sh\n0003-breaks.sh\n0004-after.sh\n"));
    }
}
