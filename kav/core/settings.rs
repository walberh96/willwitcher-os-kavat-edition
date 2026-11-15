//! The settings kav can change. Each lives in the file of the tool that uses it: pacman's,
//! snapper's, systemd's for the timer that looks for updates. kav keeps no copy of those,
//! it reads the file and changes its line. The few that no tool has a file for are kav's
//! own, in /etc/kav/kav.toml.

use std::process::Command;

use crate::config::{self, Config};
use crate::system::{Paths, as_root, is_root, myself_as_root, read, run};

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

pub fn mirror() -> Option<String> {
    mirror_in(&std::fs::read_to_string("/etc/pacman.d/mirrorlist").ok()?)
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
        format!("kav set takes: {}", each.join(", or "))
    };
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => {
            for setting in &SETTINGS {
                let now = get(paths, setting.name).map_or("it is root's to read".to_string(), |value| shown(setting, &value));
                println!("{:<11} {:<38} {now}", setting.name, setting.what);
            }
            println!("\n{:<11} {:<38} {}", "mirror", "The mirror in use", mirror().unwrap_or_else(|| "none is set in /etc/pacman.d/mirrorlist".into()));
            println!("\nWhat protects the system is not changed:");
            for (what, value) in LOCKED {
                println!("  {what}: {value}");
            }
            Ok(())
        }
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
}
