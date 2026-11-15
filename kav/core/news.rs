//! Arch's news, read before an update: now and then an item says that something has to be
//! done by hand, and an update that goes ahead without it can leave the system broken.

use std::path::PathBuf;
use std::process::Command;

use crate::system::{Paths, as_root, read, run, someone_there, write_as_root, yes};

/// The items that asked for something by hand and were dealt with on this machine, each
/// with the day: one line for each, the item's address first.
const DEALT_WITH: &str = "/var/lib/kav/interventions";

/// One item of the news.
#[derive(Debug, PartialEq)]
pub struct Item {
    pub title: String,
    pub date: String,
    pub link: String,
    pub text: String,
    /// the commands it names, as they are written in it
    pub commands: Vec<String>,
}

impl Item {
    /// Whether it asks for something to be done by hand.
    pub fn asks_for_hands(&self) -> bool {
        let words = format!("{} {}", self.title, self.text).to_lowercase();
        words.contains("manual intervention") || words.contains("manual action") || words.contains("requires intervention")
    }
}

/// Where the news is: Arch's feed, or what KAV_NEWS names, a file or another address.
fn source() -> String {
    std::env::var("KAV_NEWS").unwrap_or_else(|_| "https://archlinux.org/feeds/news/".into())
}

fn read_file(paths: &Paths) -> PathBuf {
    paths.requests.with_file_name("news-read")
}

/// What is between a tag's opening and its closing, in a piece of the feed.
fn inside<'a>(piece: &'a str, tag: &str) -> &'a str {
    let Some(start) = piece.find(&format!("<{tag}>")) else { return "" };
    let rest = &piece[start + tag.len() + 2..];
    rest.find(&format!("</{tag}>")).map_or("", |end| &rest[..end])
}

/// Text as it reads: without the marks of a web page, and with its few escapes undone.
fn plain(text: &str) -> String {
    let text = unescaped(text);
    let mut out = String::new();
    let mut in_mark = false;
    for letter in text.chars() {
        match letter {
            '<' => in_mark = true,
            '>' if in_mark => {
                in_mark = false;
                out.push(' ');
            }
            _ if !in_mark => out.push(letter),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn unescaped(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&")
}

/// What is between each opening of a mark and its closing, and the text without those pieces.
fn pieces(text: &str, mark: &str) -> (Vec<String>, String) {
    let (open, close) = (format!("<{mark}>"), format!("</{mark}>"));
    let (mut found, mut rest, mut left) = (Vec::new(), String::new(), text);
    while let Some(start) = left.find(&open) {
        let after = &left[start + open.len()..];
        let Some(end) = after.find(&close) else { break };
        rest.push_str(&left[..start]);
        found.push(after[..end].to_string());
        left = &after[end + close.len()..];
    }
    rest.push_str(left);
    (found, rest)
}

/// Whether a piece of code reads as a command and not as a name or as what a program
/// answered: it starts with a program that changes the system. In the running text it also
/// has to say something after it, since there a bare word is a name.
fn is_command(text: &str, alone: bool) -> bool {
    const PROGRAMS: [&str; 30] = [
        "pacman", "pacman-key", "systemctl", "rm", "mv", "cp", "ln", "mkdir", "touch", "chown", "chmod", "sed", "mkinitcpio", "grub-install", "grub-mkconfig",
        "bootctl", "locale-gen", "paccache", "pacdiff", "usermod", "gpasswd", "dkms", "depmod", "update-ca-trust", "archlinux-keyring-wkd-sync", "sbctl", "efibootmgr",
        "snapper", "btrfs", "fwupdmgr",
    ];
    let mut words = text.split_whitespace().skip_while(|word| *word == "sudo");
    words.next().is_some_and(|program| PROGRAMS.contains(&program)) && (alone || words.next().is_some())
}

/// The commands an item names: the lines of its blocks of code, and the pieces of code in
/// its running text, that read as commands. `description` is the item as the feed has it.
fn commands_in(description: &str) -> Vec<String> {
    let page = unescaped(description);
    let (blocks, running) = pieces(&page, "pre");
    let mut found: Vec<String> = Vec::new();
    let mut add = |line: &str, alone: bool| {
        // a line may be written after a prompt
        let line = line.trim();
        let line = line.strip_prefix("# ").or_else(|| line.strip_prefix("$ ")).unwrap_or(line).trim().to_string();
        if is_command(&line, alone) && !found.contains(&line) {
            found.push(line);
        }
    };
    for block in blocks {
        let block = block.replace("<code>", "").replace("</code>", "");
        unescaped(&block).lines().for_each(|line| add(line, true));
    }
    for code in pieces(&running, "code").0 {
        add(&unescaped(&code), false);
    }
    found
}

/// The items of a feed, the newest first, as the feed has them.
pub fn items(feed: &str) -> Vec<Item> {
    feed.split("<item>")
        .skip(1)
        .map(|piece| Item {
            title: plain(inside(piece, "title")),
            date: inside(piece, "pubDate").split_whitespace().skip(1).take(3).collect::<Vec<_>>().join(" "),
            link: inside(piece, "link").trim().to_string(),
            text: plain(inside(piece, "description")),
            commands: commands_in(inside(piece, "description")),
        })
        .filter(|item| !item.title.is_empty())
        .collect()
}

/// The news nobody has read on this machine yet; None when the news cannot be reached.
/// The first time, everything there is counts as read: it is older than the system.
pub fn unread(paths: &Paths) -> Option<Vec<Item>> {
    let from = source();
    let feed = if from.starts_with("http://") || from.starts_with("https://") {
        read(Command::new("curl").args(["--fail", "--silent", "--location", "--max-time", "15", "--", &from]))?
    } else {
        std::fs::read_to_string(&from).ok()?
    };
    let all = items(&feed);
    let file = read_file(paths);
    match std::fs::read_to_string(&file) {
        Ok(known) => {
            // what was dealt with on this machine is read for everyone who uses it
            let dealt = std::fs::read_to_string(DEALT_WITH).unwrap_or_default();
            let known = |item: &Item| known.lines().chain(dealt.lines().filter_map(|line| line.split('\t').next())).any(|line| line == item.link);
            Some(all.into_iter().filter(|item| !known(item)).collect())
        }
        Err(_) => {
            mark_read(paths, &all);
            Some(Vec::new())
        }
    }
}

pub fn mark_read(paths: &Paths, items: &[Item]) {
    let file = read_file(paths);
    let mut known = std::fs::read_to_string(&file).unwrap_or_default();
    for item in items {
        known.push_str(&item.link);
        known.push('\n');
    }
    if let Some(folder) = file.parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    let _ = std::fs::write(file, known);
}

/// `kav news`: the unread items, whole, and they are read from then on.
pub fn show(paths: &Paths) -> Result<(), String> {
    let Some(new) = unread(paths) else {
        return Err(format!("the news could not be reached at {}", source()));
    };
    if new.is_empty() {
        println!("kav: there is no news since the last time.");
        return Ok(());
    }
    for item in &new {
        println!("{}\n{}{}\n{}\n\n{}\n", item.title, item.date, if item.asks_for_hands() { "  ·  asks for something to be done by hand" } else { "" }, item.link, item.text);
        if item.asks_for_hands() && !item.commands.is_empty() {
            println!("The commands it names:");
            item.commands.iter().for_each(|command| println!("    {command}"));
            println!("kav update, in a terminal, goes through them with you and runs none by itself.\n");
        }
    }
    mark_read(paths, &new);
    Ok(())
}

/// Goes through an item that asks for something by hand, with the person: the item
/// whole, the commands found in it, each run only on a yes, and at the end whether the item
/// is dealt with. Ok when it is; that is then kept for this machine.
fn guide(item: &Item) -> Result<(), String> {
    println!("\n{}\n{}\n{}\n\n{}\n", item.title, item.date, item.link, item.text);
    if item.commands.is_empty() {
        println!("kav found no command in it: what it asks for is done by hand, in another terminal.");
    } else {
        println!("The commands it names:");
        item.commands.iter().for_each(|command| println!("    {command}"));
        println!("\nkav runs none of them by itself. Some are for before the update, and some stand in\nfor it or come after it: the item says which. Each one is asked for.\n");
        for command in &item.commands {
            if yes(&format!("Run `{command}` now, as root?")) {
                let line = command.strip_prefix("sudo ").unwrap_or(command);
                match run(as_root("sh").args(["-c", line])) {
                    Ok(()) => println!("kav: it ran."),
                    Err(problem) => println!("kav: it did not end well: {problem}"),
                }
            }
        }
    }
    if !yes("Is this item dealt with, so that the update goes on?") {
        return Err(format!("the update waits for: {}\n     `kav update` asks again", item.title));
    }
    let mut dealt = std::fs::read_to_string(DEALT_WITH).unwrap_or_default();
    dealt.push_str(&format!("{}\t{}\n", item.link, crate::system::today()));
    write_as_root(DEALT_WITH, &dealt)
}

/// Before an update: Ok when it may go ahead. News that asks for hands stops it: with a
/// person at the terminal each such item is gone through there and then, and without one
/// the update waits until the news has been read. The rest is only named.
pub fn before_update(paths: &Paths) -> Result<(), String> {
    let Some(new) = unread(paths) else {
        println!("kav: Arch's news could not be reached; the update goes ahead without it.");
        return Ok(());
    };
    if new.is_empty() {
        return Ok(());
    }
    println!("kav: news from Arch since the last time:");
    for item in &new {
        println!("  {}  {}{}", item.date, item.title, if item.asks_for_hands() { "  ← asks for something to be done by hand" } else { "" });
    }
    if new.iter().any(Item::asks_for_hands) && someone_there() {
        for item in new.iter().filter(|item| item.asks_for_hands()) {
            guide(item)?;
        }
    } else if new.iter().any(Item::asks_for_hands) {
        return Err("the update waits: `kav news` shows what has to be done by hand; after reading it, `kav update` goes ahead".into());
    }
    mark_read(paths, &new);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEED: &str = "<rss><channel><title>Arch Linux: Recent news updates</title>\
        <item><title>linux-firmware &gt;= 20250613 requires manual intervention</title><link>https://archlinux.org/news/one/</link>\
        <description>&lt;p&gt;Run &lt;code&gt;pacman -Rdd linux-firmware&lt;/code&gt; first.&lt;/p&gt;</description>\
        <pubDate>Sat, 21 Jun 2025 23:09:08 +0000</pubDate></item>\
        <item><title>Plasma 6.4 is out</title><link>https://archlinux.org/news/two/</link><description>&lt;p&gt;Nothing to do.&lt;/p&gt;</description>\
        <pubDate>Fri, 20 Jun 2025 07:08:19 +0000</pubDate></item></channel></rss>";

    #[test]
    fn a_feed_becomes_its_items_in_plain_words() {
        let found = items(FEED);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].title, "linux-firmware >= 20250613 requires manual intervention");
        assert_eq!(found[0].date, "21 Jun 2025");
        assert_eq!(found[0].text, "Run pacman -Rdd linux-firmware first.");
        assert_eq!(found[1].link, "https://archlinux.org/news/two/");
    }

    #[test]
    fn the_commands_an_item_names_are_found() {
        assert_eq!(items(FEED)[0].commands, ["pacman -Rdd linux-firmware"]);
        assert!(items(FEED)[1].commands.is_empty());
        let item = "&lt;p&gt;The &lt;code&gt;linux-firmware&lt;/code&gt; package was split. Run &lt;code&gt;sudo systemctl restart x&lt;/code&gt;, then:&lt;/p&gt;\
            &lt;pre&gt;&lt;code&gt;# pacman -Rdd linux-firmware\n$ pacman -Syu linux-firmware &amp;amp;&amp;amp; echo done\n\n# a remark\nerror: failed to commit transaction\n  mkinitcpio -P\n&lt;/code&gt;&lt;/pre&gt;\
            &lt;p&gt;See &lt;code&gt;pacman&lt;/code&gt; and &lt;code&gt;/etc/mkinitcpio.conf&lt;/code&gt;.&lt;/p&gt;";
        assert_eq!(commands_in(item), ["pacman -Rdd linux-firmware", "pacman -Syu linux-firmware && echo done", "mkinitcpio -P", "sudo systemctl restart x"]);
    }

    #[test]
    fn an_item_that_asks_for_hands_is_told_from_one_that_does_not() {
        let found = items(FEED);
        assert!(found[0].asks_for_hands());
        assert!(!found[1].asks_for_hands());
    }
}
