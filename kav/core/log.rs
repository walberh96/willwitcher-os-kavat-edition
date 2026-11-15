//! pacman's own log, /var/log/pacman.log: what each change of packages did. It is kept
//! outside the snapshots, so it remembers changes that were gone back from too.

/// One package in a change: what was done to it, and its versions as pacman wrote them.
#[derive(Debug, PartialEq)]
pub struct Change {
    /// installed, upgraded, removed, reinstalled or downgraded
    pub verb: &'static str,
    pub name: String,
    pub versions: String,
}

/// One change of packages: when it started, as 2026-11-13 10:15:00, and what it did.
#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub when: String,
    pub changes: Vec<Change>,
}

pub fn text() -> String {
    std::fs::read_to_string("/var/log/pacman.log").unwrap_or_default()
}

/// The changes of packages in a log, oldest first. One that did nothing is left out.
pub fn transactions(log: &str) -> Vec<Transaction> {
    let mut all: Vec<Transaction> = Vec::new();
    let mut open = false;
    for line in log.lines() {
        let Some((stamp, rest)) = line.strip_prefix('[').and_then(|line| line.split_once("] ")) else { continue };
        let Some(rest) = rest.strip_prefix("[ALPM] ") else { continue };
        if rest.starts_with("transaction started") {
            all.push(Transaction { when: stamp.replace('T', " ").chars().take(19).collect(), changes: Vec::new() });
            open = true;
        } else if rest.starts_with("transaction completed") {
            open = false;
        } else if open {
            for verb in ["installed", "upgraded", "removed", "reinstalled", "downgraded"] {
                if let (Some(what), Some(change)) = (rest.strip_prefix(verb).and_then(|rest| rest.strip_prefix(' ')), all.last_mut()) {
                    let (name, versions) = what.split_once(' ').unwrap_or((what, ""));
                    change.changes.push(Change { verb, name: name.to_string(), versions: versions.trim_matches(['(', ')']).to_string() });
                }
            }
        }
    }
    all.retain(|transaction| !transaction.changes.is_empty());
    all
}

impl Transaction {
    /// The change in a few words: upgraded 14: linux, mesa +12 more.
    pub fn in_short(&self) -> String {
        let mut parts = Vec::new();
        for verb in ["installed", "upgraded", "removed", "reinstalled", "downgraded"] {
            let names: Vec<&str> = self.changes.iter().filter(|change| change.verb == verb).map(|change| change.name.as_str()).collect();
            match names.len() {
                0 => {}
                1..=3 => parts.push(format!("{verb} {}", names.join(", "))),
                count => parts.push(format!("{verb} {count}: {}, {} +{} more", names[0], names[1], count - 2)),
            }
        }
        parts.join("; ")
    }
}

#[cfg(test)]
pub const SAMPLE: &str = "\
[2026-11-13T10:15:00+0100] [PACMAN] Running 'pacman -S --needed -- os-prober'
[2026-11-13T10:15:02+0100] [ALPM] transaction started
[2026-11-13T10:15:02+0100] [ALPM] installed os-prober (1.83-1)
[2026-11-13T10:15:03+0100] [ALPM] transaction completed
[2026-11-13T10:15:03+0100] [ALPM] running '30-systemd-update.hook'...
[2026-11-14T09:00:00+0100] [PACMAN] starting full system upgrade
[2026-11-14T09:00:10+0100] [ALPM] transaction started
[2026-11-14T09:00:11+0100] [ALPM] upgraded glibc (2.42-1 -> 2.42-2)
[2026-11-14T09:00:12+0100] [ALPM] upgraded linux (6.17.7.arch1-1 -> 6.17.8.arch1-1)
[2026-11-14T09:00:13+0100] [ALPM] upgraded kitty (0.43.1-1 -> 0.44.0-1)
[2026-11-14T09:00:13+0100] [ALPM] upgraded zsh (5.9-5 -> 5.9-6)
[2026-11-14T09:00:14+0100] [ALPM] removed os-prober (1.83-1)
[2026-11-14T09:00:15+0100] [ALPM] transaction completed
[2026-11-14T09:30:00+0100] [ALPM] transaction started
[2026-11-14T09:30:00+0100] [ALPM] transaction completed
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_becomes_its_changes_of_packages() {
        let all = transactions(SAMPLE);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].when, "2026-11-13 10:15:02");
        assert_eq!(all[0].changes, [Change { verb: "installed", name: "os-prober".into(), versions: "1.83-1".into() }]);
        assert_eq!(all[1].changes[1].versions, "6.17.7.arch1-1 -> 6.17.8.arch1-1");
        assert_eq!(all[1].changes.len(), 5);
    }

    #[test]
    fn a_change_is_told_in_a_few_words() {
        let all = transactions(SAMPLE);
        assert_eq!(all[0].in_short(), "installed os-prober");
        assert_eq!(all[1].in_short(), "upgraded 4: glibc, linux +2 more; removed os-prober");
    }
}
