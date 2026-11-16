//! kav: one command over pacman.
//!
//! A package comes from one of three places: Arch's repositories, the system's own, [kavat],
//! or the AUR. The first two are pacman's, and kav hands them over as they are. The AUR is
//! only reached through the reviewed list: a package on it is built from the exact commit
//! that a person approved, and a package that is not on it is refused.

mod allowlist;
mod aur;
mod browse;
mod cache;
mod config;
mod doctor;
mod firmware;
mod guard;
mod log;
mod migrate;
mod news;
mod review;
mod settings;
mod snapshots;
mod system;
mod tui;
mod updates;
mod version;

use allowlist::{Allowlist, Entry};
use std::io::{IsTerminal, Write};
use std::process::Command;
use system::{Paths, as_root, read, run};

const USAGE: &str = "\
usage: kav             in a terminal: everything below on one screen
       kav <command>

  install <package>…   install packages: from Arch's repositories, from [kavat], or from the
                       AUR when the package is on the reviewed list
  remove <package>…    remove packages, with what only they needed
  update               bring the whole system up to date, and the AUR packages whose
                       reviewed commit moved; then the migrations and the firmware
  update --list        what an update would bring, with its sizes; nothing is changed
  check                look for updates without changing the system
  hold <package>       leave an AUR package as it is when updating; unhold lets it go
  search <words>       look for a package in the repositories and on the reviewed list
  info <package>       what a package is, and where it would come from
  list --aur           the reviewed list, with what is installed from it and what is asked for
  list --installed     what is installed; --explicit what was asked for by name, --orphans
                       what nothing needs any more, --foreign what no repository has
  review <package>     read an AUR package, or what changed in it since it was reviewed,
                       with the lines worth a second look marked; then approve it or not
  request <package>    ask for an AUR package to be reviewed
  undo [--yes]         take the system back to before the last change of packages
  snapshots            the snapshots, with what each change did; then size [<number>] for
                       the room each takes, compare <number> with the system now,
                       boot <number> to start from it the next time, once,
                       delete <number>, pin <number> or unpin <number>
  migrate [--list]     run the migrations that wait: what a new version changes that
                       pacman does not manage
  news                 Arch's news since the last time: an update waits for it to be read
                       when an item asks for something to be done by hand
  doctor               check the health of the system
  fix <what>           mend what doctor found: orphans, lock, cache, service <unit>,
                       merge <file ending in .pacnew>
  set [<what> <value>] what kav can change, and changing it: downloads at once, snapshots
                       kept, check for updates, cache versions kept, jobs of a build,
                       mirror <country> for the mirrors of a country, or worldwide

The reviewed list is /usr/share/kav/allowlist.toml, and /etc/kav/allowlist.toml for what
was approved on this machine.";

/// Where a package would come from.
#[derive(Debug, PartialEq)]
enum Origin {
    /// one of pacman's repositories, by its name; "kavat" is the system's own
    Repository(String),
    /// the AUR, at the commit of its entry on the reviewed list
    Aur(Entry),
    /// nowhere that kav installs from
    Refused,
}

/// Decides where a package comes from. `repository` says which of pacman's repositories
/// has it, if any: they come first, so a package of the AUR never stands in for one of Arch's.
fn origin(name: &str, list: &Allowlist, repository: &dyn Fn(&str) -> Option<String>) -> Origin {
    if !allowlist::is_name(name) {
        return Origin::Refused;
    }
    match (repository(name), list.get(name)) {
        (Some(repository), _) => Origin::Repository(repository),
        (None, Some(entry)) => Origin::Aur(entry.clone()),
        (None, None) => Origin::Refused,
    }
}

/// Which of pacman's repositories has a package, a group or something that provides the name.
fn in_pacman(name: &str) -> Option<String> {
    if let Some(found) = read(Command::new("pacman").args(["-Si", "--", name])) {
        let repository = found.lines().find_map(|line| line.strip_prefix("Repository").map(|rest| rest.trim_start_matches([' ', ':']).to_string()));
        return Some(repository.unwrap_or_else(|| "arch".into()));
    }
    // not a package by that name: a group, or a name that packages provide
    read(Command::new("pacman").args(["-Sp", "--print-format", "%r", "--", name])).map(|found| found.lines().next().unwrap_or("arch").to_string())
}

/// pacman knows what the repositories hold from lists it fetches. A system that was just
/// installed has none yet, and without them every package would look like it is nowhere.
fn need_lists() -> Result<(), String> {
    let lists = std::fs::read_dir("/var/lib/pacman/sync").into_iter().flatten().filter_map(Result::ok);
    if lists.map(|entry| entry.path()).any(|path| path.extension().is_some_and(|ending| ending == "db")) {
        return Ok(());
    }
    Err("pacman has not fetched its lists of packages yet, so there is no telling where a package comes from.\n     kav update fetches them, and brings the system up to date.".into())
}

fn installed_version(name: &str) -> Option<String> {
    read(Command::new("pacman").args(["-Q", "--", name])).and_then(|found| found.split_whitespace().nth(1).map(str::to_string))
}

/// Whether the AUR has a package by that name, when it can be asked. None when it cannot.
fn in_the_aur(paths: &Paths, name: &str) -> Option<bool> {
    let address = format!("{}/rpc/v5/info?arg[]={name}", paths.aur);
    let answer = read(Command::new("curl").args(["--silent", "--fail", "--max-time", "5", "--globoff", &address]))?;
    Some(answer.contains("\"resultcount\":1"))
}

fn refusal(paths: &Paths, name: &str) -> String {
    if !allowlist::is_name(name) {
        return format!("{name} is not the name of a package");
    }
    let place = match in_the_aur(paths, name) {
        Some(true) => "is in the AUR, but not on the reviewed list",
        _ => "is not in Arch's repositories, in [kavat] or on the reviewed list of the AUR",
    };
    format!("{name} {place}.\n     Only what was read and approved comes from the AUR. To ask for it: kav request {name}")
}

fn install(paths: &Paths, names: &[String]) -> Result<(), String> {
    if names.is_empty() {
        return Err("install takes the names of the packages: kav install firefox".into());
    }
    need_lists()?;
    let list = Allowlist::load(&paths.lists)?;
    let mut from_pacman = Vec::new();
    let mut from_aur = Vec::new();
    let mut refused = Vec::new();
    for name in names {
        match origin(name, &list, &in_pacman) {
            Origin::Repository(_) => from_pacman.push(name.as_str()),
            Origin::Aur(entry) => from_aur.push(entry),
            Origin::Refused => refused.push(refusal(paths, name)),
        }
    }
    // one package that cannot come stops them all: nothing is left half done
    if !refused.is_empty() {
        return Err(format!("{}\n     Nothing was installed.", refused.join("\nkav: ")));
    }
    // what the AUR's packages need is settled before anything is installed
    let builds = plan(paths, &from_aur, &list)?;
    if !from_pacman.is_empty() {
        let mut arguments = vec!["-S", "--needed", "--"];
        arguments.extend(&from_pacman);
        guard::pacman(&arguments)?;
    }
    build_and_install(paths, &builds, &from_aur)
}

/// Whether pacman can give a package without the AUR: it is installed, or something
/// installed provides it, or a repository has it.
fn settled(name: &str) -> bool {
    Command::new("pacman").args(["-T", "--", name]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok_and(|status| status.success()) || in_pacman(name).is_some()
}

/// The packages of the AUR to build for the ones asked for, in order: each is fetched, left
/// at its reviewed commit and asked what it needs. Nothing is built here.
fn plan(paths: &Paths, asked: &[Entry], list: &Allowlist) -> Result<Vec<Entry>, String> {
    let needs = |entry: &Entry| -> Result<Vec<String>, String> {
        println!("kav: {} comes from the AUR, at the commit reviewed on {}: {}", entry.name, entry.reviewed, &entry.commit[..12]);
        let (folder, _) = aur::fetch(paths, &entry.name)?;
        aur::check_out(&folder, entry)?;
        Ok(aur::needs(&folder, &entry.commit))
    };
    let builds = aur::order(asked, list, &needs, &settled)?;
    let brought: Vec<&str> = builds.iter().filter(|entry| !asked.contains(entry)).map(|entry| entry.name.as_str()).collect();
    if !brought.is_empty() {
        println!("kav: also from the AUR, because the above need them, each one reviewed by itself: {}", brought.join(", "));
    }
    Ok(builds)
}

/// Builds the packages of a plan, in its order, and installs each as it is built. The
/// ones that were not asked for by name are installed as what another package needs.
fn build_and_install(paths: &Paths, builds: &[Entry], asked: &[Entry]) -> Result<(), String> {
    if builds.is_empty() {
        return Ok(());
    }
    let jobs = config::Config::load(&paths.settings())?.jobs;
    let mut built: Vec<std::path::PathBuf> = Vec::new();
    for entry in builds {
        let folder = paths.cache.join("aur").join(&entry.name);
        aur::check_out(&folder, entry)?;
        let packages = aur::build(paths, &folder, &built, jobs)?;
        let files: Vec<String> = packages.iter().map(|package| package.to_string_lossy().into_owned()).collect();
        let mut arguments = vec!["-U", "--needed"];
        if !asked.contains(entry) && installed_version(&entry.name).is_none() {
            arguments.push("--asdeps");
        }
        arguments.push("--");
        arguments.extend(files.iter().map(String::as_str));
        guard::pacman(&arguments)?;
        system::write_as_root(&format!("/var/lib/kav/built/{}", entry.name), &format!("{}\n", entry.commit))?;
        built.extend(packages);
    }
    Ok(())
}

fn remove(names: &[String]) -> Result<(), String> {
    if names.is_empty() || !names.iter().all(|name| allowlist::is_name(name)) {
        return Err("remove takes the names of the packages: kav remove firefox".into());
    }
    let mut arguments = vec!["-Rns", "--"];
    arguments.extend(names.iter().map(String::as_str));
    guard::pacman(&arguments)
}

/// The packages installed from outside pacman's repositories, which is where the AUR's are.
fn foreign() -> Vec<String> {
    read(Command::new("pacman").arg("-Qmq")).unwrap_or_default().lines().map(str::to_string).collect()
}

fn update(paths: &Paths) -> Result<(), String> {
    // news that asks for something to be done by hand stops the update until it is dealt with
    news::before_update(paths)?;
    let held = config::Config::load(&paths.settings())?.hold;
    let log_before = log::text().len();
    guard::pacman(&["-Syu"])?;
    let list = Allowlist::load(&paths.lists)?;
    let mut moved = Vec::new();
    for name in foreign() {
        match list.get(&name) {
            Some(entry) if aur::built_commit(&name).as_deref() == Some(entry.commit.as_str()) => {}
            Some(_) if held.contains(&name) => println!("kav: {name} is held back: it stays as it is. kav unhold {name} lets it go"),
            Some(entry) => moved.push(entry.clone()),
            None => println!("kav: {name} is installed but not on the reviewed list: it stays as it is"),
        }
    }
    let builds = plan(paths, &moved, &list)?;
    build_and_install(paths, &builds, &moved)?;
    // what the new version changes that pacman does not manage; one that fails is told at the end too
    let migrated = migrate::after_update();
    if let Err(problem) = &migrated {
        println!("\nkav: {problem}");
    }
    if let Err(problem) = cache::clean(paths) {
        println!("kav: pacman's cache was not cleaned: {problem}");
    }
    firmware::step();
    // the system is looked over after the change, while going back is one command away
    println!("\nkav: the system after the update:");
    if doctor::check().is_err() {
        println!("\nkav: something is not as it should be. `kav fix` mends what it can; `kav undo` takes the system back to before this update.");
    }
    let restart = updates::restart_after(log_before);
    if !restart.is_empty() {
        println!("\nkav: restart the machine when you can: {} changed, and what is running is still the old one.", restart.join(", "));
    }
    migrated.map_err(|_| "the update is in, but a migration failed: see above".to_string())
}

/// `kav hold <package>` and `kav unhold <package>`: only a package of the AUR can be held
/// back. Arch's move all together.
fn hold(paths: &Paths, name: &str, held: bool) -> Result<(), String> {
    if !allowlist::is_name(name) {
        return Err(format!("{name} is not the name of a package"));
    }
    let list = Allowlist::load(&paths.lists)?;
    if list.get(name).is_none() && !foreign().iter().any(|other| other == name) {
        return Err(format!("{name} is not a package of the AUR. Arch's packages are brought up to date all together:\n     Arch does not stand behind a system with some of them held back."));
    }
    if !system::is_root() {
        return system::myself_as_root(&[if held { "hold" } else { "unhold" }, name]);
    }
    config::set_held(&paths.settings(), name, held)?;
    match held {
        true => println!("kav: {name} is held back: kav update leaves it as it is."),
        false => println!("kav: {name} is not held back any more."),
    }
    Ok(())
}

fn search(paths: &Paths, words: &[String]) -> Result<(), String> {
    if words.is_empty() {
        return Err("search takes what to look for: kav search editor".into());
    }
    need_lists()?;
    let found = Command::new("pacman").arg("-Ss").arg("--").args(words).status().is_ok_and(|status| status.success());
    let list = Allowlist::load(&paths.lists)?;
    let matches = |entry: &&Entry| words.iter().all(|word| entry.name.contains(&word.to_lowercase()) || entry.notes.to_lowercase().contains(&word.to_lowercase()));
    let reviewed: Vec<&Entry> = list.entries().filter(matches).collect();
    for entry in &reviewed {
        println!("aur/{} (reviewed {})\n    {}", entry.name, entry.reviewed, entry.notes);
    }
    if !found && reviewed.is_empty() {
        return Err("nothing by those words in the repositories or on the reviewed list".into());
    }
    Ok(())
}

fn info(paths: &Paths, name: &str) -> Result<(), String> {
    need_lists()?;
    let list = Allowlist::load(&paths.lists)?;
    match origin(name, &list, &in_pacman) {
        Origin::Repository(_) => run(Command::new("pacman").args(["-Si", "--", name])),
        Origin::Aur(entry) => {
            println!("Name      : {}", entry.name);
            println!("Comes from: the AUR, on the reviewed list");
            println!("Commit    : {}", entry.commit);
            println!("Reviewed  : {}", entry.reviewed);
            println!("Sources   : {}", entry.sources.join(", "));
            println!("Notes     : {}", entry.notes);
            println!("Installed : {}", installed_version(&entry.name).unwrap_or_else(|| "no".into()));
            Ok(())
        }
        Origin::Refused => Err(refusal(paths, name)),
    }
}

fn requests(paths: &Paths) -> Vec<String> {
    std::fs::read_to_string(&paths.requests).unwrap_or_default().lines().map(str::to_string).collect()
}

fn list_aur(paths: &Paths) -> Result<(), String> {
    let list = Allowlist::load(&paths.lists)?;
    let installed = foreign();
    if list.entries().next().is_none() {
        println!("The reviewed list is empty: no package of the AUR has been approved.");
    }
    for entry in list.entries() {
        let state = match (installed_version(&entry.name), aur::built_commit(&entry.name)) {
            (None, _) => "not installed".to_string(),
            (Some(version), Some(built)) if built == entry.commit => format!("installed, {version}"),
            (Some(version), _) => format!("installed, {version}; kav update builds the reviewed commit"),
        };
        println!("{}  {}  reviewed {}  {state}", entry.name, &entry.commit[..12], entry.reviewed);
    }
    for name in installed.iter().filter(|name| list.get(name).is_none()) {
        println!("{name}  installed, but not on the reviewed list");
    }
    for name in requests(paths) {
        if list.get(&name).is_none() {
            println!("{name}  asked for; kav review {name} reads it");
        }
    }
    Ok(())
}

fn request(paths: &Paths, name: &str) -> Result<(), String> {
    if !allowlist::is_name(name) {
        return Err(format!("{name} is not the name of a package"));
    }
    need_lists()?;
    let list = Allowlist::load(&paths.lists)?;
    if let Origin::Repository(repository) = origin(name, &list, &in_pacman) {
        return Err(format!("{name} needs no review: it is in {repository}, and kav install {name} installs it"));
    }
    if list.get(name).is_some() {
        return Err(format!("{name} is already on the reviewed list: kav install {name} installs it"));
    }
    let mut asked = requests(paths);
    if !asked.iter().any(|other| other == name) {
        asked.push(name.to_string());
        let problem = |error: std::io::Error| format!("{}: {error}", paths.requests.display());
        std::fs::create_dir_all(paths.requests.parent().unwrap()).map_err(problem)?;
        std::fs::write(&paths.requests, asked.join("\n") + "\n").map_err(problem)?;
    }
    println!("kav: {name} is asked for. kav review {name} shows what it would bring, and approves it or not.");
    Ok(())
}

/// Shows an AUR package for a person to read: all of it the first time, and what changed
/// since the reviewed commit after that. Then asks whether to approve what was shown.
fn review_package(paths: &Paths, name: &str) -> Result<(), String> {
    need_lists()?;
    let list = Allowlist::load(&paths.lists)?;
    if let Some(repository) = in_pacman(name) {
        return Err(format!("{name} needs no review: it is in {repository}"));
    }
    let (folder, head) = aur::fetch(paths, name)?;
    let approved = list.get(name);
    if approved.is_some_and(|entry| entry.commit == head) {
        println!("kav: {name} is at the commit that was reviewed, {}. There is nothing new to read.", &head[..12]);
        return Ok(());
    }
    let files = aur::files_at(&folder, &head);
    let mut found = 0;
    match approved {
        Some(entry) => {
            println!("{name}: what changed since the commit reviewed on {}, {} -> {}\n", entry.reviewed, &entry.commit[..12], &head[..12]);
            for line in aur::difference(&folder, &entry.commit, &head).lines() {
                // only what the change brings in is judged: lines that start with one +
                let added = line.strip_prefix('+').filter(|rest| !rest.starts_with("++"));
                match added.and_then(review::suspicion) {
                    Some(why) => {
                        found += 1;
                        println!("{line}\n    ^^^ {why}");
                    }
                    None => println!("{line}"),
                }
            }
        }
        None => {
            println!("{name}: not reviewed yet. This is what the AUR has, at {}\n", &head[..12]);
            for (file, text) in &files {
                println!("--- {file}");
                for (at, line) in text.lines().enumerate() {
                    println!("{:4}  {line}", at + 1);
                    if let Some(why) = review::suspicion(line) {
                        found += 1;
                        println!("      ^^^ {why}");
                    }
                }
                println!();
            }
        }
    }
    let hosts: Vec<String> = files.iter().flat_map(|(_, text)| review::hosts(text)).fold(Vec::new(), |mut all, host| {
        if !all.contains(&host) {
            all.push(host);
        }
        all
    });
    let known = approved.map(|entry| entry.sources.clone()).unwrap_or_default();
    let fresh: Vec<&String> = hosts.iter().filter(|host| !known.contains(host)).collect();
    println!("\nLines worth a second look: {found}");
    println!("Downloads from: {}", if hosts.is_empty() { "nowhere".to_string() } else { hosts.join(", ") });
    if approved.is_some() && !fresh.is_empty() {
        println!("New since the review: {}", fresh.iter().map(|host| host.as_str()).collect::<Vec<_>>().join(", "));
    }
    if !std::io::stdin().is_terminal() {
        println!("\nTo approve it, run kav review {name} in a terminal.");
        return Ok(());
    }
    print!("\nApproving fixes {name} to this commit on this machine. Type its name to approve it, or Enter to leave it: ");
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    let _ = std::io::stdin().read_line(&mut answer);
    if answer.trim() != name {
        println!("kav: {name} was not approved.");
        return Ok(());
    }
    let mut note = String::new();
    print!("A note on what it is and what you looked at (optional): ");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut note);
    let program = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut pin = as_root(&program.to_string_lossy());
    pin.args(["pin", name, &head, note.trim()]).args(&hosts);
    run(&mut pin)?;
    println!("kav: {name} is on the reviewed list, at {}. kav install {name} installs it.", &head[..12]);
    Ok(())
}

/// The step of a review that writes the approval, as root: kav pin <name> <commit> <notes> [host…]
fn pin(paths: &Paths, arguments: &[String]) -> Result<(), String> {
    let [name, commit, notes, hosts @ ..] = arguments else {
        return Err("pin is a step of kav review, which gives it what it needs".into());
    };
    if !system::is_root() {
        return Err("an approval is written as root: kav review does it".into());
    }
    let notes: String = notes.chars().filter(|c| !c.is_control()).collect();
    let entry = Entry { name: name.clone(), commit: commit.clone(), reviewed: system::today(), sources: hosts.to_vec(), notes };
    allowlist::approve(paths.machine_list(), &entry)
}

/// Takes the system back to before the last change of packages. Each of pacman's
/// transactions leaves a snapshot from before it, which says what the change was.
fn undo(sure: bool) -> Result<(), String> {
    let listed = read(as_root("snapper").args(["--no-dbus", "-c", "root", "--csvout", "--no-headers", "list", "--columns", "number,type,date,description"]))
        .ok_or("the snapshots could not be listed")?;
    let last = listed.lines().filter_map(|line| {
        let mut fields = line.splitn(4, ',');
        let (number, kind, date, what) = (fields.next()?, fields.next()?, fields.next()?, fields.next()?);
        (kind == "pre").then(|| (number.to_string(), date.to_string(), what.to_string()))
    });
    let Some((number, date, what)) = last.last() else {
        return Err("there is no change of packages to go back from".into());
    };
    println!("The last change of packages: {what}\nSnapshot {number} is the system as it was before it, on {date}.");
    if !sure {
        if !std::io::stdin().is_terminal() {
            return Err("to go back without being asked: kav undo --yes".into());
        }
        print!("Go back to it? [y/N] ");
        let _ = std::io::stdout().flush();
        let mut answer = String::new();
        let _ = std::io::stdin().read_line(&mut answer);
        if !answer.trim().eq_ignore_ascii_case("y") {
            println!("kav: nothing was changed.");
            return Ok(());
        }
    }
    run(as_root("kavat-rollback").arg(&number))?;
    println!("kav: restart to be back there. /home is not touched.");
    Ok(())
}

fn command(arguments: &[String]) -> Result<(), String> {
    let paths = Paths::from_environment();
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["install", ..] => install(&paths, &arguments[1..]),
        ["remove", ..] => remove(&arguments[1..]),
        ["update"] => update(&paths),
        ["update", "--list"] => updates::show(&paths),
        ["check"] => updates::check(false),
        ["check", "--notify"] => updates::check(true),
        ["hold", name] => hold(&paths, name, true),
        ["unhold", name] => hold(&paths, name, false),
        ["list", filter @ ("--installed" | "--explicit" | "--orphans" | "--foreign")] => browse::show(filter),
        ["snapshots", ..] => snapshots::command(&arguments[1..]),
        ["migrate", ..] => migrate::command(&arguments[1..]),
        ["aside", ..] => guard::aside(&arguments[1..]),
        ["search", ..] => search(&paths, &arguments[1..]),
        ["info", name] => info(&paths, name),
        ["list", "--aur"] => list_aur(&paths),
        ["review", name] => review_package(&paths, name),
        ["request", name] => request(&paths, name),
        ["pin", ..] => pin(&paths, &arguments[1..]),
        ["undo"] => undo(false),
        ["undo", "--yes"] => undo(true),
        ["doctor"] => doctor::check(),
        ["news"] => news::show(&paths),
        ["fix", ..] => doctor::fix(&arguments[1..]),
        ["set", ..] => settings::set(&paths, &arguments[1..]),
        // with nothing asked and a person at the terminal: everything on one screen
        [] if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => tui::run(paths),
        [] | ["help"] | ["--help"] | ["-h"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(format!("that is not a command\n\n{USAGE}")),
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Err(problem) = command(&arguments) {
        eprintln!("kav: {problem}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> Allowlist {
        let text = "[packages.reviewed-tool]\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\nreviewed = \"2026-11-13\"\n\
                    [packages.firefox]\ncommit = \"89abcdef0123456789abcdef0123456789abcdef\"\nreviewed = \"2026-11-13\"\n";
        Allowlist::parse("list", text).unwrap()
    }

    fn arch(name: &str) -> Option<String> {
        match name {
            "firefox" => Some("extra".into()),
            "kavat-theme" => Some("kavat".into()),
            _ => None,
        }
    }

    #[test]
    fn a_package_comes_from_pacman_first_then_from_the_reviewed_list() {
        assert_eq!(origin("kavat-theme", &list(), &arch), Origin::Repository("kavat".into()));
        assert!(matches!(origin("reviewed-tool", &list(), &arch), Origin::Aur(entry) if entry.commit.starts_with("0123")));
        // a package of the AUR never stands in for one that Arch has
        assert_eq!(origin("firefox", &list(), &arch), Origin::Repository("extra".into()));
    }

    #[test]
    fn a_package_outside_the_list_is_refused() {
        assert_eq!(origin("yay", &list(), &arch), Origin::Refused);
        assert_eq!(origin("--overwrite=*", &list(), &arch), Origin::Refused);
        assert_eq!(origin("", &list(), &arch), Origin::Refused);
    }
}
