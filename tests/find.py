#!/usr/bin/env python3
"""Check the file search: the index the installer leaves, and kavat-find on the desktop.

usage: tests/find.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. Before anybody logs in, the index of the whole disk has to be
there already, with the user's folder in it and without the snapshots or the installer, and
a change of packages has to bring it up to date. Then it logs in and opens the search with
Super+Shift+F: a file of the user's made a moment ago is found by name and opens in the
editor, a file of the system is found in the index and its path is copied, and a word
inside a file opens the editor at its line. Last, zsh has to show its completions in the
same picker.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes. Pictures of the screen are left in out/test/.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
HOME = f'/home/{USER}'
PACKAGE = 'os-prober'  # taken out to see the index follow; nothing else needs it
NOTE = 'Documents/quarterly kavatneedle.txt'  # a file of the user's, made after the index
WORD = 'kavathaystack'  # what its second line says
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME={HOME} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
WINDOWS = 'hyprctl --instance 0 clients'


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('find', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    def until(what, command, wanted, timeout=20):
        """Check that a command of the user's comes to say something within a few seconds."""
        deadline = time.time() + timeout
        _, found = user(command)
        while wanted not in found and time.time() < deadline:
            time.sleep(0.5)
            _, found = user(command)
        return checks.report(wanted in found, what, f'wanted {wanted}, found {found}')

    def open_search(turns=0):
        """Open the search from the keyboard, and go round its sources."""
        machine.screen.press('super', 'shift', 'f')
        opened = until('Super+Shift+F opens the search in a window over the desktop' if not turns else 'the search opens again, the same size',
                       f'{WINDOWS} | grep -B6 "class: kavat-find" | grep -E "size|floating" | tr -d "\\n\\t"', 'size: 1344,756floating: 1')
        time.sleep(1.5)
        for _ in range(turns):
            machine.screen.press('ctrl', 'f')
            time.sleep(1.5)
        return opened

    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)

        print('the index of the whole disk', flush=True)
        status, output = machine.run('stat -c %U:%G:%a /var/lib/plocate/plocate.db && plocate -c /etc/os-release')
        checks.report(status == 0 and output.split()[-1] != '0', 'the installer left the index built', output)
        status, output = machine.run(f'plocate -c {HOME}/.zshrc; plocate -c /.snapshots/; plocate -c /usr/bin/calamares')
        counts = output.split()
        checks.report(len(counts) == 3 and counts[0] != '0', "the user's folder is in it", output)
        checks.report(counts[1:] == ['0', '0'], 'the snapshots and the installer are not', output)
        status, output = machine.run('systemctl is-active plocate-updatedb.timer; '
                                     'systemctl show -p ExecStart plocate-updatedb.service | grep -c /usr/lib/kavat/find-index')
        checks.report(output.split() == ['active', '1'], 'it is built again once a day, the way this disk needs', output)
        status, output = machine.run(f'plocate -c /usr/bin/{PACKAGE} && pacman -Rdd --noconfirm {PACKAGE}', 120)
        checks.report(status == 0, f'{PACKAGE} is in the index, and is taken out of the system', output)
        status, output = machine.wait_until(f'test "$(plocate -c /usr/bin/{PACKAGE})" = 0', 60)
        checks.report(status == 0, 'a change of packages brings the index up to date', output)

        print('the search on the desktop', flush=True)
        log_in(machine, PASSWORD)
        check_desktop(machine, checks, USER)
        user(f'mkdir -p ~/Documents && printf "first line\\nthe {WORD} is here\\n" > ~/"{NOTE}"')
        status, output = user('hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)

        if open_search():
            machine.screen.type('kavatneedle')
            time.sleep(1.5)
            machine.screen.picture(f'{pictures}/find-home.png')
            machine.screen.press('enter')
            until('a file made a moment ago is found by name, and Enter opens it in the editor',
                  'pgrep -a nvim', f'{HOME}/{NOTE}')
            until('the search closes once something is picked',
                  f'{WINDOWS} | grep -c "class: kavat-find"; echo done', '0\ndone')
            user('pkill -x nvim')
            time.sleep(1)

        if open_search(turns=1):
            machine.screen.type('/etc/os-release')
            time.sleep(2)
            machine.screen.picture(f'{pictures}/find-disk.png')
            machine.screen.press('ctrl', 'y')
            until("a file of the system is found in the index, and Ctrl+Y copies its path",
                  'echo "copied $(wl-paste -n) and no more"', 'copied /etc/os-release and no more')

        if open_search(turns=2):
            machine.screen.type(WORD)
            time.sleep(2)
            machine.screen.picture(f'{pictures}/find-text.png')
            machine.screen.press('enter')
            until('a word inside a file is found, and Enter opens the editor at its line',
                  'pgrep -a nvim', f'+2 {HOME}/{NOTE}')
            user('pkill -x nvim')

        print('the picker in zsh', flush=True)
        status, output = user('zsh -ic \'bindkey "^I"; bindkey "^G"; print -r -- $FZF_DEFAULT_OPTS_FILE; cat $FZF_DEFAULT_OPTS_FILE\' 2>&1')
        checks.report('fzf-tab-complete' in output, 'Tab in zsh shows the completions in the picker', output)
        checks.report('kavat-find-text' in output, 'and Ctrl+G looks inside files', output)
        checks.report('/kavat/theme/current/fzf/options' in output and '--color=' in output and '{{' not in output,
                      "the picker's colours come from the theme", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/find-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
