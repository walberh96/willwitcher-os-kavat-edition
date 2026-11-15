//! Arch's news, read before an update: now and then an item says that something has to be
//! done by hand, and an update that goes ahead without it can leave the system broken.

use std::path::PathBuf;
use std::process::Command;

use crate::system::{read, Paths};

/// One item of the news.
#[derive(Debug, PartialEq)]
pub struct Item {
    pub title: String,
    pub date: String,
    pub link: String,
    pub text: String,
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
    let text = text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&");
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

/// The items of a feed, the newest first, as the feed has them.
pub fn items(feed: &str) -> Vec<Item> {
    feed.split("<item>")
        .skip(1)
        .map(|piece| Item {
            title: plain(inside(piece, "title")),
            date: inside(piece, "pubDate").split_whitespace().skip(1).take(3).collect::<Vec<_>>().join(" "),
            link: inside(piece, "link").trim().to_string(),
            text: plain(inside(piece, "description")),
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
        Ok(known) => Some(all.into_iter().filter(|item| !known.lines().any(|line| line == item.link)).collect()),
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
    }
    mark_read(paths, &new);
    Ok(())
}

/// Before an update: Ok when it may go ahead. News that asks for hands stops it until it
/// has been read; the rest is only named.
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
    if new.iter().any(Item::asks_for_hands) {
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
    fn an_item_that_asks_for_hands_is_told_from_one_that_does_not() {
        let found = items(FEED);
        assert!(found[0].asks_for_hands());
        assert!(!found[1].asks_for_hands());
    }
}
