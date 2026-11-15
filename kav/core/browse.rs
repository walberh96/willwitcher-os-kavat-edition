//! The filters of Browse: which of the packages are shown. The same four are `kav list`'s.

use crate::system::read;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Filter {
    All,
    Installed,
    Explicit,
    Orphans,
    Aur,
}

impl Filter {
    /// The one after this, around to the first: what a key in Browse goes through.
    pub fn next(self) -> Filter {
        match self {
            Filter::All => Filter::Installed,
            Filter::Installed => Filter::Explicit,
            Filter::Explicit => Filter::Orphans,
            Filter::Orphans => Filter::Aur,
            Filter::Aur => Filter::All,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Filter::All => "all",
            Filter::Installed => "installed",
            Filter::Explicit => "explicit",
            Filter::Orphans => "orphans",
            Filter::Aur => "AUR",
        }
    }

    /// What pacman is asked for the packages of this filter. The AUR's are the ones no
    /// repository has. With no filter there is nothing to ask.
    fn question(self) -> Option<&'static str> {
        match self {
            Filter::All => None,
            Filter::Installed => Some("-Q"),
            Filter::Explicit => Some("-Qe"),
            Filter::Orphans => Some("-Qdt"),
            Filter::Aur => Some("-Qm"),
        }
    }

    /// The installed packages of this filter, each with its version.
    pub fn packages(self) -> Vec<(String, String)> {
        // pacman ends with an error when there are none, which reads as nothing here
        let Some(question) = self.question() else { return Vec::new() };
        pairs(&read(Command::new("pacman").arg(question)).unwrap_or_default())
    }
}

/// The packages in what pacman -Q answers: a name and a version on each line.
fn pairs(answer: &str) -> Vec<(String, String)> {
    answer.lines().filter_map(|line| line.split_once(' ')).map(|(name, version)| (name.to_string(), version.trim().to_string())).collect()
}

/// Which of the packages found by a search stay under a filter: the ones that are among
/// the filter's own. With no filter, all of them.
pub fn keep(filter: Filter, name: &str, among: &[(String, String)]) -> bool {
    filter == Filter::All || among.iter().any(|(other, _)| other == name)
}

/// `kav list --installed`, `--explicit`, `--orphans` and `--foreign`.
pub fn show(word: &str) -> Result<(), String> {
    let filter = match word {
        "--installed" => Filter::Installed,
        "--explicit" => Filter::Explicit,
        "--orphans" => Filter::Orphans,
        _ => Filter::Aur,
    };
    let all = filter.packages();
    if all.is_empty() {
        println!("kav: no package is like that.");
    }
    for (name, version) in all {
        println!("{name} {version}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_filters_go_around() {
        let mut filter = Filter::All;
        let mut titles = Vec::new();
        for _ in 0..6 {
            titles.push(filter.title());
            filter = filter.next();
        }
        assert_eq!(titles, ["all", "installed", "explicit", "orphans", "AUR", "all"]);
    }

    #[test]
    fn a_filter_keeps_only_its_own_packages() {
        let orphans = pairs("some-library 1.0-1\nold-tool 2.3-4\n");
        assert_eq!(orphans[1], ("old-tool".to_string(), "2.3-4".to_string()));
        assert!(keep(Filter::Orphans, "old-tool", &orphans));
        assert!(!keep(Filter::Orphans, "kitty", &orphans));
        assert!(keep(Filter::All, "kitty", &[]));
    }
}
