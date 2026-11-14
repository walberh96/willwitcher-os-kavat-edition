#!/usr/bin/env python3
"""Break the user's desktop on purpose, and check that the next boot opens in safe mode.

usage: tests/safemode.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. It gives the user a Hyprland configuration that makes
Hyprland quit as it starts, and logs in: the desktop never comes up. At the next boot GRUB has to
show its menu with Safe mode chosen and boot it on its own; in safe mode the desktop has to
come up, with the user's broken file ignored but still there. The boot after that is a
normal one again.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about four minutes.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
BROKEN = f'/home/{USER}/.config/hypr/hyprland.lua'
STATE = "grub-editenv /boot/grub/grubenv list | sed -n 's/^kavat_boot=//p'"




def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('safemode', encrypted=len(sys.argv) > 1)
    checks = Checks()

    print('breaking the desktop', flush=True)
    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            status, output = machine.run('cat /proc/cmdline')
            checks.report('kavat.safe' not in output, 'it is a normal boot', output)
            status, output = machine.wait_until(f'[ "$({STATE})" = greeter ]', 60)
            checks.report(status == 0, 'the boot is recorded as having reached the login screen', machine.run(STATE)[1])
            machine.run(f"install -d -o {USER} -g {USER} $(dirname {BROKEN}) && "
                        f"echo 'os.exit(1)' > {BROKEN} && "
                        f"chown {USER}: {BROKEN}")
            log_in(machine, PASSWORD)
            status, output = machine.wait_until(f'[ "$({STATE})" = session ]', 60)
            checks.report(status == 0, 'the login is recorded', machine.run(STATE)[1])
            time.sleep(30)
            status, output = machine.run(STATE)
            checks.report(output == 'session', 'the desktop does not come up, and that is recorded', output)
            if checks.failed:
                print_journal(machine)
            checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()
    if checks.failed:
        sys.exit(1)

    print('booting after the failure', flush=True)
    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the system boots on its own'):
            status, output = machine.run('cat /proc/cmdline')
            checks.report('kavat.safe=1' in output, 'GRUB chose safe mode', output)
            log_in(machine, PASSWORD)
            check_desktop(machine, checks, USER)
            status, output = machine.run(f'test -s {BROKEN}')
            checks.report(status == 0, "the user's broken file was ignored, not deleted", output)
            status, output = machine.wait_until(f'[ -z "$({STATE})" ]', 60)
            checks.report(status == 0, 'the desktop is recorded as up', machine.run(STATE)[1])
            if checks.failed:
                print_journal(machine)
            else:
                time.sleep(3)
            machine.screen.picture(f'{pictures}/safe-mode.png')
            checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()

    print('booting again', flush=True)
    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the system boots'):
            status, output = machine.run('cat /proc/cmdline')
            checks.report('kavat.safe' not in output, 'it is a normal boot again', output)
    finally:
        machine.stop()
    print(f'picture of the screen: {os.path.relpath(pictures)}/safe-mode.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
