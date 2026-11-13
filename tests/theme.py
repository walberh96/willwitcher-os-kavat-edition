#!/usr/bin/env python3
"""Change the theme while the desktop is running, and check that every app follows.

usage: tests/theme.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. It logs in as a user who never touched the theme, and checks
that the desktop already wears the system's own: Hyprland's borders, Kitty and the shell.
Then it changes the mode, the palette and the accent with kavat-theme, and checks that each
app took the change in under a second, without any of them starting again. A change that
cannot be applied has to leave everything as it was. This is half of the gate of phase 3.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes. Pictures of the screen are left in out/test/.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
KITTY = '/tmp/kitty-test'  # where the test's Kitty listens
BAR = (300, 17)  # a point of the shell's bar with nothing on it, on a 1920x1080 screen
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME=/home/{USER} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
RUNNING = 'pgrep -x Hyprland; pgrep -x noctalia; pgrep -x kitty'


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('theme', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    def border(which):
        """A border colour of Hyprland, as rrggbb."""
        _, output = user(f'hyprctl --instance 0 getoption general:col.{which}_border')
        return output.split()[2][2:] if output.startswith('gradient data: ') else output

    def kitty(name):
        _, output = user(f'kitty @ --to unix:{KITTY} get-colors')
        found = [line.split()[1] for line in output.splitlines() if line.split()[:1] == [name]]
        return found[0] if found else output

    def follows(what, read, wanted, timeout=5):
        """Check that something reaches a value within a few seconds of a change."""
        deadline = time.time() + timeout
        found = read()
        while found != wanted and time.time() < deadline:
            time.sleep(0.3)
            found = read()
        checks.report(found == wanted, what, f'wanted {wanted}, found {found}')

    def light_at(point):
        """How light a point of the screen is, from 0 to 255."""
        return sum(machine.screen.pixel(*point)) // 3

    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        machine.wait_until('systemctl is-active lightdm', 60)
        time.sleep(5)  # the login screen takes a moment to listen
        machine.screen.type(PASSWORD)
        machine.screen.press('enter')
        check_desktop(machine, checks, USER)

        print('the theme of a user who never chose one', flush=True)
        status, output = user('test ! -e ~/.config/kavat/theme.toml && kavat-theme get mode')
        checks.report(status == 0 and output == 'dark', 'the user has no theme file, and the mode is dark', output)
        follows("Hyprland's active border is the accent of Kavat", lambda: border('active'), 'f2a93b')
        follows("Hyprland's other borders are Kavat's border colour", lambda: border('inactive'), '2a2f3f')
        follows('the shell uses the palette of the theme', lambda: user('noctalia msg color-scheme-get')[1], 'custom Kavat')
        follows('the shell is in dark mode', lambda: user('noctalia msg theme-mode-get')[1], 'dark')
        user(f'hyprctl --instance 0 eval \'hl.exec_cmd("kitty -o allow_remote_control=yes --listen-on unix:{KITTY}")\'')
        status, output = machine.wait_until(f'test -S {KITTY}', 30)
        checks.report(status == 0, 'Kitty opens', output)
        follows("Kitty's background is Kavat's dark surface", lambda: kitty('background'), '#151821')
        follows("Kitty's text is Kavat's light text", lambda: kitty('foreground'), '#e8e6df')
        time.sleep(2)
        _, output = user('hyprctl --instance 0 clients | grep -c "^Window"; hyprctl --instance 0 layers | grep -c -i -E "wizard|setup"')
        checks.report(output == '1\n0', 'nothing else opens on a first login: no news, no first-run dialog', output)
        dark = light_at(BAR)
        checks.report(dark < 60, "the shell's bar is dark on the screen", f'lightness {dark} of 255')
        machine.screen.picture(f'{pictures}/theme-dark.png')

        print('changing to light mode', flush=True)
        _, before = machine.run(RUNNING)
        status, output = user('start=$(date +%s%N); kavat-theme set mode light; status=$?; '
                              'echo $(( ($(date +%s%N) - start) / 1000000 )); exit $status')
        took = int(output) if output.isdigit() else None
        checks.report(status == 0 and took is not None and took < 1000,
                      'kavat-theme changes the mode in under a second', f'{output} ms')
        follows("Hyprland's borders turn to the light border colour", lambda: border('inactive'), 'd2cbba', 2)
        follows("Kitty's background turns to paper", lambda: kitty('background'), '#fbf9f4', 2)
        follows("Kitty's text turns dark", lambda: kitty('foreground'), '#1d1f26', 2)
        follows('the shell turns to light mode', lambda: user('noctalia msg theme-mode-get')[1], 'light', 2)
        follows('GTK apps are asked for their light side',
                lambda: user('gsettings get org.gnome.desktop.interface color-scheme')[1], "'prefer-light'", 2)
        time.sleep(1)
        light = light_at(BAR)
        checks.report(light > 200, "the shell's bar is light on the screen", f'lightness {light} of 255')
        _, after = machine.run(RUNNING)
        checks.report(before == after and len(before.split()) >= 3, 'no app had to start again', f'{before}\nthen\n{after}')
        status, output = user('cat ~/.config/kavat/theme.toml')
        checks.report(output == 'mode = "light"', "the user's theme file holds only what changed", output)
        machine.screen.picture(f'{pictures}/theme-light.png')

        print('changing palette and accent', flush=True)
        status, output = user('kavat-theme set palette nord accent green mode dark')
        checks.report(status == 0, 'kavat-theme changes palette, accent and mode at once', output)
        follows("Hyprland's active border is Nord's green", lambda: border('active'), 'a3be8c', 2)
        follows("Kitty's background is Nord's dark surface", lambda: kitty('background'), '#2e3440', 2)
        follows('the shell is back in dark mode', lambda: user('noctalia msg theme-mode-get')[1], 'dark', 2)
        time.sleep(1)
        machine.screen.picture(f'{pictures}/theme-nord.png')

        print('changes that cannot be applied', flush=True)
        _, current = user('readlink ~/.local/state/kavat/theme/current')
        status, output = user('kavat-theme set mode sepia')
        checks.report(status != 0 and 'mode takes dark, light' in output, 'a value that does not exist is refused', output)
        user('mkdir -p ~/.config/kavat/templates/broken && cd ~/.config/kavat/templates/broken && '
             'printf \'[[file]]\\nfrom = "x.conf"\\n\' > template.toml && echo "colour {{ acent }}" > x.conf')
        status, output = user('kavat-theme set mode light')
        checks.report(status != 0 and 'no token called acent' in output, 'a template that fails stops the change', output)
        status, output = user('kavat-theme get mode; readlink ~/.local/state/kavat/theme/current')
        checks.report(output == f'dark\n{current}', 'and the theme stays whole, as it was', output)
        checks.report(border('active') == 'a3be8c' and kitty('background') == '#2e3440',
                      'the apps still wear it', f"{border('active')} {kitty('background')}")
        user('rm -r ~/.config/kavat/templates/broken')
        status, output = user('hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration still loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/theme-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
