//! Packages of the AUR: fetching the commit that was reviewed, and no other, and building
//! it in a clean system that cannot see anyone's home folder.

use crate::allowlist::{Allowlist, Entry, is_commit, is_name};
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

/// Which commit an installed AUR package was built from: kav keeps it beside the package.
pub fn built_commit(name: &str) -> Option<String> {
    std::fs::read_to_string(format!("/var/lib/kav/built/{name}")).ok().map(|text| text.trim().to_string())
}

/// The packages that a package's own record of itself, its .SRCINFO, says it needs: to
/// run, to build and to check. Each is a bare name, without the versions asked of it.
pub fn needs_in(record: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in record.lines() {
        let Some((field, value)) = line.trim().split_once(" = ") else { continue };
        // a field may be for one architecture only: depends_x86_64
        let field = field.strip_suffix("_x86_64").unwrap_or(field);
        if ["depends", "makedepends", "checkdepends"].contains(&field) {
            let name = value.split(['<', '>', '=', ':']).next().unwrap_or("").trim().to_string();
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// What a package needs, at the commit that was reviewed.
pub fn needs(folder: &Path, commit: &str) -> Vec<String> {
    needs_in(&read(git(folder).args(["show", &format!("{commit}:.SRCINFO")])).unwrap_or_default())
}

/// The packages of the AUR to build for a request, in the order to build them: each one
/// after what it needs. What a package needs comes from pacman's repositories when it can
/// (`settled` says so, also for what is installed already) and from the reviewed list
/// when it cannot; each package has to be on the list by itself, and one that is not stops
/// everything, by its name. `needs` gives what a package on the list needs.
pub fn order(
    asked: &[Entry],
    list: &Allowlist,
    needs: &dyn Fn(&Entry) -> Result<Vec<String>, String>,
    settled: &dyn Fn(&str) -> bool,
) -> Result<Vec<Entry>, String> {
    fn visit(
        entry: &Entry,
        asked: &[Entry],
        list: &Allowlist,
        needs: &dyn Fn(&Entry) -> Result<Vec<String>, String>,
        settled: &dyn Fn(&str) -> bool,
        path: &mut Vec<String>,
        done: &mut Vec<Entry>,
    ) -> Result<(), String> {
        if done.iter().any(|other| other.name == entry.name) {
            return Ok(());
        }
        if path.contains(&entry.name) {
            return Err(format!("{} needs itself, by way of {}: nothing was built", entry.name, path.join(", ")));
        }
        path.push(entry.name.clone());
        for name in needs(entry)? {
            // one that is being built anyway is built first, even when an older build of it is installed
            if let Some(other) = asked.iter().find(|other| other.name == name) {
                visit(other, asked, list, needs, settled, path, done)?;
            } else if !settled(&name) {
                match list.get(&name) {
                    Some(other) => visit(other, asked, list, needs, settled, path, done)?,
                    None => {
                        return Err(format!(
                            "{} needs {name}, which is not in Arch's repositories, in [kavat] or on the reviewed list of the AUR.\n     \
                             Each package of the AUR is approved by itself. To ask for it: kav request {name}",
                            entry.name
                        ));
                    }
                }
            }
        }
        path.pop();
        done.push(entry.clone());
        Ok(())
    }
    let mut done = Vec::new();
    for entry in asked {
        visit(entry, asked, list, needs, settled, &mut Vec::new(), &mut done)?;
    }
    Ok(done)
}

/// Builds the package in the clean system and gives the files it made. `with` are packages
/// built just before that this one needs: they are put into the clean system first.
/// `jobs` is how many processors the build may use; 0 is all of them.
pub fn build(paths: &Paths, folder: &Path, with: &[PathBuf], jobs: u32) -> Result<Vec<PathBuf>, String> {
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
    let jobs = if jobs == 0 { std::thread::available_parallelism().map_or(1, |count| count.get() as u32) } else { jobs };
    let mut make = Command::new("makechrootpkg");
    make.current_dir(folder).env("PKGDEST", &out).env("MAKEFLAGS", format!("-j{jobs}")).arg("-c").arg("-r").arg(&paths.chroot);
    for package in with {
        make.arg("-I").arg(package);
    }
    run(&mut make)?;
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

    const RECORD: &str = "pkgbase = some-package\n\tpkgver = 1\n\tmakedepends = cmake\n\tdepends = glibc\n\tdepends = some-library>=2.0\n\
                          \tdepends_x86_64 = lib32-thing\n\tdepends_aarch64 = not-here\n\toptdepends = extra: for more\n\tcheckdepends = some-library\n\npkgname = some-package\n";

    #[test]
    fn what_a_package_needs_is_read_from_its_own_record() {
        assert_eq!(needs_in(RECORD), ["cmake", "glibc", "some-library", "lib32-thing"]);
    }

    fn reviewed(names: &[&str]) -> Allowlist {
        let text: String = names.iter().map(|name| format!("[packages.{name}]\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\nreviewed = \"2026-11-13\"\n")).collect();
        Allowlist::parse("list", &text).unwrap()
    }

    /// A small world: what each package of the AUR needs, and what pacman can give.
    fn needs_of(entry: &Entry) -> Result<Vec<String>, String> {
        Ok(match entry.name.as_str() {
            "app" => vec!["glibc", "library", "helper"],
            "library" => vec!["base-library", "glibc"],
            "helper" => vec!["library"],
            "lonely" => vec!["not-reviewed"],
            "snake" => vec!["tail"],
            "tail" => vec!["snake"],
            _ => vec![],
        }
        .into_iter()
        .map(String::from)
        .collect())
    }

    #[test]
    fn what_a_package_needs_from_the_aur_is_built_before_it() {
        let list = reviewed(&["app", "library", "helper", "base-library", "lonely", "snake", "tail"]);
        let names = |asked: &[&str], settled: &dyn Fn(&str) -> bool| -> Result<Vec<String>, String> {
            let asked: Vec<Entry> = asked.iter().map(|name| list.get(name).unwrap().clone()).collect();
            order(&asked, &list, &needs_of, settled).map(|all| all.into_iter().map(|entry| entry.name).collect())
        };
        let in_arch = |name: &str| name == "glibc";
        assert_eq!(names(&["app"], &in_arch).unwrap(), ["base-library", "library", "helper", "app"]);
        // what is installed already is not built again, unless it was asked for too
        let installed = |name: &str| name == "glibc" || name == "library";
        assert_eq!(names(&["app"], &installed).unwrap(), ["helper", "app"]);
        assert_eq!(names(&["app", "library"], &installed).unwrap(), ["base-library", "library", "helper", "app"]);
    }

    #[test]
    fn a_need_that_is_not_approved_stops_everything_by_its_name() {
        let list = reviewed(&["app", "library", "helper", "base-library", "lonely", "snake", "tail"]);
        let ask = |names: &[&str]| {
            let asked: Vec<Entry> = names.iter().map(|name| list.get(name).unwrap().clone()).collect();
            order(&asked, &list, &needs_of, &|name| name == "glibc").unwrap_err()
        };
        let problem = ask(&["app", "lonely"]);
        assert!(problem.starts_with("lonely needs not-reviewed, which is not in Arch's repositories") && problem.ends_with("kav request not-reviewed"), "{problem}");
        assert!(ask(&["snake"]).starts_with("snake needs itself"), "{}", ask(&["snake"]));
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
