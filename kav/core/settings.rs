//! The settings kav can change. Each lives in the file of the tool that uses it: pacman's,
//! snapper's, systemd's for the timer that looks for updates. kav keeps no copy of those,
//! it reads the file and changes its line. The few that no tool has a file for are kav's
//! own, in /etc/kav/kav.toml. The mirrors are pacman's list of them, which kav writes whole:
//! the ones Arch lists in a country, best first, or the ones that answer from anywhere.

use std::process::Command;

use crate::config::{self, Config};
use crate::system::{Paths, as_root, is_root, myself_as_root, read, run, someone_there, write_as_root};

/// What a setting takes: a number, with the values the full screen goes through and the
/// least and the most, or one of a few words.
pub enum Kind {
    Number { steps: &'static [u32], least: u32, most: u32 },
    Words(&'static [&'static str]),
}

/// One setting: its word after `kav set`, what it is, and what it takes.
pub struct Setting {
    pub name: &'static str,
    pub what: &'static str,
    pub kind: Kind,
}

pub const SETTINGS: [Setting; 5] = [
    Setting { name: "downloads", what: "Packages downloaded at once", kind: Kind::Number { steps: &[1, 3, 5, 8, 12], least: 1, most: 20 } },
    Setting { name: "snapshots", what: "Snapshots kept", kind: Kind::Number { steps: &[5, 10, 20, 30], least: 2, most: 100 } },
    Setting { name: "check", what: "Looks for updates", kind: Kind::Words(&["daily", "weekly", "never"]) },
    Setting { name: "cache", what: "Versions kept in pacman's cache", kind: Kind::Number { steps: &[1, 2, 3, 5], least: 1, most: 20 } },
    Setting { name: "jobs", what: "Processors a build from the AUR uses", kind: Kind::Number { steps: &[0, 1, 2, 4, 8, 16], least: 0, most: 256 } },
];

/// What protects the system, and so cannot be changed: shown, and locked.
pub const LOCKED: [(&str, &str); 4] = [
    ("The AUR, only through the reviewed list", "always"),
    ("A build from the AUR: in a clean system that cannot see your home", "always"),
    ("A build from the AUR: the reviewed commit, and no other", "always"),
    ("A snapshot before and after every change of packages", "always"),
];

fn pacman_conf() -> String {
    std::env::var("KAV_PACMAN_CONF").unwrap_or_else(|_| "/etc/pacman.conf".into())
}

/// What pacman's file says of the downloads at once: one, when it says nothing.
fn downloads_in(text: &str) -> u32 {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("ParallelDownloads"))
        .filter_map(|rest| rest.trim().strip_prefix('=')?.trim().parse().ok())
        .next_back()
        .unwrap_or(1)
}

/// What snapper's file says of the snapshots kept.
fn snapshots_in(text: &str) -> Option<u32> {
    text.lines().find_map(|line| line.trim().strip_prefix("NUMBER_LIMIT=")?.trim_matches('"').parse().ok())
}

/// The mirror in use: the first one in pacman's list of them, by its host.
fn mirror_in(text: &str) -> Option<String> {
    let address = text.lines().find_map(|line| line.trim().strip_prefix("Server")?.trim_start().strip_prefix('=')).map(str::trim)?;
    let host = address.split_once("://").map_or(address, |(_, rest)| rest).split('/').next()?;
    (!host.is_empty()).then(|| host.to_string())
}

fn mirrorlist() -> String {
    std::env::var("KAV_MIRRORLIST").unwrap_or_else(|_| "/etc/pacman.d/mirrorlist".into())
}

pub fn mirror() -> Option<String> {
    mirror_in(&std::fs::read_to_string(mirrorlist()).ok()?)
}

/// The mirrors that answer from anywhere: each request goes to a server nearby. They are
/// the ones the system comes with, and stay last in the list of a country.
const WORLDWIDE: [&str; 2] = ["https://geo.mirror.pkgbuild.com/$repo/os/$arch", "https://mirror.rackspace.com/archlinux/$repo/os/$arch"];

/// How many mirrors of a country are kept: pacman goes to the next when one fails.
const MIRRORS_KEPT: usize = 5;

/// A country as Arch's list of mirrors takes it: its two letters.
fn is_country(text: &str) -> bool {
    text.len() == 2 && text.chars().all(|c| c.is_ascii_alphabetic())
}

/// The mirrors in what Arch answers, in its order, which is best first. Only an address
/// as pacman takes one counts: the answer comes from the network.
fn mirrors_in(answer: &str) -> Vec<String> {
    let address = |line: &str| line.trim().trim_start_matches('#').trim_start().strip_prefix("Server")?.trim_start().strip_prefix('=').map(|rest| rest.trim().to_string());
    let sound = |address: &String| address.starts_with("https://") && address.ends_with("/$repo/os/$arch") && address.chars().all(|c| c.is_ascii_graphic());
    answer.lines().filter_map(address).filter(sound).collect()
}

/// pacman's list of mirrors, whole: those of a place, and the worldwide ones after them.
fn mirrorlist_of(place: &str, mirrors: &[String]) -> String {
    let mut text = match place {
        "worldwide" => "# Arch's worldwide mirrors: each request goes to a server nearby.\n".to_string(),
        country => format!("# Arch's mirrors in {}, the best first, and the worldwide ones after them.\n", country.to_uppercase()),
    };
    text.push_str("# kav set mirror wrote this file, and writes it again.\n");
    for mirror in mirrors.iter().take(MIRRORS_KEPT).map(String::as_str).chain(WORLDWIDE) {
        text.push_str(&format!("Server = {mirror}\n"));
    }
    text
}

/// What Arch lists as the mirrors of a country; KAV_MIRRORS names a file or another address.
fn mirrors_of(country: &str) -> Result<Vec<String>, String> {
    let from = std::env::var("KAV_MIRRORS").unwrap_or_else(|_| "https://archlinux.org/mirrorlist/".into());
    let answer = if from.starts_with("http://") || from.starts_with("https://") {
        let address = format!("{from}?country={}&protocol=https&use_mirror_status=on", country.to_uppercase());
        read(Command::new("curl").args(["--fail", "--silent", "--location", "--max-time", "15", "--", &address]))
    } else {
        std::fs::read_to_string(&from).ok()
    };
    let answer = answer.ok_or("Arch's list of mirrors could not be fetched: the mirrors stay as they were")?;
    match mirrors_in(&answer) {
        mirrors if mirrors.is_empty() => Err(format!("Arch lists no mirrors in {}: the mirrors stay as they were", country.to_uppercase())),
        mirrors => Ok(mirrors),
    }
}

/// `kav set mirror`: the mirrors of a country, or the worldwide ones. With no place named
/// and a person there, it asks.
fn set_mirror(place: Option<&str>) -> Result<(), String> {
    let takes = "kav set mirror takes a country by its two letters, like es or us, or worldwide";
    let place = match place {
        Some(place) => place.to_lowercase(),
        None if someone_there() => {
            use std::io::Write;
            println!("The mirror in use: {}", mirror().unwrap_or_else(|| "none".into()));
            print!("Mirrors of which country? Its two letters, like es or us, or worldwide: ");
            let _ = std::io::stdout().flush();
            let mut answer = String::new();
            let _ = std::io::stdin().read_line(&mut answer);
            answer.trim().to_lowercase()
        }
        None => return Err(takes.into()),
    };
    let mirrors = match place.as_str() {
        "worldwide" => Vec::new(),
        country if is_country(country) => mirrors_of(country)?,
        _ => return Err(takes.into()),
    };
    write_as_root(&mirrorlist(), &mirrorlist_of(&place, &mirrors))?;
    println!("kav: the mirror in use is {} now. The next change of packages comes from it.", mirror().unwrap_or_else(|| "none".into()));
    Ok(())
}

/// What a setting is now. None when it cannot be read: snapper's file is root's.
pub fn get(paths: &Paths, name: &str) -> Option<String> {
    let own = || Config::load(&paths.settings()).ok();
    match name {
        "downloads" => std::fs::read_to_string(pacman_conf()).ok().map(|text| downloads_in(&text).to_string()),
        "snapshots" => std::fs::read_to_string("/etc/snapper/configs/root")
            .ok()
            .or_else(|| read(Command::new("sudo").args(["-n", "cat", "/etc/snapper/configs/root"])))
            .and_then(|text| snapshots_in(&text))
            .map(|kept| kept.to_string()),
        "check" => Some(crate::updates::frequency_now().to_string()),
        "cache" => own().map(|config| config.cache.to_string()),
        "jobs" => own().map(|config| config.jobs.to_string()),
        _ => None,
    }
}

/// A value as it is shown: no processors named is all of them.
pub fn shown(setting: &Setting, value: &str) -> String {
    if setting.name == "jobs" && value == "0" { "all".into() } else { value.into() }
}

/// The value after this one, for the full screen: the next step, and around to the first.
pub fn next(setting: &Setting, now: Option<&str>) -> String {
    match setting.kind {
        Kind::Number { steps, .. } => {
            let now: Option<u32> = now.and_then(|now| now.parse().ok());
            steps.iter().copied().find(|&step| now.is_none_or(|now| step > now)).unwrap_or(steps[0]).to_string()
        }
        Kind::Words(words) => {
            let at = words.iter().position(|word| Some(*word) == now).map_or(0, |at| at + 1);
            words[at % words.len()].to_string()
        }
    }
}

/// Whether a setting takes a value.
fn takes_value(setting: &Setting, value: &str) -> bool {
    match setting.kind {
        Kind::Number { least, most, .. } => value.parse().is_ok_and(|value: u32| (least..=most).contains(&value)),
        Kind::Words(words) => words.contains(&value),
    }
}

/// `kav set`: with nothing after it, what each setting is; with a name and a value, the change.
pub fn set(paths: &Paths, arguments: &[String]) -> Result<(), String> {
    let takes = || {
        let each: Vec<String> = SETTINGS
            .iter()
            .map(|setting| match setting.kind {
                Kind::Number { least, most, .. } => format!("{} <{least} to {most}>", setting.name),
                Kind::Words(words) => format!("{} <{}>", setting.name, words.join(", ")),
            })
            .collect();
        format!("kav set takes: {}, or mirror <a country's two letters, or worldwide>", each.join(", or "))
    };
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => {
            for setting in &SETTINGS {
                let now = get(paths, setting.name).map_or("it is root's to read".to_string(), |value| shown(setting, &value));
                println!("{:<11} {:<38} {now}", setting.name, setting.what);
            }
            println!("{:<11} {:<38} {}", "mirror", "The mirror in use", mirror().unwrap_or_else(|| format!("none is set in {}", mirrorlist())));
            println!("\nWhat protects the system is not changed:");
            for (what, value) in LOCKED {
                println!("  {what}: {value}");
            }
            Ok(())
        }
        ["mirror"] => set_mirror(None),
        ["mirror", place] => set_mirror(Some(place)),
        [name, value] => {
            let setting = SETTINGS.iter().find(|setting| setting.name == *name).ok_or_else(takes)?;
            if !takes_value(setting, value) {
                return Err(takes());
            }
            // what is kav's own, or systemd's, is written by kav itself as root
            if ["check", "cache", "jobs"].contains(&setting.name) && !is_root() {
                return myself_as_root(&["set", name, value]);
            }
            match setting.name {
                "downloads" => {
                    let file = pacman_conf();
                    let text = std::fs::read_to_string(&file).map_err(|error| format!("{file}: {error}"))?;
                    // the line is changed where it is, commented or not; with none, it goes under [options]
                    let script = if text.lines().any(|line| line.trim_start_matches(['#', ' ']).starts_with("ParallelDownloads")) {
                        format!("s/^#\\? *ParallelDownloads.*/ParallelDownloads = {value}/")
                    } else {
                        format!("/^\\[options\\]/a ParallelDownloads = {value}")
                    };
                    run(as_root("sed").args(["-i", &script, &file]))
                }
                "snapshots" => run(as_root("snapper").args(["-c", "root", "set-config", &format!("NUMBER_LIMIT={value}")])),
                "check" => crate::updates::set_frequency(value),
                key => config::set_number(&paths.settings(), key, value.parse().unwrap_or(0)),
            }?;
            println!("kav: {} is {} now.", setting.what.to_lowercase(), shown(setting, value));
            if setting.name == "cache" {
                crate::cache::clean(paths)?;
            }
            Ok(())
        }
        _ => Err(takes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_files_are_read_for_what_they_say() {
        assert_eq!(downloads_in("[options]\n#ParallelDownloads = 9\nParallelDownloads = 5\n"), 5);
        assert_eq!(downloads_in("[options]\n#ParallelDownloads = 9\n"), 1);
        assert_eq!(snapshots_in("NUMBER_CLEANUP=\"yes\"\nNUMBER_LIMIT=\"10\"\nNUMBER_LIMIT_IMPORTANT=\"4\"\n"), Some(10));
        assert_eq!(snapshots_in("TIMELINE_CREATE=\"no\"\n"), None);
    }

    #[test]
    fn the_next_value_goes_around() {
        assert_eq!(next(&SETTINGS[0], Some("5")), "8");
        assert_eq!(next(&SETTINGS[0], Some("12")), "1");
        assert_eq!(next(&SETTINGS[0], Some("4")), "5");
        assert_eq!(next(&SETTINGS[1], None), "5");
        assert_eq!(next(&SETTINGS[2], Some("weekly")), "never");
        assert_eq!(next(&SETTINGS[2], Some("never")), "daily");
        assert_eq!(next(&SETTINGS[4], Some("16")), "0");
        assert_eq!(shown(&SETTINGS[4], "0"), "all");
    }

    #[test]
    fn a_setting_takes_only_its_own_values() {
        assert!(takes_value(&SETTINGS[0], "8") && !takes_value(&SETTINGS[0], "99") && !takes_value(&SETTINGS[0], "many"));
        assert!(takes_value(&SETTINGS[2], "daily") && !takes_value(&SETTINGS[2], "hourly"));
        assert!(takes_value(&SETTINGS[4], "0") && !takes_value(&SETTINGS[3], "0"));
    }

    #[test]
    fn the_mirror_in_use_is_the_first_of_the_list() {
        let list = "## Arch Linux mirrors\n#Server = https://commented.example.org/$repo/os/$arch\nServer = https://geo.mirror.pkgbuild.com/$repo/os/$arch\nServer = https://second.example.org/$repo/os/$arch\n";
        assert_eq!(mirror_in(list).as_deref(), Some("geo.mirror.pkgbuild.com"));
        assert_eq!(mirror_in("# nothing\n"), None);
    }

    #[test]
    fn the_mirrors_of_a_country_are_read_from_what_arch_answers() {
        let answer = "##\n## Arch Linux repository mirrorlist\n## Generated on 2026-11-16\n##\n\n## Spain\n#Server = https://ftp.rediris.es/mirror/archlinux/$repo/os/$arch\n\
                      ## Spain\n#Server = https://mirror.example.es/archlinux/$repo/os/$arch\n#Server = http://plain.example.es/$repo/os/$arch\n\
                      #Server = https://odd.example.es/$repo/os/$arch\tXferCommand = sh\n#Server = https://short.example.es/\n";
        assert_eq!(mirrors_in(answer), ["https://ftp.rediris.es/mirror/archlinux/$repo/os/$arch", "https://mirror.example.es/archlinux/$repo/os/$arch"]);
        // a country Arch does not know is answered with a page, not a list
        assert!(mirrors_in("<!DOCTYPE html>\n<html lang=\"en\">\n").is_empty());
        assert!(is_country("es") && is_country("US") && !is_country("spain") && !is_country("e1") && !is_country(""));
    }

    #[test]
    fn the_list_of_mirrors_is_written_whole() {
        let many: Vec<String> = (1..=8).map(|number| format!("https://m{number}.example.es/$repo/os/$arch")).collect();
        let text = mirrorlist_of("es", &many);
        assert!(text.starts_with("# Arch's mirrors in ES, the best first"), "{text}");
        // the best five of the country, and the worldwide ones after them
        assert_eq!(text.lines().filter(|line| line.starts_with("Server = ")).count(), 7);
        assert_eq!(mirror_in(&text).as_deref(), Some("m1.example.es"));
        assert!(text.ends_with("Server = https://mirror.rackspace.com/archlinux/$repo/os/$arch\n"), "{text}");
        let text = mirrorlist_of("worldwide", &[]);
        assert_eq!(mirror_in(&text).as_deref(), Some("geo.mirror.pkgbuild.com"));
        assert_eq!(text.lines().filter(|line| line.starts_with("Server = ")).count(), 2);
    }
}
