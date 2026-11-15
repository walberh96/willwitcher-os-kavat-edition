//! Which of two versions of a package is the newer, decided as pacman decides it.

use std::cmp::Ordering;

/// Compares one part of a version, as pacman's own vercmp does: runs of digits as
/// numbers, runs of letters as words, and whatever is between them only separates.
fn compare_part(one: &str, two: &str) -> Ordering {
    let (mut one, mut two) = (one.as_bytes(), two.as_bytes());
    while !one.is_empty() && !two.is_empty() {
        let skip = |text: &[u8]| text.iter().take_while(|c| !c.is_ascii_alphanumeric()).count();
        let (gap_one, gap_two) = (skip(one), skip(two));
        (one, two) = (&one[gap_one..], &two[gap_two..]);
        if one.is_empty() || two.is_empty() {
            break;
        }
        if gap_one != gap_two {
            return gap_one.cmp(&gap_two);
        }
        let digits = one[0].is_ascii_digit();
        let run = |text: &[u8]| text.iter().take_while(|c| if digits { c.is_ascii_digit() } else { c.is_ascii_alphabetic() }).count();
        let (run_one, run_two) = (run(one), run(two));
        // a number is newer than a word
        if run_two == 0 {
            return if digits { Ordering::Greater } else { Ordering::Less };
        }
        let (mut piece_one, mut piece_two) = (&one[..run_one], &two[..run_two]);
        (one, two) = (&one[run_one..], &two[run_two..]);
        if digits {
            let bare = |piece: &[u8]| piece.iter().take_while(|c| **c == b'0').count();
            (piece_one, piece_two) = (&piece_one[bare(piece_one)..], &piece_two[bare(piece_two)..]);
            if piece_one.len() != piece_two.len() {
                return piece_one.len().cmp(&piece_two.len());
            }
        }
        if piece_one != piece_two {
            return piece_one.cmp(piece_two);
        }
    }
    match (one.first(), two.first()) {
        (None, None) => Ordering::Equal,
        // letters left over never beat nothing: 1.0a is older than 1.0
        (None, Some(next)) if !next.is_ascii_alphabetic() => Ordering::Less,
        (Some(next), _) if next.is_ascii_alphabetic() => Ordering::Less,
        _ => Ordering::Greater,
    }
}

/// Compares two whole versions, like 1:2.4.1-3: the epoch, then the version, then the release.
pub fn compare(one: &str, two: &str) -> Ordering {
    let split = |text: &str| -> (u64, String, Option<String>) {
        let (epoch, rest) = match text.split_once(':') {
            Some((epoch, rest)) if !epoch.is_empty() && epoch.chars().all(|c| c.is_ascii_digit()) => (epoch.parse().unwrap_or(0), rest),
            _ => (0, text),
        };
        match rest.rsplit_once('-') {
            Some((version, release)) => (epoch, version.to_string(), Some(release.to_string())),
            None => (epoch, rest.to_string(), None),
        }
    };
    let ((epoch_one, version_one, release_one), (epoch_two, version_two, release_two)) = (split(one), split(two));
    let by_release = match (release_one, release_two) {
        (Some(one), Some(two)) => compare_part(&one, &two),
        _ => Ordering::Equal,
    };
    epoch_one.cmp(&epoch_two).then(compare_part(&version_one, &version_two)).then(by_release)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_ordered_as_pacman_orders_them() {
        for (older, newer) in [
            ("1.0-1", "1.0-2"),
            ("1.9-1", "1.10-1"),
            ("1.0a-1", "1.0-1"),
            ("1.0-1", "1.0.1-1"),
            ("2.0-1", "1:0.1-1"),
            ("6.17.7.arch1-1", "6.17.8.arch1-1"),
            ("1.0alpha-1", "1.0beta-1"),
            ("20250613.12fe085f-5", "20250613.12fe085f-10"),
            ("1.0rc1-1", "1.0-1"),
            ("007-1", "8-1"),
        ] {
            assert_eq!(compare(older, newer), Ordering::Less, "{older} {newer}");
            assert_eq!(compare(newer, older), Ordering::Greater, "{newer} {older}");
        }
        assert_eq!(compare("1.0-1", "1.0-1"), Ordering::Equal);
        assert_eq!(compare("1.01-1", "1.1-1"), Ordering::Equal);
        assert_eq!(compare("1.0", "1.0-3"), Ordering::Equal);
    }
}
