//! pacman's cache of downloaded packages: how many versions of each are kept. The older
//! ones are what a downgrade by hand would need, and nothing else.

use crate::config::Config;
use crate::system::{Paths, as_root, has, run, size};
use std::collections::BTreeMap;

const CACHE: &str = "/var/cache/pacman/pkg";

/// The name, the version and the architecture in the name of a package's file.
fn parts(file: &str) -> Option<(&str, &str, &str)> {
    let stem = &file[..file.find(".pkg.tar")?];
    let (rest, architecture) = stem.rsplit_once('-')?;
    let (rest, release) = rest.rsplit_once('-')?;
    let (name, _) = rest.rsplit_once('-')?;
    let version = &stem[name.len() + 1..stem.len() - architecture.len() - 1];
    (!name.is_empty() && !release.is_empty()).then_some((name, version, architecture))
}

/// The files that are beyond what is kept: of each package, all but its newest versions.
/// A signature goes with its package. What is not a package's file is left alone.
pub fn beyond(files: &[String], keep: u32) -> Vec<String> {
    let mut versions: BTreeMap<(&str, &str), Vec<&str>> = BTreeMap::new();
    for file in files.iter().filter(|file| !file.ends_with(".sig")) {
        if let Some((name, version, architecture)) = parts(file) {
            let known = versions.entry((name, architecture)).or_default();
            if !known.contains(&version) {
                known.push(version);
            }
        }
    }
    let mut old = Vec::new();
    for ((name, architecture), mut known) in versions {
        known.sort_by(|one, two| crate::version::compare(two, one));
        for version in known.into_iter().skip(keep as usize) {
            let start = format!("{name}-{version}-{architecture}.pkg.tar");
            old.extend(files.iter().filter(|file| file.starts_with(&start)).cloned());
        }
    }
    old.sort();
    old
}

/// Takes out of the cache what is beyond the versions kept: with paccache when
/// pacman-contrib is installed, and by kav itself when it is not.
pub fn clean(paths: &Paths) -> Result<(), String> {
    let keep = Config::load(&paths.settings())?.cache;
    if has("paccache") {
        return run(as_root("paccache").args(["--remove", "--keep", &keep.to_string()]));
    }
    let files: Vec<String> = std::fs::read_dir(CACHE).into_iter().flatten().filter_map(Result::ok).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
    let old = beyond(&files, keep);
    if old.is_empty() {
        return Ok(());
    }
    let freed: u64 = old.iter().filter_map(|file| std::fs::metadata(format!("{CACHE}/{file}")).ok()).map(|file| file.len()).sum();
    run(as_root("rm").arg("-f").arg("--").args(old.iter().map(|file| format!("{CACHE}/{file}"))))?;
    println!("kav: {} older files left pacman's cache, {}. It keeps {keep} versions of each package.", old.len(), size(freed));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_s_name_says_which_package_and_version_it_is() {
        assert_eq!(parts("xorg-server-21.1.20-1-x86_64.pkg.tar.zst"), Some(("xorg-server", "21.1.20-1", "x86_64")));
        assert_eq!(parts("kavat-icons-1:0.2-3-any.pkg.tar.zst.sig"), Some(("kavat-icons", "1:0.2-3", "any")));
        assert_eq!(parts("download-abc123"), None);
    }

    #[test]
    fn only_the_newest_versions_of_each_package_stay() {
        let files: Vec<String> = [
            "kitty-0.9-1-x86_64.pkg.tar.zst",
            "kitty-0.10-1-x86_64.pkg.tar.zst",
            "kitty-0.10-1-x86_64.pkg.tar.zst.sig",
            "kitty-0.10-2-x86_64.pkg.tar.zst",
            "kitty-0.8-1-x86_64.pkg.tar.zst",
            "kitty-0.8-1-x86_64.pkg.tar.zst.sig",
            "kitty-terminfo-0.8-1-any.pkg.tar.zst",
            "zsh-5.9-5-x86_64.pkg.tar.zst",
            "notes.txt",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(beyond(&files, 2), ["kitty-0.8-1-x86_64.pkg.tar.zst", "kitty-0.8-1-x86_64.pkg.tar.zst.sig", "kitty-0.9-1-x86_64.pkg.tar.zst"]);
        assert_eq!(beyond(&files, 1).len(), 5);
        assert!(beyond(&files, 4).is_empty());
    }
}
