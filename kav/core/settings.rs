//! The settings kav can change. Each lives in the file of the tool that uses it, pacman's
//! or snapper's: kav keeps no copy, it reads the file and changes its line.

use std::process::Command;

use crate::system::{as_root, read, run};

/// One setting: its word after `kav set`, what it is, the values the full screen goes
/// through, and the least and the most it takes.
pub struct Setting {
    pub name: &'static str,
    pub what: &'static str,
    pub steps: &'static [u32],
    pub least: u32,
    pub most: u32,
}

pub const SETTINGS: [Setting; 2] = [
    Setting { name: "downloads", what: "Packages downloaded at once", steps: &[1, 3, 5, 8, 12], least: 1, most: 20 },
    Setting { name: "snapshots", what: "Snapshots kept", steps: &[5, 10, 20, 30], least: 2, most: 100 },
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

/// What a setting is now. None when it cannot be read: snapper's file is root's.
pub fn get(name: &str) -> Option<u32> {
    match name {
        "downloads" => std::fs::read_to_string(pacman_conf()).ok().map(|text| downloads_in(&text)),
        "snapshots" => std::fs::read_to_string("/etc/snapper/configs/root")
            .ok()
            .or_else(|| read(Command::new("sudo").args(["-n", "cat", "/etc/snapper/configs/root"])))
            .and_then(|text| snapshots_in(&text)),
        _ => None,
    }
}

/// The value after this one, for the full screen: the next step, and around to the first.
pub fn next(setting: &Setting, now: Option<u32>) -> u32 {
    let now = now.unwrap_or(0);
    setting.steps.iter().copied().find(|&step| step > now).unwrap_or(setting.steps[0])
}

/// `kav set`: with nothing after it, what each setting is; with a name and a number, the change.
pub fn set(arguments: &[String]) -> Result<(), String> {
    let takes = || {
        let each: Vec<String> = SETTINGS.iter().map(|s| format!("{} <{} to {}>", s.name, s.least, s.most)).collect();
        format!("kav set takes: {}", each.join(", or "))
    };
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => {
            for setting in &SETTINGS {
                let now = get(setting.name).map_or("it is root's to read".to_string(), |value| value.to_string());
                println!("{:<11} {:<30} {now}", setting.name, setting.what);
            }
            Ok(())
        }
        [name, value] => {
            let setting = SETTINGS.iter().find(|setting| setting.name == *name).ok_or_else(takes)?;
            let value: u32 = value.parse().ok().filter(|v| (setting.least..=setting.most).contains(v)).ok_or_else(takes)?;
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
                _ => run(as_root("snapper").args(["-c", "root", "set-config", &format!("NUMBER_LIMIT={value}")])),
            }?;
            println!("kav: {} is {value} now.", setting.what.to_lowercase());
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
        assert_eq!(next(&SETTINGS[0], Some(5)), 8);
        assert_eq!(next(&SETTINGS[0], Some(12)), 1);
        assert_eq!(next(&SETTINGS[0], Some(4)), 5);
        assert_eq!(next(&SETTINGS[1], None), 5);
    }
}
