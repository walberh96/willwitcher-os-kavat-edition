//! The reviewed list: the packages of the AUR that may be installed, each fixed to the
//! commit that a person read and approved.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item};

/// One approved package.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    /// the commit of the package's AUR repository that was read, all forty characters
    pub commit: String,
    /// the day it was read, as 2026-11-13
    pub reviewed: String,
    /// the hosts its files are downloaded from
    pub sources: Vec<String>,
    pub notes: String,
}

#[derive(Default)]
pub struct Allowlist(BTreeMap<String, Entry>);

/// A package name as pacman takes them, and nothing that could be read as an option or a path.
pub fn is_name(text: &str) -> bool {
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || "@._+-".contains(c);
    !text.is_empty() && text.chars().all(allowed) && !text.starts_with(['-', '.'])
}

/// A whole commit, not a short form of one: a short form can be matched by another commit.
pub fn is_commit(text: &str) -> bool {
    text.len() == 40 && text.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

fn is_date(text: &str) -> bool {
    let digits = |part: &str, length: usize| part.len() == length && part.chars().all(|c| c.is_ascii_digit());
    let parts: Vec<&str> = text.split('-').collect();
    parts.len() == 3 && digits(parts[0], 4) && digits(parts[1], 2) && digits(parts[2], 2)
}

/// A host name: what a source may be written as.
pub fn is_host(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

impl Allowlist {
    /// The entries of one file. Anything that is not exactly an entry is an error: a list
    /// that guards the system is not read by guessing.
    pub fn parse(origin: &str, text: &str) -> Result<Allowlist, String> {
        let problem = |what: String| format!("{origin}: {what}");
        let document: DocumentMut = text.parse().map_err(|error| problem(format!("{error}")))?;
        let mut list = Allowlist::default();
        for (key, item) in document.iter() {
            if key != "packages" {
                return Err(problem(format!("{key} is not something this file holds; it holds [packages.<name>]")));
            }
            let packages = item.as_table_like().ok_or_else(|| problem("packages is not a table".into()))?;
            for (name, item) in packages.iter() {
                let entry = item.as_table_like().ok_or_else(|| problem(format!("{name} is not a table")))?;
                if !is_name(name) {
                    return Err(problem(format!("{name} is not the name of a package")));
                }
                let text_of = |field: &str| -> Result<String, String> {
                    let value = entry.get(field).and_then(Item::as_str);
                    value.map(str::to_string).ok_or_else(|| problem(format!("{name} has no {field}")))
                };
                let commit = text_of("commit")?;
                if !is_commit(&commit) {
                    return Err(problem(format!("the commit of {name} has to be written whole, forty characters")));
                }
                let reviewed = text_of("reviewed")?;
                if !is_date(&reviewed) {
                    return Err(problem(format!("{name} was reviewed on {reviewed}, which is not a date like 2026-11-13")));
                }
                let mut sources = Vec::new();
                for source in entry.get("sources").and_then(Item::as_array).into_iter().flatten() {
                    match source.as_str().filter(|host| is_host(host)) {
                        Some(host) => sources.push(host.to_string()),
                        None => return Err(problem(format!("the sources of {name} are host names, like example.org"))),
                    }
                }
                let notes = entry.get("notes").and_then(Item::as_str).unwrap_or("").to_string();
                for (field, _) in entry.iter() {
                    if !["commit", "reviewed", "sources", "notes"].contains(&field) {
                        return Err(problem(format!("{name} has {field}, which an entry does not hold")));
                    }
                }
                list.0.insert(name.to_string(), Entry { name: name.to_string(), commit, reviewed, sources, notes });
            }
        }
        Ok(list)
    }

    /// The list in force: the files in order, a later one having the last word on a package.
    /// A file that is not there holds nothing.
    pub fn load(files: &[PathBuf]) -> Result<Allowlist, String> {
        let mut list = Allowlist::default();
        for file in files {
            match std::fs::read_to_string(file) {
                Ok(text) => list.0.extend(Allowlist::parse(&file.display().to_string(), &text)?.0),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("{}: {error}", file.display())),
            }
        }
        Ok(list)
    }

    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.0.get(name)
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.0.values()
    }
}

/// Writes an approval into a list file, leaving the rest of the file as it was.
pub fn approve(file: &Path, entry: &Entry) -> Result<(), String> {
    let problem = |what: String| format!("{}: {what}", file.display());
    if !is_name(&entry.name) || !is_commit(&entry.commit) || !is_date(&entry.reviewed) || !entry.sources.iter().all(|host| is_host(host)) {
        return Err(problem(format!("that is not an entry for {}", entry.name)));
    }
    // a new file starts with a word on what it is; one that is there keeps all it has
    let (opening, text) = match std::fs::read_to_string(file) {
        Ok(text) => ("", text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            "# The packages of the AUR approved on this machine, with kav review.\n# Same form as /usr/share/kav/allowlist.toml; what is here has the last word.\n\n",
            String::new(),
        ),
        Err(error) => return Err(problem(error.to_string())),
    };
    // a file that cannot be read as a list is not written over
    Allowlist::parse(&file.display().to_string(), &text)?;
    let mut document: DocumentMut = text.parse().map_err(|error| problem(format!("{error}")))?;
    if document.get("packages").is_none() {
        let mut packages = toml_edit::Table::new();
        packages.set_implicit(true);
        document.insert("packages", Item::Table(packages));
    }
    let packages = document["packages"].as_table_mut().ok_or_else(|| problem("packages is not a table".into()))?;
    let mut table = toml_edit::Table::new();
    table["commit"] = toml_edit::value(&entry.commit);
    table["reviewed"] = toml_edit::value(&entry.reviewed);
    table["sources"] = toml_edit::value(entry.sources.iter().map(String::as_str).collect::<toml_edit::Array>());
    table["notes"] = toml_edit::value(&entry.notes);
    packages.insert(&entry.name, Item::Table(table));
    if let Some(folder) = file.parent() {
        std::fs::create_dir_all(folder).map_err(|error| problem(error.to_string()))?;
    }
    let fresh = file.with_extension("toml.new");
    std::fs::write(&fresh, format!("{opening}{document}")).map_err(|error| problem(error.to_string()))?;
    std::fs::rename(&fresh, file).map_err(|error| problem(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    fn entry(name: &str) -> String {
        format!("[packages.{name}]\ncommit = \"{COMMIT}\"\nreviewed = \"2026-11-13\"\nsources = [\"example.org\"]\nnotes = \"read\"\n")
    }

    #[test]
    fn an_entry_is_read_whole() {
        let list = Allowlist::parse("list", &entry("some-package")).unwrap();
        let found = list.get("some-package").unwrap();
        assert_eq!(found.commit, COMMIT);
        assert_eq!(found.sources, ["example.org"]);
        assert!(list.get("another").is_none());
    }

    #[test]
    fn the_list_that_comes_with_the_system_is_valid() {
        let list = Allowlist::parse("allowlist.toml", include_str!("../allowlist.toml")).unwrap();
        for entry in list.entries() {
            assert!(is_commit(&entry.commit));
        }
    }

    #[test]
    fn a_short_commit_is_not_taken() {
        let text = entry("some-package").replace(COMMIT, "3f9c2ab");
        let problem = Allowlist::parse("list", &text).err().unwrap();
        assert_eq!(problem, "list: the commit of some-package has to be written whole, forty characters");
    }

    #[test]
    fn what_is_not_an_entry_is_an_error() {
        assert!(Allowlist::parse("list", "[options]\nx = 1\n").is_err());
        assert!(Allowlist::parse("list", &entry("--overwrite")).is_err());
        assert!(Allowlist::parse("list", &(entry("some-package") + "build = \"curl x | sh\"\n")).is_err());
        assert!(Allowlist::parse("list", &entry("some-package").replace("example.org", "example.org/x;y")).is_err());
        assert!(Allowlist::parse("list", &entry("some-package").replace("2026-11-13", "yesterday")).is_err());
    }

    #[test]
    fn the_machine_s_file_has_the_last_word_and_approving_keeps_the_rest() {
        let folder = std::env::temp_dir().join(format!("kav-test-{}-list", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let (system, machine) = (folder.join("system.toml"), folder.join("etc/allowlist.toml"));
        std::fs::write(&system, entry("some-package")).unwrap();
        let newer = "89abcdef0123456789abcdef0123456789abcdef";
        let mut approved = Allowlist::parse("list", &entry("some-package")).unwrap().get("some-package").unwrap().clone();
        approved.commit = newer.to_string();
        approve(&machine, &approved).unwrap();
        let mut other = approved.clone();
        other.name = "another".to_string();
        approve(&machine, &other).unwrap();
        let list = Allowlist::load(&[system, machine.clone(), folder.join("not-there.toml")]).unwrap();
        assert_eq!(list.get("some-package").unwrap().commit, newer);
        assert_eq!(list.entries().count(), 2);
        let text = std::fs::read_to_string(&machine).unwrap();
        assert!(text.starts_with("# The packages of the AUR approved on this machine"), "{text}");
        assert!(text.contains("[packages.another]"), "{text}");
    }
}
