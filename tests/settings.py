#!/usr/bin/env python3
"""Check the settings panel: it is there from the first login, and what is pressed in it
changes the system.

usage: tests/settings.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. It logs in as a user who never opened the panel: the shell
has to have the system's plugin on, and must not be set to look for plugins on the network.
Super+T has to open the panel on the page of the look; pressing Light there, and then a
palette, has to change the theme of the whole system, as kavat-theme set does.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes. Pictures of the screen are left in out/test/.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, near, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
HOME = f'/home/{USER}'
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME={HOME} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
# where things are in the panel, on a 1920x1080 screen
TITLE_SIDE = (1500, 160)  # a point of the panel with nothing on it
LIGHT = (702, 481)        # the Light half of Mode
NORD = (684, 385)         # the card of the Nord palette
PANEL_DARK = (0x15, 0x18, 0x21)  # the panel's background with Kavat's dark theme


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('settings', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    def until(what, command, wanted, timeout=20):
        deadline = time.time() + timeout
        _, found = user(command)
        while wanted not in found and time.time() < deadline:
            time.sleep(0.5)
            _, found = user(command)
        return checks.report(wanted in found, what, f'wanted {wanted}, found {found}')

    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        log_in(machine, PASSWORD)
        check_desktop(machine, checks, USER)

        print('the settings panel', flush=True)
        until("the shell has the system's settings plugin on, from the first login",
              'kavat-shell msg plugins list', 'kavat/settings [kavat] 0.1.0 enabled')
        status, output = user('grep -c "^auto_update = \\"none\\"" ~/.config/kavat-shell/kavat-theme.toml')
        checks.report(output == '1', 'and is not set to look for plugins on the network', output)

        # the shell starts its plugins a little after the login
        time.sleep(5)
        before = machine.screen.pixel(*TITLE_SIDE)
        # The keys open the panel and close it, so they are pressed once and the panel waited
        # for; only when nothing came are they pressed again, for the key the test's screen
        # loses now and then.
        for _ in range(2):
            machine.screen.press('super', 't')
            deadline = time.time() + 12
            after = machine.screen.pixel(*TITLE_SIDE)
            while near(before, after, 12) and time.time() < deadline:
                time.sleep(0.5)
                after = machine.screen.pixel(*TITLE_SIDE)
            if not near(before, after, 12):
                break
        time.sleep(2)
        checks.report(not near(before, after, 12) and near(after, PANEL_DARK, 40), 'Super+T opens the panel',
                      f'that point was {before} and is {after}')
        machine.screen.picture(f'{pictures}/settings.png')

        machine.screen.click(*LIGHT)
        until('pressing Light there turns the whole system light', 'kavat-theme get mode', 'light', 8)
        time.sleep(2)  # the panel takes no press while a change is being applied
        machine.screen.click(*NORD)
        until('and pressing a palette changes it', 'kavat-theme get palette', 'nord', 8)
        time.sleep(2)
        machine.screen.picture(f'{pictures}/settings-light.png')
        status, output = user('hyprctl --instance 0 getoption general:col.active_border')
        checks.report('88c0d0' in output or '5e81ac' in output or 'gradient' in output, 'Hyprland wears the new palette', output)

        user('kavat-shell msg panel-open kavat/settings:panel windows')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/settings-windows.png')

        print('the settings each app keeps in its own file', flush=True)
        status, output = user('kavat-conf set terminal cursor_shape beam && kavat-conf set terminal window_padding_width 6 && '
                              'kavat-conf show terminal | head -n 3; cat ~/.config/kitty/kitty.conf')
        checks.report(status == 0 and 'cursor_shape beam' in output and output.count('window_padding_width 6') == 2,
                      "a setting of the terminal is changed in Kitty's own file, and read back from it", output)
        status, output = user('kavat-conf set files mgr.show_hidden true && grep -c "^show_hidden = true" ~/.config/yazi/yazi.toml; '
                              'grep -c "^# Your Yazi settings" ~/.config/yazi/yazi.toml; kavat-conf set files mgr.sort_by backwards 2>&1')
        checks.report(output.splitlines()[:2] == ['1', '1'] and 'mgr.sort_by takes' in output,
                      "one of Yazi's changes only its own line, and a value it cannot take is refused", output)
        status, output = user('kavat-conf show lock; kavat-conf set lock lock_after 5 && kavat-conf show lock && pgrep -c -x hypridle')
        checks.report(output.split() == ['lock_after', '10', 'screen_off_after', '15', 'lock_after', '5', 'screen_off_after', '15', '1'],
                      'the screen locks after ten minutes unless another time is set, and hypridle takes the new one', output)
        status, output = user('kavat-conf show language; kavat-conf set language keyboard_first es && '
                              'hyprctl --instance 0 getoption input:kb_layout | head -n 1; kavat-conf set language keyboard_first us && '
                              'hyprctl --instance 0 getoption input:kb_layout | head -n 1; hyprctl --instance 0 getoption input:kb_options | head -n 1')
        lines = output.splitlines() + [''] * 4
        checks.report(lines[0] == 'keyboard_first us' and 'es,us' in lines[1] and 'us,es' in lines[2] and 'grp:alt_shift_toggle' in lines[3],
                      'the keyboard is the one of the installation, with the other behind it, and the first can be changed', output)
        # every page that is built has to draw: one that fails leaves a line in the shell's log
        pages = 'wallpaper accessibility windows displays sound network terminal editor files notes bar packages boot login language'
        status, output = user(f'for page in {pages}; do kavat-shell msg panel-open kavat/settings:panel $page > /dev/null; sleep 1.2; done; '
                              'grep -c -i "kavat/settings.*error\\|luau.*error\\|panel.luau:" ~/.cache/kavat-shell/kavat-shell.log', 60)
        checks.report(output == '0', 'each of the pages that are built is drawn without a failure', output)
        machine.screen.picture(f'{pictures}/settings-language.png')
        user('kavat-shell msg panel-open kavat/settings:panel keys')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/settings-keys.png')
        user('kavat-shell msg panel-close kavat/settings:panel')

        print('the lock screen', flush=True)
        machine.screen.press('super', 'shift', 'l')
        until('Super+Shift+L locks the screen', 'pgrep -c -x hyprlock', '1')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/lock.png')
        machine.screen.type('not-the-password')
        machine.screen.press('enter')
        time.sleep(4)
        status, output = user('pgrep -c -x hyprlock')
        checks.report(output == '1', 'a wrong password leaves it locked', output)
        machine.screen.type(PASSWORD)
        machine.screen.press('enter')
        until('and the right one opens it', 'pgrep -c -x hyprlock; echo open', '0\nopen')
        status, output = user('hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/settings*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
