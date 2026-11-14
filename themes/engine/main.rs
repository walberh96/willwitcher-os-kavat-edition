//! kavat-theme: the look of the whole system, from one file.
//!
//! The settings in ~/.config/kavat/theme.toml are resolved into tokens, the tokens fill
//! in a template for each app, and the apps are told to read their files again.

mod apply;
mod color;
mod template;
mod theme;

use apply::Paths;
use theme::{KEYS, Token};

const USAGE: &str = "\
usage: kavat-theme <command>

  apply [--no-reload]      make every app's file from the settings and tell the apps
  set <setting> <value>…   change settings in ~/.config/kavat/theme.toml and apply them,
                           e.g. kavat-theme set mode light, or set palette nord accent frost
  get [setting]            the settings in force
  boot                     give the boot menu, the loading screen and the login screen the
                           look in force; they are system files, so it asks for a password
  list                     the palettes, each with its accents
  tokens                   every colour and value the templates can use

The settings and what each one takes are in /usr/share/kavat/themes/theme.toml.";

/// Prints a line. When whoever was reading has gone, as in `kavat-theme tokens | head`,
/// there is nothing left to say.
fn say(line: &str) {
    use std::io::Write;
    if writeln!(std::io::stdout(), "{line}").is_err() {
        std::process::exit(0);
    }
}

fn show(token: &Token) -> String {
    match token {
        Token::Color(colour) => colour.to_string(),
        Token::Number(number) => format!("{number}"),
        Token::Flag(flag) => format!("{flag}"),
        Token::Text(text) => text.clone(),
    }
}

/// Makes the files and puts them in place; with `reload`, tells the apps when they changed.
fn apply(paths: &Paths, user: Option<&toml_edit::DocumentMut>, reload: bool) -> Result<apply::Plan, String> {
    let settings = apply::settings(paths, user)?;
    let tokens = apply::tokens(paths, &settings)?;
    let plan = apply::plan(paths, &tokens)?;
    if user.is_none() && apply::commit(paths, &plan)? && reload {
        apply::reload(paths, &plan);
    }
    Ok(plan)
}

fn set(paths: &Paths, pairs: &[String]) -> Result<(), String> {
    if pairs.is_empty() || !pairs.len().is_multiple_of(2) {
        return Err("set takes a setting and its value, once or more: kavat-theme set mode light".into());
    }
    let file = paths.user_file();
    let mut document = apply::read_document(&file)?.unwrap_or_default();
    for pair in pairs.chunks(2) {
        theme::write_setting(&mut document, &pair[0], &pair[1]).map_err(|problem| match problem {
            theme::WriteProblem::Value(problem) => problem,
            theme::WriteProblem::NotASection(part) => format!("{}: {part} is not a section", file.display()),
        })?;
    }
    // Everything is worked out with the new settings before the file is written: a value
    // that cannot be applied never reaches it.
    let plan = apply(paths, Some(&document), false)?;
    let problem = |error: std::io::Error| format!("{}: {error}", file.display());
    std::fs::create_dir_all(file.parent().unwrap()).map_err(problem)?;
    let fresh = file.with_extension("toml.new");
    std::fs::write(&fresh, document.to_string()).map_err(problem)?;
    std::fs::rename(&fresh, &file).map_err(problem)?;
    if apply::commit(paths, &plan)? {
        apply::reload(paths, &plan);
    }
    Ok(())
}

/// The look of the screens before the desktop. They are files of the system, so the work
/// is done as root: a person's command hands over the settings in force, and nothing else,
/// to the same command run through pkexec. Root keeps the settings it was handed, so that
/// run again with none, as the installer and an update do, it gives the same look; until
/// someone hands it any, the look is the machine's own.
fn boot(pairs: &[String]) -> Result<(), String> {
    let root = std::fs::metadata("/proc/self").is_ok_and(|me| std::os::unix::fs::MetadataExt::uid(&me) == 0);
    if !root {
        if !pairs.is_empty() {
            return Err("boot takes no settings: it uses the ones in force".into());
        }
        let settings = apply::settings(&Paths::from_environment()?, None)?;
        let program = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut command = std::process::Command::new("pkexec");
        command.arg(program).arg("boot");
        for (key, _) in KEYS {
            command.arg(key).arg(show(settings.get(key)));
        }
        return match command.status() {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err("the boot keeps the look it had".into()),
            Err(error) => Err(format!("pkexec: {error}")),
        };
    }
    if !pairs.len().is_multiple_of(2) {
        return Err("boot takes settings in pairs: kavat-theme boot mode light".into());
    }
    let paths = Paths::for_the_system();
    let kept = paths.state.join("settings.toml");
    let mut given = if pairs.is_empty() { apply::read_document(&kept)?.unwrap_or_default() } else { toml_edit::DocumentMut::new() };
    for pair in pairs.chunks(2) {
        theme::write_setting(&mut given, &pair[0], &pair[1]).map_err(|problem| match problem {
            theme::WriteProblem::Value(problem) => problem,
            theme::WriteProblem::NotASection(part) => format!("{part} is not a section"),
        })?;
    }
    let settings = apply::settings(&paths, Some(&given))?;
    let plan = apply::plan(&paths, &apply::tokens(&paths, &settings)?)?;
    apply::commit(&paths, &plan)?;
    if !pairs.is_empty() {
        std::fs::write(&kept, given.to_string()).map_err(|error| format!("{}: {error}", kept.display()))?;
    }
    apply::install(&paths, &plan)
}

fn run(arguments: &[String]) -> Result<(), String> {
    if arguments.first().is_some_and(|command| command == "boot") {
        return boot(&arguments[1..]);
    }
    let paths = Paths::from_environment()?;
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["apply"] => apply(&paths, None, true).map(drop),
        ["apply", "--no-reload"] => apply(&paths, None, false).map(drop),
        ["set", ..] => set(&paths, &arguments[1..]),
        ["get"] => {
            let settings = apply::settings(&paths, None)?;
            for (key, _) in KEYS {
                say(&format!("{key} = {}", show(settings.get(key))));
            }
            Ok(())
        }
        ["get", key] => {
            let settings = apply::settings(&paths, None)?;
            theme::kind_of(key).ok_or_else(|| theme::parse_value(key, "").unwrap_err())?;
            say(&show(settings.get(key)));
            Ok(())
        }
        ["list"] => {
            let folders = paths.palette_folders();
            for id in theme::available(&folders) {
                let palette = theme::Palette::load(&id, &folders)?;
                let accents: Vec<&str> = palette.accents.iter().map(|(name, _)| name.as_str()).collect();
                let modes = match (&palette.dark, &palette.light) {
                    (Some(_), Some(_)) => "dark and light",
                    (Some(_), None) => "dark only",
                    _ => "light only",
                };
                say(&format!("{id}: {}, {modes}; accents: {}", palette.name, accents.join(", ")));
            }
            Ok(())
        }
        ["tokens"] => {
            let settings = apply::settings(&paths, None)?;
            for (name, token) in apply::tokens(&paths, &settings)? {
                say(&format!("{name} {}", show(&token)));
            }
            Ok(())
        }
        [] | ["help"] | ["--help"] | ["-h"] => {
            say(USAGE);
            Ok(())
        }
        _ => Err(format!("that is not a command\n\n{USAGE}")),
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Err(problem) = run(&arguments) {
        eprintln!("kavat-theme: {problem}");
        std::process::exit(1);
    }
}
