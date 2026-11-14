//! Reading a package of the AUR before it is approved: what in its build files deserves a
//! second look, and where it downloads from. This marks lines; it decides nothing. A person
//! reads the package and approves it.

/// Why a line of a PKGBUILD or of an install script is marked, if it is.
pub fn suspicion(line: &str) -> Option<&'static str> {
    let code = line.split_once('#').map_or(line, |(code, _)| code);
    let has = |words: &[&str]| words.iter().any(|word| code.contains(word));
    let piped_to_shell = ["| sh", "|sh", "| bash", "|bash", "| zsh", "| sudo", "|sudo"];
    if has(&["curl", "wget", "fetch"]) && has(&piped_to_shell) {
        Some("downloads something and runs it")
    } else if has(&piped_to_shell) || has(&["eval ", "base64 -d", "base64 --decode", "xxd -r", "python -c", "python3 -c", "perl -e"]) {
        Some("runs text that is put together on the spot")
    } else if has(&["sudo ", "su -c", "pkexec", "doas "]) {
        Some("asks for root while it builds")
    } else if has(&["chmod +s", "chmod u+s", "chmod 4", "chmod 6", "setcap "]) {
        Some("gives a program more rights than who runs it")
    } else if has(&["/etc/sudoers", "/etc/passwd", "/etc/shadow", "authorized_keys", ".ssh/", "/etc/pam.d", "/etc/polkit-1"]) {
        Some("touches who may do what on the system")
    } else if has(&["$HOME", "${HOME}", "~/", "/home/"]) {
        Some("reaches into a home folder")
    } else if has(&["curl ", "wget ", "git clone", "nc ", "ncat ", "/dev/tcp/"]) {
        Some("uses the network outside its listed sources")
    } else if has(&["'SKIP'", "\"SKIP\"", "(SKIP)"]) {
        Some("a file is taken without checking its checksum")
    } else if code.trim_start().starts_with("install=") {
        Some("runs a script as root when it is installed, upgraded or removed")
    } else if has(&["systemctl enable", "systemctl start", "systemctl --now"]) {
        Some("starts a service by itself")
    } else if has(&["rm -rf /", "mkfs", "dd of=/dev", "> /dev/sd", "> /dev/nvme"]) {
        Some("can destroy data")
    } else {
        None
    }
}

/// The hosts that the text names in addresses, in the order they appear, each once.
pub fn hosts(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for marker in ["://", "git@"] {
        for (at, _) in text.match_indices(marker) {
            let rest = &text[at + marker.len()..];
            // user@host and host:port name the same host
            let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || ".-@:".contains(c))).unwrap_or(rest.len());
            let host = rest[..end].rsplit('@').next().unwrap_or("").split(':').next().unwrap_or("").trim_matches('.').to_lowercase();
            if host.contains('.') && !found.contains(&host) {
                found.push(host);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_careful_reader_would_stop_at_is_marked() {
        for (line, why) in [
            ("  curl -fsSL https://get.example.org | sh", "downloads something and runs it"),
            ("  eval \"$(echo aGk= | base64 -d)\"", "runs text that is put together on the spot"),
            ("  sudo cp x /usr/bin", "asks for root while it builds"),
            ("  chmod +s \"$pkgdir/usr/bin/x\"", "gives a program more rights than who runs it"),
            ("  cat ~/.ssh/id_ed25519", "touches who may do what on the system"),
            ("  cp -r $HOME/.config .", "reaches into a home folder"),
            ("  wget https://example.org/extra.tar", "uses the network outside its listed sources"),
            ("sha256sums=('SKIP')", "a file is taken without checking its checksum"),
            ("install=x.install", "runs a script as root when it is installed, upgraded or removed"),
            ("  systemctl enable --now x.service", "starts a service by itself"),
        ] {
            assert_eq!(suspicion(line), Some(why), "{line}");
        }
    }

    #[test]
    fn an_ordinary_build_is_not_marked() {
        let text = "pkgname=x\npkgver=1.0\nsource=(\"https://example.org/x-$pkgver.tar.gz\")\nsha256sums=('0a1b')\n\
                    build() {\n  cd x-$pkgver\n  make  # curl | sh would be marked, a comment is not\n}\n\
                    package() {\n  install -Dm755 x \"$pkgdir/usr/bin/x\"\n}\n";
        assert_eq!(text.lines().filter_map(suspicion).count(), 0);
    }

    #[test]
    fn the_hosts_a_package_downloads_from_are_found() {
        let text = "url=https://example.org/x\nsource=(\"x::git+https://GitHub.com/a/b.git#tag=1\" 'https://cdn.example.net:8443/x.tar'\n \"git+ssh://git@git.example.com/x.git\")";
        assert_eq!(hosts(text), ["example.org", "github.com", "cdn.example.net", "git.example.com"]);
    }
}
