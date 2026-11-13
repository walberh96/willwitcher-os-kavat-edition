#!/usr/bin/env python3
"""Boot the ISO in a virtual machine and check that it reaches the desktop.

usage: tests/boot.py [iso]
  iso   the image to boot; the newest one in out/ when not given

This is the gate of phase 1. The machine boots from the ISO with UEFI and no network. The
checks run inside it, through QEMU's guest agent, and a picture of the screen is left in
out/test/desktop.png. The exit status is 0 only when every check passes.

Needs qemu-desktop and edk2-ovmf. It runs as a normal user.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, ROOT, Checks, Machine, check_desktop, check_packages, newest_iso, print_journal


def main():
    if len(sys.argv) > 2 or (len(sys.argv) == 2 and sys.argv[1].startswith('-')):
        sys.exit(__doc__)
    iso = sys.argv[1] if len(sys.argv) == 2 else newest_iso()
    print(f'booting {os.path.relpath(iso)}', flush=True)

    picture = f'{ROOT}/out/test/desktop.png'
    checks = Checks()
    machine = Machine(iso=iso)
    try:
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the system boots'):
            check_desktop(machine, checks, 'live')
            check_packages(machine, checks)
            if checks.failed:
                print_journal(machine)
            else:
                time.sleep(3)  # let the shell finish drawing before the picture
        machine.screen.picture(picture)
        print(f'picture of the screen: {os.path.relpath(picture)}')
    finally:
        machine.stop()
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
