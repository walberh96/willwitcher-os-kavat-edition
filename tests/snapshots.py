#!/usr/bin/env python3
"""Change the installed system, boot the snapshot from before the change, and go back to it.

usage: tests/snapshots.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. On the installed system it removes a package, and snap-pac
takes a snapshot before and after. Then it asks, with kavat-boot-once, to boot the snapshot
from before once, and checks that the package is there and that the snapshot booted with a layer in
memory over it. From that snapshot it runs kavat-rollback, boots normally, and checks that
the system is now that snapshot: the package is back.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about three minutes.
"""
import os
import re
import sys

from machine import BOOT_TIMEOUT, Checks, Machine, over_installed, print_journal

PACKAGE = 'os-prober'  # removed and brought back; nothing else needs it


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('snapshots', encrypted=len(sys.argv) > 1)

    checks = Checks()
    print('changing the installed system', flush=True)
    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            status, output = machine.run(f'pacman -Rns --noconfirm {PACKAGE}', 120)
            checks.report(status == 0, f'pacman removes {PACKAGE}', output)
            status, output = machine.run('snapper --no-dbus -c root --csvout --no-headers list --columns number,type,description')
            pre = [line.split(',')[0] for line in output.splitlines() if ',pre,' in line and PACKAGE in line]
            if checks.report(len(pre) == 1, 'snap-pac took a snapshot before the change', output):
                number = pre[0]
                status, output = machine.wait_until(f'grep -q "@snapshots/{number}/snapshot" /boot/grub/grub-btrfs.cfg', 60)
                checks.report(status == 0, 'the boot menu lists it', output)
                status, output = machine.run(f'kavat-boot-once {number}')
                checks.report(status == 0, 'the next boot is asked to start from it', output)
            if checks.failed:
                print_journal(machine)
            checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()
    if checks.failed:
        sys.exit(1)

    print(f'booting snapshot {number}', flush=True)
    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the snapshot boots'):
            status, output = machine.run('cat /proc/cmdline')
            checks.report(f'@snapshots/{number}/snapshot' in output and 'systemd.volatile=overlay' in output,
                          'it is the snapshot, with a layer in memory over it', output)
            status, output = machine.run('findmnt -n -o FSTYPE /')
            checks.report(output == 'overlay', 'the root takes writes in memory', output)
            status, output = machine.run(f'pacman -Q {PACKAGE}')
            checks.report(status == 0, f'{PACKAGE} is there, as before the change', output)
            status, output = machine.wait_until('systemctl is-active lightdm', 60)
            checks.report(status == 0, 'the login screen is up', output)
            status, output = machine.run('systemctl --failed --no-legend')
            checks.report(not output, 'no service failed', output)
            status, output = machine.run(f'kavat-rollback {number}', 60)
            checks.report(status == 0, f'kavat-rollback makes snapshot {number} the system', output)
            if checks.failed:
                print_journal(machine)
            machine.screen.picture(f'{pictures}/snapshot.png')
            checks.report(machine.power_off(), 'the machine switches off')
        else:
            machine.screen.picture(f'{pictures}/snapshot.png')
    finally:
        machine.stop()
    if checks.failed:
        print(f'picture of the screen: {os.path.relpath(pictures)}/snapshot.png')
        sys.exit(1)

    print('booting the system after going back', flush=True)
    machine = Machine(**machine_settings)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the system boots'):
            _, cmdline = machine.run('cat /proc/cmdline')
            _, options = machine.run('findmnt -n -o OPTIONS /')
            checks.report('@snapshots' not in cmdline and 'subvol=/@' in options.split(','),
                          'it boots the normal entry, from @', f'{cmdline}\n{options}')
            status, output = machine.run('grub-editenv /boot/grub/grubenv list')
            checks.report(status == 0 and not re.search(r'^kavat_once_snapshot=.', output, re.M),
                          'GRUB booted the snapshot only once', output)
            status, output = machine.run(f'pacman -Q {PACKAGE}')
            checks.report(status == 0, f'{PACKAGE} is back', output)
            status, output = machine.run("btrfs subvolume list / | grep -c ' path @.old-'")
            checks.report(output == '1', 'the system as it was is kept aside', output)
            status, output = machine.wait_until('systemctl is-active lightdm', 60)
            checks.report(status == 0, 'the login screen is up', output)
            if checks.failed:
                print_journal(machine)
            machine.screen.picture(f'{pictures}/after-rollback.png')
    finally:
        machine.stop()
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
