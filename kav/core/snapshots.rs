//! The snapshots of the system: which packages each one changed, how one differs from the
//! system as it is now, deleting one and keeping one from being cleaned away. They are
//! snapper's, and root's to read; pacman's log says what each change did.

use crate::log::Transaction;
use crate::system::{as_root, read, run, someone_there, yes};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub number: u32,
    /// pre, post or single
    pub kind: String,
    /// for a post: the snapshot from before the same change
    pub pre: Option<u32>,
    /// as 2026-11-13 10:15:00
    pub date: String,
    /// what cleans it away when there are too many; nothing when it is pinned
    pub cleanup: String,
    pub description: String,
    /// what the change it belongs to did to the packages, when pacman's log has it
    pub changed: String,
    /// the same, package by package
    pub packages: Vec<String>,
}

impl Snapshot {
    pub fn pinned(&self) -> bool {
        self.cleanup.is_empty()
    }
}

/// The snapshots in what snapper lists, as number,type,pre-number,date,cleanup,description
/// with commas between; the oldest first. Number 0 is the system itself and is left out.
pub fn parse(listed: &str) -> Vec<Snapshot> {
    listed
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(6, ',');
            let (number, kind, pre, date, cleanup, description) = (fields.next()?, fields.next()?, fields.next()?, fields.next()?, fields.next()?, fields.next()?);
            // a description with a comma in it comes between quotes
            let description = match description.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')) {
                Some(inside) => inside.replace("\"\"", "\""),
                None => description.to_string(),
            };
            let number = number.trim().parse().ok().filter(|number| *number != 0)?;
            Some(Snapshot { number, kind: kind.to_string(), pre: pre.parse().ok(), date: date.to_string(), cleanup: cleanup.to_string(), description, changed: String::new(), packages: Vec::new() })
        })
        .collect()
}

/// Gives each snapshot what its change did: the change in pacman's log that started
/// between the snapshot from before it and the one from after.
pub fn with_changes(mut snapshots: Vec<Snapshot>, transactions: &[Transaction]) -> Vec<Snapshot> {
    let pairs: Vec<(u32, u32)> = snapshots.iter().filter_map(|snapshot| Some((snapshot.pre?, snapshot.number))).collect();
    for (before, after) in pairs {
        let date_of = |number: u32| snapshots.iter().find(|snapshot| snapshot.number == number).map(|snapshot| snapshot.date.clone());
        let (Some(from), Some(to)) = (date_of(before), date_of(after)) else { continue };
        let Some(change) = transactions.iter().find(|transaction| transaction.when >= from && transaction.when <= to) else { continue };
        for snapshot in snapshots.iter_mut().filter(|snapshot| snapshot.number == before || snapshot.number == after) {
            snapshot.changed = change.in_short();
            snapshot.packages = change.changes.iter().map(|change| format!("{:<11} {}  {}", change.verb, change.name, change.versions)).collect();
        }
    }
    snapshots
}

/// The snapshots there are, the oldest first; None when snapper could not be asked.
pub fn list() -> Option<Vec<Snapshot>> {
    let listed = read(as_root("snapper").args(["--no-dbus", "--iso", "-c", "root", "--csvout", "--no-headers", "list", "--columns", "number,type,pre-number,date,cleanup,description"]))?;
    Some(with_changes(parse(&listed), &crate::log::transactions(&crate::log::text())))
}

/// One snapshot in a line.
pub fn line(snapshot: &Snapshot) -> String {
    let what = match (snapshot.kind.as_str(), snapshot.changed.is_empty()) {
        ("pre", false) => format!("before: {}", snapshot.changed),
        ("post", false) => format!("after: {}", snapshot.changed),
        _ => snapshot.description.clone(),
    };
    format!("{:>4}  {:<6} {}  {what}{}", snapshot.number, snapshot.kind, snapshot.date.chars().take(16).collect::<String>(), if snapshot.pinned() { "  · pinned" } else { "" })
}

/// How a package differs between a snapshot and the system now.
#[derive(Debug, PartialEq)]
pub struct Difference {
    /// installed, removed, upgraded or downgraded: what happened since the snapshot
    pub what: &'static str,
    pub name: String,
    pub versions: String,
}

/// The packages whose version differs between two answers of pacman -Q: what was
/// installed then, and what is now.
pub fn differences(then: &str, now: &str) -> Vec<Difference> {
    let versions = |text: &str| -> BTreeMap<String, String> { text.lines().filter_map(|line| line.split_once(' ')).map(|(name, version)| (name.to_string(), version.trim().to_string())).collect() };
    let (then, now) = (versions(then), versions(now));
    let mut all = Vec::new();
    for (name, old) in &then {
        match now.get(name) {
            None => all.push(Difference { what: "removed", name: name.clone(), versions: old.clone() }),
            Some(new) if new != old => {
                let what = if crate::version::compare(new, old) == Ordering::Less { "downgraded" } else { "upgraded" };
                all.push(Difference { what, name: name.clone(), versions: format!("{old} -> {new}") });
            }
            Some(_) => {}
        }
    }
    for (name, new) in now.iter().filter(|(name, _)| !then.contains_key(*name)) {
        all.push(Difference { what: "installed", name: name.clone(), versions: new.clone() });
    }
    all.sort_by(|one, two| (one.what, &one.name).cmp(&(two.what, &two.name)));
    all
}

fn number_of(text: &str) -> Result<u32, String> {
    text.parse().ok().filter(|number| *number != 0).ok_or_else(|| format!("{text} is not the number of a snapshot: kav snapshots lists them"))
}

fn find(number: u32) -> Result<Snapshot, String> {
    let all = list().ok_or("the snapshots could not be listed")?;
    all.into_iter().find(|snapshot| snapshot.number == number).ok_or_else(|| format!("there is no snapshot {number}: kav snapshots lists them"))
}

/// `kav snapshots`, and what comes after it.
pub fn command(arguments: &[String]) -> Result<(), String> {
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => {
            let all = list().ok_or("the snapshots could not be listed")?;
            if all.is_empty() {
                println!("There are no snapshots yet: each change of packages takes one before and one after.");
            }
            for snapshot in &all {
                println!("{}", line(snapshot));
            }
            Ok(())
        }
        ["compare", number] => {
            let snapshot = find(number_of(number)?)?;
            let then = read(as_root("pacman").args(["-Q", "--dbpath", &format!("/.snapshots/{}/snapshot/var/lib/pacman", snapshot.number)]))
                .ok_or_else(|| format!("the packages of snapshot {} could not be read", snapshot.number))?;
            let now = read(Command::new("pacman").arg("-Q")).ok_or("the packages installed now could not be read")?;
            let all = differences(&then, &now);
            println!("Since snapshot {}, of {}:", snapshot.number, snapshot.date);
            for difference in &all {
                println!("  {:<11} {}  {}", difference.what, difference.name, difference.versions);
            }
            match all.len() {
                0 => println!("  no package differs: the system has the packages it had then."),
                1 => println!("1 package differs. kavat-rollback {} takes the system back to it.", snapshot.number),
                count => println!("{count} packages differ. kavat-rollback {} takes the system back to it.", snapshot.number),
            }
            Ok(())
        }
        ["delete", number] | ["delete", number, "--yes"] => {
            let snapshot = find(number_of(number)?)?;
            println!("{}", line(&snapshot));
            if arguments.len() < 3 {
                if !someone_there() {
                    return Err(format!("to delete it without being asked: kav snapshots delete {number} --yes"));
                }
                if !yes("Delete this snapshot? The system cannot go back to it after.") {
                    println!("kav: nothing was deleted.");
                    return Ok(());
                }
            }
            run(as_root("snapper").args(["--no-dbus", "-c", "root", "delete", &snapshot.number.to_string()]))?;
            println!("kav: snapshot {} is deleted.", snapshot.number);
            Ok(())
        }
        [word @ ("pin" | "unpin"), number] => {
            let snapshot = find(number_of(number)?)?;
            // snapper cleans away the snapshots that name a way of cleaning; one that names none stays
            let cleanup = if *word == "pin" { "" } else { "number" };
            run(as_root("snapper").args(["--no-dbus", "-c", "root", "modify", "--cleanup-algorithm", cleanup, &snapshot.number.to_string()]))?;
            match *word {
                "pin" => println!("kav: snapshot {} is pinned: it stays until it is deleted by hand, or unpinned.", snapshot.number),
                _ => println!("kav: snapshot {} is not pinned any more: it goes when there are too many.", snapshot.number),
            }
            Ok(())
        }
        _ => Err("kav snapshots takes nothing, or: compare <number>, delete <number> [--yes], pin <number>, unpin <number>".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTED: &str = "\
0,single,,,,current
1,single,,2026-11-12 20:00:00,,the system as it was installed
2,pre,,2026-11-13 10:15:01,number,pacman -S --needed -- os-prober
3,post,2,2026-11-13 10:15:04,number,os-prober
4,pre,,2026-11-14 09:00:09,number,pacman -Syu
5,post,4,2026-11-14 09:00:16,number,\"glibc kitty linux os-prober zsh, and \"\"more\"\"\"
6,pre,,2026-11-14 12:00:00,number,pacman -S nothing
";

    #[test]
    fn what_snapper_lists_becomes_the_snapshots() {
        let all = parse(LISTED);
        assert_eq!(all.len(), 6);
        assert_eq!((all[0].number, all[0].pinned(), all[0].description.as_str()), (1, true, "the system as it was installed"));
        assert_eq!((all[2].kind.as_str(), all[2].pre, all[2].pinned()), ("post", Some(2), false));
        assert_eq!(all[4].description, "glibc kitty linux os-prober zsh, and \"more\"");
    }

    #[test]
    fn each_snapshot_knows_what_its_change_did() {
        let all = with_changes(parse(LISTED), &crate::log::transactions(crate::log::SAMPLE));
        assert_eq!(all[1].changed, "installed os-prober");
        assert_eq!(all[2].changed, "installed os-prober");
        assert_eq!(all[3].changed, "upgraded 4: glibc, linux +2 more; removed os-prober");
        // a change that never ended has no snapshot from after it, and nothing is guessed
        assert_eq!(all[5].changed, "");
        assert_eq!(all[1].packages, ["installed   os-prober  1.83-1"]);
        assert_eq!(all[3].packages.len(), 5);
        assert_eq!(line(&all[1]), "   2  pre    2026-11-13 10:15  before: installed os-prober");
        assert_eq!(line(&all[0]), "   1  single 2026-11-12 20:00  the system as it was installed  · pinned");
    }

    #[test]
    fn a_snapshot_is_compared_with_the_system_by_the_versions_of_its_packages() {
        let then = "glibc 2.42-1\nkitty 0.44.0-1\nos-prober 1.83-1\nzsh 5.9-5\n";
        let now = "glibc 2.42-2\nkitty 0.43.1-1\nyazi 25.5-1\nzsh 5.9-5\n";
        let all = differences(then, now);
        let told: Vec<String> = all.iter().map(|difference| format!("{} {} {}", difference.what, difference.name, difference.versions)).collect();
        assert_eq!(told, ["downgraded kitty 0.44.0-1 -> 0.43.1-1", "installed yazi 25.5-1", "removed os-prober 1.83-1", "upgraded glibc 2.42-1 -> 2.42-2"]);
        assert!(differences(now, now).is_empty());
    }
}
