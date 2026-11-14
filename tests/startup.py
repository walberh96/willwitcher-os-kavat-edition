#!/usr/bin/env python3
"""From the power button to the desktop: check that the boot menu, the loading screen and
the login screen wear the theme of the system, and that the menu's entries are the system's.

usage: tests/startup.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. It looks at the screen while the machine boots: the boot menu
has to show the theme's colours and the system's entries in their order, the loading screen
its logo over the same picture, and the login screen the same again, and logging in there
has to reach the desktop. Then it gives the three another look with kavat-theme boot, boots
again and checks that they changed, and boots the menu's Recovery entry, which has to end
in the recovery console and not count as a boot that failed.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about three minutes. Pictures of the screen are left in out/test/.
"""
import os
import re
import sys
import time

from machine import (AVATAR, BOOT_TIMEOUT, CORNER, GLOW, Checks, Machine, check_desktop, log_in, near, over_installed,
                     print_journal, wait_for_login_screen)

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
ENTRIES = ['Willwitcher-OS Kavat Edition', 'Fallback kernel', 'Safe mode', 'Snapshots', 'Recovery',
           'Firmware settings', 'Other systems']
TOKENS = '/var/lib/kavat/theme/current/greeter/tokens.css'  # the colours in force, as the login screen reads them
# The boot menu, on the 1280x800 screen that the machine's firmware gives GRUB: the left
# edge of the box around the chosen entry, which is the first one, and a point with nothing.
MENU_BOX = [(x, 248) for x in range(112, 124)]
MENU_EMPTY = (40, 40)
# The loading screen, on a 1920x1080 screen: the middle of the logo, and a point with nothing.
LOGO = (960, 497)


def colour(text):
    """(red, green, blue) of a colour written as #rrggbb."""
    return tuple(int(text[at:at + 2], 16) for at in (1, 3, 5))


def colours(machine):
    """The theme's background and accent, read from the file the login screen reads."""
    _, text = machine.run(f'cat {TOKENS}')
    found = dict(re.findall(r'--([a-z0-9-]+): (#[0-9a-f]{6});', text))
    return colour(found.get('bg', '#000000')), colour(found.get('accent', '#000000'))


def wait_for_menu(machine, timeout=60):
    """The colours of the boot menu once it is on the screen: (the empty point, the box's edge)."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            size, (empty, *box) = machine.screen.pixels(MENU_EMPTY, *MENU_BOX)
        except OSError:
            size = None
        if size == (1280, 800):
            # the box's edge is the point of that row least like the background
            edge = max(box, key=lambda point: sum(abs(a - b) for a, b in zip(point, empty)))
            if not near(edge, empty):
                return empty, edge
        time.sleep(0.2)
    return None


def stop_countdown(machine):
    """Stop the menu's countdown without choosing anything: down, and back up."""
    machine.screen.press('down')
    machine.screen.press('up')


def watch_loading(machine, background, accent, picture, timeout=90):
    """True if, on the way to the login screen, the loading screen shows its logo over the picture."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            size, (logo, corner, avatar) = machine.screen.pixels(LOGO, CORNER, AVATAR)
        except OSError:
            continue
        if size == (1920, 1080) and near(logo, accent) and near(corner, background):
            if near(avatar, accent):
                return False  # that is the login screen already
            machine.screen.picture(picture)
            return True
        time.sleep(0.1)
    return False


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('startup', encrypted=len(sys.argv) > 1)
    checks = Checks()

    print('booting the installed system, watching the screen', flush=True)
    machine = Machine(**machine_settings)
    try:
        menu = wait_for_menu(machine)
        if not checks.report(menu is not None, 'the boot menu shows, with a box around its first entry'):
            machine.screen.picture(f'{pictures}/startup-stuck.png')
            sys.exit(1)
        stop_countdown(machine)
        machine.screen.picture(f'{pictures}/startup-menu.png')
        machine.screen.press('enter')
        # the theme's colours are known once the system is up; until then, the ones the menu showed
        loading = watch_loading(machine, menu[0], menu[1], f'{pictures}/startup-loading.png')
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        background, accent = colours(machine)
        checks.report(near(menu[0], background) and near(menu[1], accent),
                      "the boot menu wears the theme: its background and its accent", f'{menu}, theme {background} {accent}')
        checks.report(loading, 'the loading screen shows the logo over the same picture')

        _, output = machine.run("sed -n -E \"s/^ *(menuentry|submenu) '([^']*)'.*/\\2/p\" /boot/grub/grub.cfg")
        names = output.splitlines()
        wanted = [name for name in ENTRIES if name in names or name not in ('Snapshots', 'Other systems')]
        checks.report(names == wanted, "the menu's entries are the system's, in their order", output)
        # the menu leaves Snapshots out until there is a snapshot to boot
        if machine.run('test -e /boot/grub/grub-btrfs.cfg')[0] != 0:
            names = [name for name in names if name != 'Snapshots']
        _, output = machine.run('plymouth-set-default-theme; lsinitcpio /boot/initramfs-linux-zen.img | grep -c themes/kavat/')
        checks.report(output.split() and output.split()[0] == 'kavat' and output.split()[-1] not in ('', '0'),
                      "the loading screen is the system's, and travels inside the boot image", output)
        _, output = machine.run('grep -c "^greeter-session=kavat-greeter" /etc/lightdm/lightdm.conf')
        checks.report(output == '1', "LightDM shows the system's own login screen", output)

        if checks.report(wait_for_login_screen(machine), 'the login screen shows who logs in'):
            _, (avatar, corner, glow) = machine.screen.pixels(AVATAR, CORNER, GLOW)
            checks.report(near(avatar, accent) and near(corner, background) and not near(glow, corner),
                          'the login screen wears the theme, over the same picture', f'{avatar} {corner} {glow}')
        machine.screen.picture(f'{pictures}/startup-login.png')
        machine.screen.type('not-the-password')
        machine.screen.press('enter')
        time.sleep(4)
        _, output = machine.run(f'pgrep -u {USER} -x Hyprland; loginctl list-sessions --no-legend | grep -c {USER}')
        checks.report(output == '0', 'a wrong password does not log in', output)
        log_in(machine, PASSWORD)
        check_desktop(machine, checks, USER)

        print('giving the boot another look', flush=True)
        _, before = machine.run('stat -c %Y /boot/initramfs-linux-zen.img')
        status, output = machine.run('kavat-theme boot mode light accent teal', 300)
        checks.report(status == 0, 'kavat-theme boot gives the three screens another look', output[-800:])
        light, teal = colours(machine)
        checks.report(sum(light) > 600 and not near(teal, accent), 'the look it made is light, with another accent', f'{light} {teal}')
        _, output = machine.run(f'grep -c \'desktop-color: "#{"%02x%02x%02x" % light}"\' /boot/grub/themes/kavat/theme.txt; '
                                'stat -c %Y /boot/initramfs-linux-zen.img')
        checks.report(output.split()[0] == '1' and output.split()[-1] != before,
                      'the boot menu has it, and the boot image was built again for the loading screen', output)
        status, output = machine.run('kavat-theme boot', 300)
        _, again = machine.run(f'cat {TOKENS}')
        checks.report(status == 0 and colours(machine) == (light, teal),
                      'asked again with no settings, it keeps the look it was given', output[-800:] + again)
        if checks.failed:
            print_journal(machine)
        checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()
    if checks.failed:
        sys.exit(1)

    print('booting again, into the recovery console', flush=True)
    machine = Machine(**machine_settings)
    try:
        menu = wait_for_menu(machine)
        if checks.report(menu is not None and near(menu[0], light) and near(menu[1], teal),
                         'the boot menu wears the new look', f'{menu}, theme {light} {teal}'):
            for _ in range(names.index('Recovery')):
                machine.screen.press('down')
            machine.screen.picture(f'{pictures}/startup-menu-light.png')
            machine.screen.press('enter')
            if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the Recovery entry boots'):
                status, output = machine.wait_until('systemctl is-active kavat-rescue.service', 60)
                _, state = machine.run('systemctl is-active lightdm; cat /proc/cmdline')
                checks.report(status == 0 and state.startswith('inactive') and 'kavat-rescue.target' in state,
                              'it ends in the recovery console, with no desktop', f'{output}\n{state}')
                status, output = machine.wait_until("! grub-editenv /boot/grub/grubenv list | grep -q '^kavat_boot='", 30)
                checks.report(status == 0, 'that does not count as a boot that failed',
                              machine.run('grub-editenv /boot/grub/grubenv list')[1])
                time.sleep(2)
                machine.screen.picture(f'{pictures}/startup-recovery.png')
                # 3 is "Start the system as usual": the login screen has to come, in the new look
                machine.screen.type('3')
                machine.screen.press('enter')
                shown = False
                deadline = time.time() + 90
                while not shown and time.time() < deadline:
                    time.sleep(1)
                    size, (avatar, corner) = machine.screen.pixels(AVATAR, CORNER)
                    shown = size == (1920, 1080) and near(avatar, teal) and near(corner, light)
                checks.report(shown, 'from there the system starts as usual, and the login screen wears the new look')
                machine.screen.picture(f'{pictures}/startup-login-light.png')
        else:
            machine.screen.picture(f'{pictures}/startup-stuck.png')
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/startup-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
