//! Packages of the AUR: fetching the commit that was reviewed, and no other, and building
//! it in a clean system that cannot see anyone's home folder.

use crate::allowlist::{Entry, is_commit, is_name};
use crate::system::{Paths, as_root, is_root, read, run};
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(folder: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(folder).env("GIT_TERMINAL_PROMPT", "0");
    command
}

/// Brings the AUR's repository of a package up to date in the cache, and gives its folder
/// and the commit the AUR offers today.
pub fn fetch(paths: &Paths, name: &str) -> Result<(PathBuf, String), String> {
    if !is_name(name) {
        return Err(format!("{name} is not the name of a package"));
    }
    let folder = paths.cache.join("aur").join(name);
    let address = format!("{}/{name}.git", paths.aur);
    if folder.join(".git").exists() {
        // the address is set again each time: the cache is not trusted to remember it
        run(git(&folder).args(["remote", "set-url", "origin", &address]))?;
        run(git(&folder).args(["fetch", "--quiet", "--prune", "origin"])).map_err(|_| format!("the AUR could not be reached for {name}"))?;
    } else {
        std::fs::create_dir_all(folder.parent().unwrap()).map_err(|error| format!("{}: {error}", folder.display()))?;
        let mut clone = Command::new("git");
        clone.env("GIT_TERMINAL_PROMPT", "0").args(["clone", "--quiet", &address]).arg(&folder);
        if run(&mut clone).is_err() {
            let _ = std::fs::remove_dir_all(&folder);
            return Err(format!("the AUR could not be reached for {name}"));
        }
    }
    // The AUR answers with an empty repository for a package that does not exist.
    let head = read(git(&folder).args(["rev-parse", "--verify", "--quiet", "origin/HEAD^{commit}"]))
        .or_else(|| read(git(&folder).args(["rev-parse", "--verify", "--quiet", "origin/master^{commit}"])))
        .map(|commit| commit.trim().to_string())
        .filter(|commit| is_commit(commit));
    match head {
        Some(head) => Ok((folder, head)),
        None => {
            let _ = std::fs::remove_dir_all(&folder);
            Err(format!("there is no package called {name} in the AUR"))
        }
    }
}

/// Leaves the package's folder holding exactly the reviewed commit: its files, and nothing
/// that a build or anyone else left there.
pub fn check_out(folder: &Path, entry: &Entry) -> Result<(), String> {
    let refused = || format!("the reviewed commit of {}, {}, is not in the AUR's history of it: nothing was built", entry.name, &entry.commit[..12]);
    run(git(folder).args(["cat-file", "-e", &format!("{}^{{commit}}", entry.commit)]).stderr(std::process::Stdio::null())).map_err(|_| refused())?;
    run(git(folder).args(["checkout", "--quiet", "--force", "--detach", &entry.commit])).map_err(|_| refused())?;
    run(git(folder).args(["clean", "--quiet", "-fdx"]))?;
    let now = read(git(folder).args(["rev-parse", "HEAD"])).unwrap_or_default();
    if now.trim() != entry.commit {
        return Err(refused());
    }
    Ok(())
}

/// Builds the package in the clean system and gives the files it made.
pub fn build(paths: &Paths, folder: &Path) -> Result<Vec<PathBuf>, String> {
    if is_root() {
        return Err("a package of the AUR is built as a normal user, not as root".into());
    }
    // Arch's own tools for building in a clean system. They bring a good deal with them,
    // so they are not part of the system until the first package of the AUR is built.
    if read(Command::new("sh").args(["-c", "command -v makechrootpkg mkarchroot arch-nspawn"])).is_none() {
        println!("kav: building from the AUR needs devtools, from Arch's repositories");
        run(as_root("pacman").args(["-S", "--needed", "devtools"]))?;
    }
    let root = paths.chroot.join("root");
    if root.join("usr/bin/pacman").exists() {
        run(as_root("arch-nspawn").arg(&root).args(["pacman", "-Syu", "--noconfirm"]))?;
    } else {
        println!("kav: making the clean system that AUR packages are built in; this happens once");
        run(as_root("mkdir").arg("-p").arg(&paths.chroot))?;
        run(as_root("mkarchroot").arg(&root).arg("base-devel"))?;
    }
    let out = folder.join("out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|error| format!("{}: {error}", out.display()))?;
    // -c starts from a fresh copy of the clean system each time
    run(Command::new("makechrootpkg").current_dir(folder).env("PKGDEST", &out).arg("-c").arg("-r").arg(&paths.chroot))?;
    let mut packages: Vec<PathBuf> = std::fs::read_dir(&out)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.to_string_lossy().contains(".pkg.tar.") && !path.to_string_lossy().ends_with(".sig"))
        .collect();
    packages.sort();
    if packages.is_empty() {
        return Err("the build made no package".into());
    }
    Ok(packages)
}

/// What a package's build files say, at a commit: the PKGBUILD and the scripts beside it.
pub fn files_at(folder: &Path, commit: &str) -> Vec<(String, String)> {
    let names = read(git(folder).args(["ls-tree", "--name-only", "-r", commit])).unwrap_or_default();
    let mut files = Vec::new();
    for name in names.lines() {
        let build_file = name == "PKGBUILD" || name.ends_with(".install") || name.ends_with(".sh") || name.ends_with(".hook");
        if build_file && let Some(text) = read(git(folder).args(["show", &format!("{commit}:{name}")])) {
            files.push((name.to_string(), text));
        }
    }
    files
}

/// What changed in the package between two commits, as git writes a difference.
pub fn difference(folder: &Path, from: &str, to: &str) -> String {
    read(git(folder).args(["diff", "--no-color", from, to])).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder that stands in for the AUR, with one package of two commits.
    fn world(name: &str) -> (Paths, String, String) {
        let root = std::env::temp_dir().join(format!("kav-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let package = root.join("aur/some-package.git");
        std::fs::create_dir_all(&package).unwrap();
        let commit = |text: &str| {
            std::fs::write(package.join("PKGBUILD"), text).unwrap();
            for arguments in [vec!["add", "PKGBUILD"], vec!["-c", "user.name=t", "-c", "user.email=t@t", "commit", "--quiet", "-m", "x"]] {
                assert!(git(&package).args(arguments).status().unwrap().success());
            }
            read(git(&package).args(["rev-parse", "HEAD"])).unwrap().trim().to_string()
        };
        assert!(git(&package).args(["init", "--quiet", "--initial-branch=master"]).status().unwrap().success());
        let first = commit("pkgname=some-package\npkgver=1\n");
        let second = commit("pkgname=some-package\npkgver=2\ninstall=x.install\n");
        let paths = Paths {
            lists: vec![root.join("system.toml"), root.join("machine.toml")],
            cache: root.join("cache"),
            requests: root.join("requests"),
            chroot: root.join("chroot"),
            aur: format!("file://{}", root.join("aur").display()),
        };
        (paths, first, second)
    }

    fn entry(commit: &str) -> Entry {
        Entry { name: "some-package".into(), commit: commit.into(), reviewed: "2026-11-13".into(), sources: vec![], notes: String::new() }
    }

    #[test]
    fn the_reviewed_commit_is_what_gets_built_even_when_the_aur_moved_on() {
        let (paths, first, second) = world("pinned");
        let (folder, head) = fetch(&paths, "some-package").unwrap();
        assert_eq!(head, second);
        std::fs::write(folder.join("left-by-a-build"), "x").unwrap();
        check_out(&folder, &entry(&first)).unwrap();
        assert_eq!(std::fs::read_to_string(folder.join("PKGBUILD")).unwrap(), "pkgname=some-package\npkgver=1\n");
        assert!(!folder.join("left-by-a-build").exists());
        // fetching again finds the same folder and the same newest commit
        assert_eq!(fetch(&paths, "some-package").unwrap().1, second);
    }

    #[test]
    fn a_commit_the_aur_does_not_have_is_refused() {
        let (paths, _, _) = world("refused");
        let (folder, _) = fetch(&paths, "some-package").unwrap();
        let problem = check_out(&folder, &entry("0123456789abcdef0123456789abcdef01234567")).unwrap_err();
        assert!(problem.contains("is not in the AUR's history of it: nothing was built"), "{problem}");
    }

    #[test]
    fn a_package_the_aur_does_not_have_is_an_error() {
        let (paths, _, _) = world("missing");
        assert!(fetch(&paths, "not-there").is_err());
        assert!(fetch(&paths, "../some-package").is_err());
    }

    #[test]
    fn what_changed_since_the_review_can_be_read() {
        let (paths, first, second) = world("changed");
        let (folder, _) = fetch(&paths, "some-package").unwrap();
        let changed = difference(&folder, &first, &second);
        assert!(changed.contains("+install=x.install"), "{changed}");
        let files = files_at(&folder, &second);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "PKGBUILD");
    }
}
