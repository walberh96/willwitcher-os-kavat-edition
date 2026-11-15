//! Two guards around a change of packages, so that it is not left halfway.
//!
//! A file that no package owns, in the place where a package wants to put one of its own,
//! stops pacman before it changes anything. kav moves such files aside and tries once more.
//!
//! Hyprland reads its configuration again whenever one of its files changes, and pacman
//! changes them one at a time: in between, Hyprland would read a configuration that is
//! half the old one and half the new. kav has it wait, and read once at the end.

use crate::system::{as_root, is_root, myself_as_root, read, run, run_heard, stamp};
use std::path::Path;
use std::process::{Command, Stdio};

/// Where the files moved out of a package's way are kept, each under the moment it was moved.
const ASIDE: &str = "/var/lib/kav/aside";

/// The files that stopped pacman because they are already there, in what it wrote. One
/// that pacman says another package owns is not among them: that is for a person to settle.
pub fn in_the_way(heard: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in heard.lines() {
        let Some((_, rest)) = line.split_once(": ") else { continue };
        let file = rest.strip_suffix(" exists in filesystem").filter(|file| file.starts_with('/') && !file.contains("/../"));
        if let Some(file) = file.filter(|file| !files.iter().any(|other| other == file)) {
            files.push(file.to_string());
        }
    }
    files
}

/// Which of the files in the way may be moved: all of them, when no package owns any.
/// One that is owned stops the whole: pacman was right to stop.
fn movable(files: &[String], owned: &dyn Fn(&str) -> bool) -> Result<Vec<String>, String> {
    match files.iter().find(|file| owned(file)) {
        Some(file) => Err(format!("{file} is in the way, and it belongs to an installed package: nothing was moved")),
        None => Ok(files.to_vec()),
    }
}

fn owned(file: &str) -> bool {
    read(Command::new("pacman").args(["-Qqo", "--", file])).is_some()
}

/// Where a file goes when it is moved aside at a moment: its whole path, under the moment.
fn place(moment: &str, file: &str) -> String {
    format!("{ASIDE}/{moment}{file}")
}

/// The steps that run as root: `kav aside move <moment> <file>…` moves the files out of
/// the way, and `kav aside back <moment> <file>…` puts back the ones whose place is still
/// empty, which is what the package did not claim after all.
pub fn aside(arguments: &[String]) -> Result<(), String> {
    let [word, moment, files @ ..] = arguments else {
        return Err("aside is a step of kav's own, which gives it what it needs".into());
    };
    if !is_root() || !moment.chars().all(|c| c.is_ascii_digit() || c == '-') || !files.iter().all(|file| file.starts_with('/') && !file.contains("/../")) {
        return Err("aside is a step of kav's own, run as root".into());
    }
    for file in files {
        let kept = place(moment, file);
        let (from, to) = match word.as_str() {
            "move" => (file.clone(), kept),
            "back" if Path::new(&kept).exists() && std::fs::symlink_metadata(file).is_err() => (kept, file.clone()),
            _ => continue,
        };
        if let Some(folder) = Path::new(&to).parent() {
            std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
        }
        run(Command::new("mv").args(["--", &from, &to]))?;
    }
    Ok(())
}

/// Has Hyprland stop reading its configuration by itself, or start again. This is the one
/// place that knows how Hyprland is asked. False when it could not be asked: there is no
/// desktop to guard then.
fn hyprland_reads_by_itself(on: bool) -> bool {
    let ask = format!("hl.config({{ misc = {{ disable_autoreload = {} }} }})", !on);
    let quiet = |arguments: &[&str]| Command::new("hyprctl").args(arguments).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|status| status.success());
    quiet(&["eval", &ask]) || quiet(&["--instance", "0", "eval", &ask])
}

/// Whether Hyprland is set not to read its configuration by itself: then it is the
/// person's choice, and kav leaves it alone.
fn hyprland_already_waits() -> bool {
    let answer = |arguments: &[&str]| read(Command::new("hyprctl").args(arguments));
    let said = answer(&["getoption", "misc:disable_autoreload"]).or_else(|| answer(&["--instance", "0", "getoption", "misc:disable_autoreload"]));
    said.is_some_and(|said| said.lines().next().is_some_and(|line| line.trim().ends_with(" 1") || line.trim().ends_with("true")))
}

/// Runs pacman for a change of packages, with the two guards around it.
pub fn pacman(arguments: &[&str]) -> Result<(), String> {
    let waiting = !hyprland_already_waits() && hyprland_reads_by_itself(false);
    let ended = change(arguments);
    if waiting {
        hyprland_reads_by_itself(true);
        // once, now that every file is whole
        let quiet = |arguments: &[&str]| Command::new("hyprctl").args(arguments).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|status| status.success());
        let _ = quiet(&["reload"]) || quiet(&["--instance", "0", "reload"]);
    }
    ended
}

fn change(arguments: &[&str]) -> Result<(), String> {
    // pacman's words are looked for as it writes them in English
    let (ended, heard) = run_heard(as_root("pacman").env("LC_MESSAGES", "C").args(arguments));
    let files = in_the_way(&heard);
    if ended.is_ok() || files.is_empty() {
        return ended;
    }
    let files = movable(&files, &owned)?;
    let moment = stamp();
    println!("\nkav: {} that no package owns {} in the way. Moving {} aside, to {ASIDE}/{moment}/, and trying once more:", plural(files.len(), "file"), if files.len() == 1 { "is" } else { "are" }, if files.len() == 1 { "it" } else { "them" });
    for file in &files {
        println!("  {file}");
    }
    let step = |word: &str| {
        let mut all = vec!["aside", word, moment.as_str()];
        all.extend(files.iter().map(String::as_str));
        myself_as_root(&all)
    };
    step("move")?;
    let again = run(as_root("pacman").args(arguments));
    // what the package did not put a file of its own in place of goes back where it was
    step("back")?;
    match again {
        Ok(()) => println!("\nkav: the change went through. What was in the way is kept in {ASIDE}/{moment}/: delete it when nothing in it is missed."),
        Err(_) => println!("\nkav: the change did not go through the second time either. What was moved is back where it was."),
    }
    again
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

#[cfg(test)]
mod tests {
    use super::*;

    const STOPPED: &str = "\
resolving dependencies...
looking for conflicting packages...
:: Proceed with installation? [Y/n] error: failed to commit transaction (conflicting files)
kavat-made-up: /usr/share/made-up/settings exists in filesystem
kavat-made-up: /usr/bin/made-up exists in filesystem
kavat-made-up: /usr/bin/made-up exists in filesystem
other: /usr/bin/vi exists in filesystem (owned by vi)
other: relative/path exists in filesystem
Errors occurred, no packages were upgraded.
";

    #[test]
    fn the_files_in_the_way_are_read_from_what_pacman_wrote() {
        assert_eq!(in_the_way(STOPPED), ["/usr/share/made-up/settings", "/usr/bin/made-up"]);
        assert!(in_the_way("error: target not found: nothing\n").is_empty());
    }

    #[test]
    fn a_file_that_a_package_owns_is_never_moved() {
        let files = in_the_way(STOPPED);
        assert_eq!(movable(&files, &|_| false).unwrap(), files);
        let problem = movable(&files, &|file| file == "/usr/bin/made-up").unwrap_err();
        assert!(problem.starts_with("/usr/bin/made-up is in the way, and it belongs to an installed package"), "{problem}");
    }

    #[test]
    fn a_file_moved_aside_keeps_its_whole_path() {
        assert_eq!(place("20261115-101500", "/usr/bin/made-up"), "/var/lib/kav/aside/20261115-101500/usr/bin/made-up");
    }
}
