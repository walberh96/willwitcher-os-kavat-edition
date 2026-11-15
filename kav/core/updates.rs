//! What an update would bring: each package with its versions, how much is downloaded,
//! how much the disk gains or loses, and whether the system asks to be restarted after.
//! Also the look for updates that a timer does, which fetches nothing but pacman's lists,
//! and into a folder of its own: the system's lists only move together with its packages.

use crate::allowlist::Allowlist;
use crate::config::Config;
use crate::system::{Paths, as_root, is_root, read, run, size};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// Where the look for updates keeps its copy of pacman's lists.
pub const CHECKED: &str = "/var/lib/kav/check";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Arch,
    Kavat,
    Aur,
}

impl Source {
    pub fn title(self) -> &'static str {
        match self {
            Source::Arch => "Arch's repositories",
            Source::Kavat => "[kavat]",
            Source::Aur => "The AUR, from the reviewed list",
        }
    }
}

/// One package that an update would change.
#[derive(Debug, PartialEq)]
pub struct Update {
    pub name: String,
    pub from: String,
    pub to: String,
    pub source: Source,
    /// what is downloaded, in bytes; nothing is known of a package of the AUR until it is built
    pub download: u64,
    /// what the disk gains, or loses, in bytes
    pub change: i64,
    pub restart: bool,
    /// a package of the AUR that kav update leaves as it is
    pub held: bool,
}

/// Whether the system asks to be restarted after this package changes: the kernel and what
/// it loads, what every program is linked against, what starts the system, the desktop
/// itself and what draws it.
pub fn asks_restart(name: &str) -> bool {
    let name = name.strip_prefix("lib32-").unwrap_or(name);
    let kernel = name == "linux" || ["linux-lts", "linux-zen", "linux-hardened", "linux-rt", "linux-firmware"].iter().any(|other| name.starts_with(other));
    let whole = ["amd-ucode", "intel-ucode", "glibc", "systemd", "systemd-libs", "dbus", "dbus-broker", "cryptsetup", "grub", "hyprland", "wayland", "mesa", "libglvnd", "xf86-video-intel"];
    let drivers = ["nvidia", "vulkan-radeon", "vulkan-intel", "vulkan-nouveau", "vulkan-swrast", "libva-mesa", "libva-intel", "intel-media-driver"];
    kernel && !name.ends_with("-headers") && !name.ends_with("-docs") || whole.contains(&name) || drivers.iter().any(|driver| name.starts_with(driver))
}

/// A size as pacman writes it, 12.34 MiB, in bytes.
fn bytes(text: &str) -> u64 {
    let mut words = text.split_whitespace();
    let amount: f64 = words.next().and_then(|amount| amount.parse().ok()).unwrap_or(0.0);
    let unit = match words.next().unwrap_or("B") {
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    (amount * unit) as u64
}

/// What pacman's records say of each package, by its name: each field by its title.
fn records(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut all = BTreeMap::new();
    for block in text.split("\n\n") {
        let fields: BTreeMap<String, String> =
            block.lines().filter(|line| !line.starts_with(' ')).filter_map(|line| line.split_once(" : ")).map(|(title, value)| (title.trim().to_string(), value.trim().to_string())).collect();
        if let Some(name) = fields.get("Name") {
            all.insert(name.clone(), fields);
        }
    }
    all
}

/// The updates of pacman's repositories: `pending` is what pacman -Qu says, `offered` its
/// records of the new versions and `installed` those of the ones in place.
fn from_repositories(pending: &str, offered: &str, installed: &str) -> Vec<Update> {
    let (offered, installed) = (records(offered), records(installed));
    let size_in = |records: &BTreeMap<String, BTreeMap<String, String>>, name: &str, title: &str| records.get(name).and_then(|fields| fields.get(title)).map_or(0, |text| bytes(text));
    let mut all = Vec::new();
    for line in pending.lines() {
        let mut words = line.split_whitespace();
        let (Some(name), Some(from), Some(_), Some(to)) = (words.next(), words.next(), words.next(), words.next()) else { continue };
        let kavat = offered.get(name).and_then(|fields| fields.get("Repository")).is_some_and(|repository| repository == "kavat");
        all.push(Update {
            name: name.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            source: if kavat { Source::Kavat } else { Source::Arch },
            download: size_in(&offered, name, "Download Size"),
            change: size_in(&offered, name, "Installed Size") as i64 - size_in(&installed, name, "Installed Size") as i64,
            restart: asks_restart(name),
            held: false,
        });
    }
    all
}

/// The newest of pacman's lists in a folder, by when the repository last changed.
fn newest_list(folder: &Path) -> Option<std::time::SystemTime> {
    let lists = std::fs::read_dir(folder.join("sync")).ok()?.filter_map(Result::ok).filter(|entry| entry.path().extension().is_some_and(|ending| ending == "db"));
    lists.filter_map(|entry| entry.metadata().ok()?.modified().ok()).max()
}

/// pacman, reading the lists that are newer: the system's own, or the ones the look for
/// updates fetched since.
fn pacman() -> Command {
    let mut command = Command::new("pacman");
    command.env("LC_ALL", "C");
    if newest_list(Path::new(CHECKED)) > newest_list(Path::new("/var/lib/pacman")) {
        command.args(["--dbpath", CHECKED]);
    }
    command
}

/// Everything an update would change, in the order it is shown: Arch's, [kavat]'s and the
/// AUR's, each by name.
pub fn pending(paths: &Paths) -> Vec<Update> {
    let waiting = read(pacman().arg("-Qu")).unwrap_or_default();
    let names: Vec<&str> = waiting.lines().filter_map(|line| line.split_whitespace().next()).collect();
    let mut all = if names.is_empty() {
        Vec::new()
    } else {
        let offered = read(pacman().arg("-Si").arg("--").args(&names)).unwrap_or_default();
        let installed = read(Command::new("pacman").env("LC_ALL", "C").arg("-Qi").arg("--").args(&names)).unwrap_or_default();
        from_repositories(&waiting, &offered, &installed)
    };
    // a package of the AUR changes when its reviewed commit is not the one it was built from
    let held = Config::load(&paths.settings()).map(|config| config.hold).unwrap_or_default();
    let foreign = read(Command::new("pacman").arg("-Qmq")).unwrap_or_default();
    if let Ok(list) = Allowlist::load(&paths.lists) {
        for name in foreign.lines() {
            let (Some(entry), built) = (list.get(name), crate::aur::built_commit(name)) else { continue };
            if built.as_deref() != Some(entry.commit.as_str()) {
                all.push(Update {
                    name: name.to_string(),
                    from: built.map_or("not built by kav".into(), |commit| commit.chars().take(12).collect()),
                    to: entry.commit[..12].to_string(),
                    source: Source::Aur,
                    download: 0,
                    change: 0,
                    restart: asks_restart(name),
                    held: held.iter().any(|other| other == name),
                });
            }
        }
    }
    all.sort_by(|a, b| (a.source as u8, &a.name).cmp(&(b.source as u8, &b.name)));
    all
}

/// A change on the disk, with its sign: +3.4 MiB.
pub fn signed(change: i64) -> String {
    format!("{}{}", if change < 0 { "-" } else { "+" }, size(change.unsigned_abs()))
}

/// The last line under the updates: what they come to.
pub fn total(all: &[Update]) -> String {
    let moving: Vec<&Update> = all.iter().filter(|update| !update.held).collect();
    if moving.is_empty() {
        return "Nothing waits to be brought up to date.".into();
    }
    let restart: Vec<&str> = moving.iter().filter(|update| update.restart).map(|update| update.name.as_str()).collect();
    format!(
        "{} packages · {} to download · {} on the disk · {}",
        moving.len(),
        size(moving.iter().map(|update| update.download).sum()),
        signed(moving.iter().map(|update| update.change).sum()),
        if restart.is_empty() { "no restart is asked for".to_string() } else { format!("a restart is asked for after, by {}", restart.join(", ")) }
    )
}

/// `kav update --list`: what an update would bring, and nothing is changed.
pub fn show(paths: &Paths) -> Result<(), String> {
    let all = pending(paths);
    for source in [Source::Arch, Source::Kavat, Source::Aur] {
        let these: Vec<&Update> = all.iter().filter(|update| update.source == source).collect();
        if these.is_empty() {
            continue;
        }
        println!("{}", source.title());
        for update in these {
            let sizes = if source == Source::Aur { "built here".to_string() } else { format!("{:>10} {:>11}", size(update.download), signed(update.change)) };
            let marks = format!("{}{}", if update.restart { "  restart" } else { "" }, if update.held { "  held back" } else { "" });
            println!("  {:<30} {} -> {}  {sizes}{marks}", update.name, update.from, update.to);
        }
    }
    println!("{}", total(&all));
    Ok(())
}

/// What a change of packages that just ended asks for: `before` is how long pacman's log
/// was when it started.
pub fn restart_after(before: usize) -> Vec<String> {
    let log = crate::log::text();
    let mut names: Vec<String> = crate::log::transactions(log.get(before..).unwrap_or(""))
        .into_iter()
        .flat_map(|transaction| transaction.changes)
        .filter(|change| change.verb != "removed" && asks_restart(&change.name))
        .map(|change| change.name)
        .collect();
    names.sort();
    names.dedup();
    names
}

/// `kav check`: looks for updates without changing the system. pacman's lists are fetched
/// into a folder of their own, beside a link to what is installed. With `notify`, as the
/// timer runs it, whoever is at the desktop is told when something waits.
pub fn check(notify: bool) -> Result<(), String> {
    if !is_root() {
        return crate::system::myself_as_root(&["check"]);
    }
    std::fs::create_dir_all(CHECKED).map_err(|error| format!("{CHECKED}: {error}"))?;
    let link = Path::new(CHECKED).join("local");
    if !link.exists() {
        std::os::unix::fs::symlink("/var/lib/pacman/local", &link).map_err(|error| format!("{}: {error}", link.display()))?;
    }
    let fetched = Command::new("pacman").args(["-Sy", "--dbpath", CHECKED, "--logfile", "/dev/null"]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    if !fetched.is_ok_and(|status| status.success()) {
        return Err("the repositories could not be reached: nothing is known of new updates".into());
    }
    let count = read(Command::new("pacman").args(["-Qu", "--dbpath", CHECKED])).unwrap_or_default().lines().count();
    match count {
        0 => println!("kav: the system is up to date."),
        count => println!("kav: {count} packages can be brought up to date. kav update --list shows them; kav update brings them."),
    }
    if notify && count > 0 {
        tell_the_desktop(&format!("{count} updates are waiting"), "kav update brings them, with a snapshot from before.");
    }
    Ok(())
}

/// A notification for each person with a desktop open, sent as that person.
fn tell_the_desktop(title: &str, text: &str) {
    for session in std::fs::read_dir("/run/user").into_iter().flatten().filter_map(Result::ok) {
        let bus = session.path().join("bus");
        let number = session.file_name().to_string_lossy().into_owned();
        let Some(user) = read(Command::new("id").args(["-nu", &number])) else { continue };
        if bus.exists() && number != "0" {
            let _ = run(Command::new("runuser")
                .args(["-u", user.trim(), "--", "env", &format!("DBUS_SESSION_BUS_ADDRESS=unix:path={}", bus.display())])
                .args(["notify-send", "--app-name=kav", "--icon=system-software-update", title, text]));
        }
    }
}

/// How often the timer looks for updates: daily, weekly or never. `masked` is whether the
/// timer is switched off in /etc/systemd/system, and `own` what kav wrote for it there.
pub fn frequency(masked: bool, own: Option<&str>) -> &'static str {
    match own {
        _ if masked => "never",
        Some(text) if text.lines().any(|line| line.trim() == "OnCalendar=daily") => "daily",
        _ => "weekly",
    }
}

const TIMER: &str = "kav-check.timer";
const TIMER_OFF: &str = "/etc/systemd/system/kav-check.timer";
const TIMER_OWN: &str = "/etc/systemd/system/kav-check.timer.d/kav.conf";

pub fn frequency_now() -> &'static str {
    frequency(std::fs::symlink_metadata(TIMER_OFF).is_ok(), std::fs::read_to_string(TIMER_OWN).ok().as_deref())
}

/// Sets how often the timer looks, in systemd's own files. As root.
pub fn set_frequency(how_often: &str) -> Result<(), String> {
    if how_often == "never" {
        return run(as_root("systemctl").args(["--quiet", "mask", "--now", TIMER]));
    }
    run(as_root("systemctl").args(["--quiet", "unmask", TIMER]))?;
    // the empty line takes away the timer's own time, so that this one is the only one
    crate::system::write_as_root(TIMER_OWN, &format!("# Written by `kav set check {how_often}`.\n[Timer]\nOnCalendar=\nOnCalendar={how_often}\n"))?;
    run(as_root("systemctl").arg("daemon-reload"))?;
    run(as_root("systemctl").args(["--quiet", "restart", TIMER]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PENDING: &str = "glibc 2.42-1 -> 2.42-2\nkavat-theme 0.3.0-1 -> 0.3.0-2\nkitty 0.43.1-1 -> 0.44.0-1\n";
    const OFFERED: &str = "\
Repository      : core
Name            : glibc
Version         : 2.42-2
Depends On      : linux-api-headers  tzdata
                  filesystem
Download Size   : 10.00 MiB
Installed Size  : 50.00 MiB

Repository      : kavat
Name            : kavat-theme
Version         : 0.3.0-2
Download Size   : 512.00 KiB
Installed Size  : 2.00 MiB

Repository      : extra
Name            : kitty
Version         : 0.44.0-1
Download Size   : 6.00 MiB
Installed Size  : 20.00 MiB
";
    const INSTALLED: &str = "\
Name            : glibc
Version         : 2.42-1
Installed Size  : 49.00 MiB

Name            : kavat-theme
Version         : 0.3.0-1
Installed Size  : 2.00 MiB

Name            : kitty
Version         : 0.43.1-1
Installed Size  : 22.00 MiB
";

    #[test]
    fn an_update_says_where_it_comes_from_and_what_it_weighs() {
        let all = from_repositories(PENDING, OFFERED, INSTALLED);
        assert_eq!(all.len(), 3);
        assert_eq!((all[0].source, all[0].download, all[0].change, all[0].restart), (Source::Arch, 10 << 20, 1 << 20, true));
        assert_eq!((all[1].source, all[1].download, all[1].change, all[1].restart), (Source::Kavat, 512 << 10, 0, false));
        assert_eq!((all[2].to.as_str(), all[2].change), ("0.44.0-1", -(2 << 20)));
        assert_eq!(total(&all), "3 packages · 16.5 MiB to download · -1.0 MiB on the disk · a restart is asked for after, by glibc");
    }

    #[test]
    fn a_package_that_is_held_back_is_not_counted() {
        let mut all = from_repositories("kitty 0.43.1-1 -> 0.44.0-1\n", OFFERED, INSTALLED);
        assert!(total(&all).ends_with("no restart is asked for"));
        all[0].held = true;
        assert_eq!(total(&all), "Nothing waits to be brought up to date.");
    }

    #[test]
    fn the_packages_that_ask_for_a_restart_are_known() {
        for name in ["linux", "linux-lts", "linux-zen", "linux-firmware", "amd-ucode", "systemd", "glibc", "lib32-glibc", "hyprland", "mesa", "lib32-mesa", "nvidia-utils", "nvidia-open-dkms", "vulkan-radeon"] {
            assert!(asks_restart(name), "{name}");
        }
        for name in ["linux-headers", "linux-lts-headers", "linux-api-headers", "kitty", "systemd-sysvcompat", "hyprland-qt-support", "mesa-utils"] {
            assert!(!asks_restart(name), "{name}");
        }
    }

    #[test]
    fn how_often_the_timer_looks_is_read_from_systemd_s_files() {
        assert_eq!(frequency(false, None), "weekly");
        assert_eq!(frequency(false, Some("[Timer]\nOnCalendar=\nOnCalendar=daily\n")), "daily");
        assert_eq!(frequency(false, Some("[Timer]\nOnCalendar=\nOnCalendar=weekly\n")), "weekly");
        assert_eq!(frequency(true, Some("[Timer]\nOnCalendar=\nOnCalendar=daily\n")), "never");
    }
}
