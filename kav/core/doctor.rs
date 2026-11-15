//! kav doctor: a look at the health of the system, one line for each thing checked.

use crate::system::read;
use std::path::Path;
use std::process::Command;

/// The files under a folder whose name ends in one of the endings; folders that cannot be
/// read are passed over.
fn files_ending(folder: &Path, endings: &[&str], found: &mut Vec<String>) {
    for entry in std::fs::read_dir(folder).into_iter().flatten().filter_map(Result::ok) {
        let path = entry.path();
        let kind = entry.file_type();
        if kind.as_ref().is_ok_and(|kind| kind.is_dir()) {
            files_ending(&path, endings, found);
        } else if endings.iter().any(|ending| path.to_string_lossy().ends_with(ending)) {
            found.push(path.display().to_string());
        }
    }
}

fn lines(text: Option<String>) -> Vec<String> {
    text.unwrap_or_default().lines().map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect()
}

/// Each thing checked: what it is, and what is wrong with it, if anything.
pub fn findings() -> Vec<(&'static str, Vec<String>)> {
    let mut all = Vec::new();

    let failed = lines(read(Command::new("systemctl").args(["--failed", "--no-legend", "--plain"])));
    all.push(("no service has failed", failed.iter().map(|line| line.split_whitespace().next().unwrap_or("").to_string()).collect()));

    // pacman ends with an error when there are none, which reads as nothing here
    all.push(("no package is left that nothing needs", lines(read(Command::new("pacman").arg("-Qdtq")))));

    let mut left = Vec::new();
    files_ending(Path::new("/etc"), &[".pacnew", ".pacsave"], &mut left);
    left.sort();
    all.push(("no settings file waits to be merged", left));

    let used = lines(read(Command::new("df").args(["--output=pcent", "/"])));
    let percent = used.last().and_then(|line| line.trim_end_matches('%').trim().parse::<u32>().ok());
    all.push(("the disk has room", match percent {
        Some(percent) if percent >= 90 => vec![format!("the system's disk is {percent}% full")],
        Some(_) => vec![],
        None => vec!["how full the disk is could not be read".into()],
    }));

    let mut snapshots = Vec::new();
    if !Path::new("/etc/snapper/configs/root").exists() {
        snapshots.push("snapper has no settings for the system: nothing takes snapshots".to_string());
    }
    if read(Command::new("pacman").args(["-Q", "snap-pac"])).is_none() {
        snapshots.push("snap-pac is not installed: a change of packages takes no snapshot".to_string());
    }
    if read(Command::new("systemctl").args(["is-active", "grub-btrfsd"])).is_none() {
        snapshots.push("grub-btrfsd is not running: new snapshots do not reach the boot menu".to_string());
    }
    all.push(("every change of packages takes a snapshot, and the boot menu lists them", snapshots));

    let locked = Path::new("/var/lib/pacman/db.lck").exists() && read(Command::new("pgrep").args(["-x", "pacman"])).is_none();
    all.push(("pacman's database is not left locked", if locked { vec!["/var/lib/pacman/db.lck is there and pacman is not running".into()] } else { vec![] }));

    all
}

/// What mends each of the checks of findings(), in their order: the word after `kav fix`.
/// A check with none has nothing a command can do for it.
pub const FIXES: [Option<&str>; 6] = [Some("service"), Some("orphans"), Some("merge"), Some("cache"), None, Some("lock")];

/// Mends what one of the checks found. Each is the plain command a person would run, in
/// the open.
pub fn fix(arguments: &[String]) -> Result<(), String> {
    use crate::system::{as_root, run};
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["orphans"] => {
            let orphans = lines(read(Command::new("pacman").arg("-Qdtq")));
            if orphans.is_empty() {
                println!("kav: no package is left that nothing needs.");
                return Ok(());
            }
            run(as_root("pacman").args(["-Rns", "--"]).args(&orphans))
        }
        ["lock"] => {
            if read(Command::new("pgrep").args(["-x", "pacman"])).is_some() {
                return Err("pacman is running: its lock is in use".into());
            }
            run(as_root("rm").args(["-f", "/var/lib/pacman/db.lck"]))
        }
        // pacman's own cleaning: it asks before it deletes the packages that are not installed
        ["cache"] => run(as_root("pacman").arg("-Sc")),
        ["service", unit] if !unit.is_empty() && unit.chars().all(|c| c.is_ascii_alphanumeric() || "-_.@:\\".contains(c)) => {
            let _ = Command::new("systemctl").args(["status", "--no-pager", "--lines", "15", unit]).status();
            println!("\nkav: starting {unit} again.");
            run(as_root("systemctl").args(["restart", unit]))
        }
        ["merge", file] if file.starts_with("/etc/") && !file.contains("..") && (file.ends_with(".pacnew") || file.ends_with(".pacsave")) => {
            let (yours, _) = file.rsplit_once('.').unwrap_or((file, ""));
            println!("kav: {yours} is yours, on the left; {file} is the package's, on the right.\nChange yours as you see fit, save and leave the editor.");
            run(as_root("nvim").args(["-d", yours, file]))?;
            print!("Delete {file}, now that it is merged? [y/N] ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            let mut answer = String::new();
            let _ = std::io::stdin().read_line(&mut answer);
            if answer.trim().eq_ignore_ascii_case("y") {
                run(as_root("rm").args(["--", file]))?;
            }
            Ok(())
        }
        _ => Err("kav fix takes: orphans, lock, cache, service <unit>, or merge <file ending in .pacnew or .pacsave>".into()),
    }
}

pub fn check() -> Result<(), String> {
    let mut wrong = 0;
    for (what, problems) in findings() {
        if problems.is_empty() {
            println!("ok    {what}");
        } else {
            wrong += 1;
            println!("FAIL  {what}");
            for problem in problems {
                println!("      {problem}");
            }
        }
    }
    if wrong == 0 { Ok(()) } else { Err(format!("{wrong} of the checks found something")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_files_left_by_pacman_are_found_wherever_they_are() {
        let root = std::env::temp_dir().join(format!("kav-test-{}-doctor", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("deep/er")).unwrap();
        for file in ["pacman.conf", "pacman.conf.pacnew", "deep/er/x.pacsave"] {
            std::fs::write(root.join(file), "").unwrap();
        }
        let mut found = Vec::new();
        files_ending(&root, &[".pacnew", ".pacsave"], &mut found);
        found.sort();
        assert_eq!(found.len(), 2);
        assert!(found[0].ends_with("deep/er/x.pacsave") && found[1].ends_with("pacman.conf.pacnew"), "{found:?}");
    }
}
