#!/usr/bin/env python3
"""Check what keeps the work safe: snapshots of the home folders, the history of the
settings, and backups off the computer.

usage: tests/backup.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. The installed system has to have the place for the snapshots
of /home and take one every hour; a file changed after a snapshot has to be found in it,
by its owner, and brought back. A change of the settings has to be in their history, and
undoing it has to change the system back. A backup to a folder standing in for an
external disk has to be encrypted, hold the home folder, give a file back, and be made by
itself when asked.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
HOME = f'/home/{USER}'
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME={HOME} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
VERSIONS = '/usr/lib/kavat/previous-versions --list'
PLACE = '/var/tmp/disk/backup'  # a folder standing in for an external disk


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('backup', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        log_in(machine, PASSWORD)
        check_desktop(machine, checks, USER)

        print('snapshots of the home folders', flush=True)
        status, output = machine.run('btrfs subvolume show /home/.snapshots > /dev/null && echo place; systemctl is-enabled snapper-timeline.timer; '
                                     'grep -c "^SNAPPER_CONFIGS=.*home" /etc/conf.d/snapper; snapper -c home get-config | grep -c "TIMELINE_CREATE.*yes"')
        checks.report(output.split() == ['place', 'enabled', '1', '1'], 'the installed system has the place for them, and takes one every hour', output)
        user('echo first > ~/kept.txt')
        machine.run('snapper -c home create -c timeline -d timeline')
        user('echo second > ~/kept.txt; mkdir ~/work; echo draft > ~/work/draft.txt')
        machine.run('snapper -c home create -c timeline -d timeline')
        user('echo third > ~/kept.txt; rm ~/work/draft.txt')
        status, output = user(f'{VERSIONS} ~/kept.txt | cut -f1,3 | tr "\\t\\n" "  "; echo; {VERSIONS} ~/work/draft.txt | cut -f1,3; cat /home/.snapshots/1/snapshot/{USER}/kept.txt; '
                              f'cat /home/.snapshots/2/snapshot/{USER}/work/draft.txt')
        checks.report(output.split('\n') == ['2 different 1 different ', '2\tgone now', 'first', 'draft'],
                      'a file changed or deleted since is found in them by its owner, newest first', output)
        status, output = machine.run('snapper -c root list | grep -c timeline; grub-mkconfig 2> /dev/null | grep -c "/home/.snapshots"')
        checks.report(output.split() == ['0', '0'], "they are not the system's snapshots, and are not in the boot menu", output)

        print('the history of the settings', flush=True)
        status, output = user('kavat-history save "As installed" > /dev/null; kavat-theme set mode light && kavat-history save "Theme: mode light" && '
                              'kavat-conf set terminal cursor_shape beam && kavat-history save "Terminal: cursor_shape beam" > /dev/null && kavat-history list | cut -f3')
        checks.report(output.split('\n') == ['kavat-history: kept.', 'Terminal: cursor_shape beam', 'Theme: mode light', 'As installed'],
                      'each change of the settings is kept, with what it was', output)
        status, output = user('kavat-history undo && kavat-conf show terminal | grep cursor_shape; kavat-history undo > /dev/null && kavat-theme get mode && '
                              'kavat-shell msg theme-mode-get')
        checks.report(output.split('\n') == ['kavat-history: undone: Terminal: cursor_shape beam', 'cursor_shape block', 'dark', 'dark'],
                      'undoing goes back one change at a time, and the system follows', output)
        status, output = user('mark=$(kavat-history list | grep "\tTerminal: cursor_shape beam$" | cut -f1); kavat-history back $mark > /dev/null && kavat-theme get mode && '
                              'kavat-conf show terminal | grep cursor_shape; kavat-history export /tmp/settings.tar.gz > /dev/null && tar -tzf /tmp/settings.tar.gz | grep -c "kavat/theme.toml"; '
                              'systemctl --user is-active kavat-history.timer')
        checks.report(output.split('\n') == ['light', 'cursor_shape beam', '1', 'active'],
                      'any change can be gone back to, the settings can be taken out as one file, and the history keeps itself', output)
        user('kavat-theme set mode dark; kavat-conf set terminal cursor_shape block')

        print('backups off the computer', flush=True)
        machine.run(f'mkdir -p /var/tmp/disk && chown {USER}: /var/tmp/disk')
        status, output = user(f'kavat-backup run 2>&1 | head -n 1; echo "the plan is in the drawer" > ~/work/plan.txt; kavat-backup to {PLACE} | tail -n 1; '
                              f'stat -c %a ~/.config/kavat/backup.key; kavat-backup run && kavat-backup | tr "\\n" " "', 180)
        checks.report('no place is chosen' in output and f'the copies go to {PLACE}' in output and '\n600\n' in output
                      and f'place: {PLACE} every: never last: ' in output and output.rstrip().endswith('done'),
                      'a copy is made in the place chosen, with a key of its own', output)
        status, output = user(f'grep -r -l "plan is in the drawer" {PLACE} | wc -l; rm ~/work/plan.txt; kavat-backup restore latest ~/work/plan.txt /tmp/back > /dev/null && '
                              f'cat /tmp/back{HOME}/work/plan.txt; kavat-backup list | grep -c "{HOME}"; kavat-history save > /dev/null; '
                              'git --git-dir ~/.local/share/kavat/history.git ls-files | grep -c backup.key', 120)
        checks.report(output.split('\n') == ['0', 'the plan is in the drawer', '1', '0'],
                      'nothing can be read at the place without the key, a file comes back from the copy, and the key is not in the history', output)
        status, output = user('kavat-backup every daily && systemctl --user is-enabled kavat-backup.timer && kavat-backup | grep every; '
                              'kavat-backup every never; systemctl --user is-enabled kavat-backup.timer 2>&1 | head -n 1')
        lines = output.split('\n')
        checks.report(lines[:2] == ['enabled', 'every: daily'] and lines[2:3] != ['enabled'], 'the copies are made by themselves when asked, and stop when asked', output)

        user('kavat-shell msg panel-open kavat/settings:panel backup > /dev/null')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/backup-page.png')
        status, output = user('kavat-shell msg panel-close kavat/settings:panel > /dev/null; hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/backup-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
