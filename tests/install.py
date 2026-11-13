#!/usr/bin/env python3
"""Install the system from the ISO onto a virtual disk, then boot the disk and log in.

usage: tests/install.py [--secure-boot] [iso]
  --secure-boot   the machine's firmware does Secure Boot and waits for keys (setup mode):
                  the installer must leave it that way and the system must still boot; the
                  disk and the pictures then go in out/test/secure-boot/
  iso             the image to install from; the newest one in out/ when not given

The machine boots from the ISO with an empty 60 GB disk and no network. The installer is
driven as a person would drive it, with the keyboard; then the machine is switched off and
started again from the disk alone, and the test logs in at the login screen. Pictures of
the screen are left in out/test/. The exit status is 0 only when every check passes.

Needs qemu-desktop and edk2-ovmf. It runs as a normal user and takes about ten minutes.
"""
import os
import subprocess
import sys
import time

from machine import BOOT_TIMEOUT, ROOT, Checks, Machine, check_desktop, check_packages, newest_iso, print_journal

USER = 'tester'
PASSWORD = 'kavat-123'
INSTALL_TIMEOUT = 900  # seconds for the installer to do its work
LOG = '/tmp/installer.log'
# what the installed system's root and its other subvolumes are mounted on
SUBVOLUMES = {'/': '/@', '/home': '/@home', '/games': '/@games', '/var/log': '/@log',
              '/var/cache/pacman/pkg': '/@pkg', '/.snapshots': '/@snapshots'}


def install(machine, checks, pictures):
    """Run the installer on the live session. True when it says it finished well."""
    live = 'runuser -u live -- env XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1'
    status, output = machine.wait_until('pgrep -u live -x noctalia', 120)
    if not checks.report(status == 0, 'the live session is up', output):
        return False

    machine.run(f'({live} kavat-installer -d > {LOG} 2>&1 &)')
    status, output = machine.wait_until(f"grep -q 'Requirements model has' {LOG}", 90)
    if not checks.report(status == 0, 'the installer opens', output):
        return False
    time.sleep(3)

    screen = machine.screen
    for _ in ('welcome', 'location', 'keyboard', 'partitions'):  # each one as it comes
        screen.press('alt', 'n')
        time.sleep(3)
    # the users page: name, then login name and computer name already filled in, then the password twice
    screen.type(USER)
    for _ in range(3):
        screen.press('tab')
    screen.type(PASSWORD)
    screen.press('tab')
    screen.type(PASSWORD)
    time.sleep(1)
    screen.press('alt', 'n')
    time.sleep(3)
    screen.picture(f'{pictures}/install-summary.png')
    screen.press('alt', 'i')  # Install, which asks to confirm, with Install now chosen
    for _ in range(5):
        time.sleep(4)
        status, output = machine.run(f"grep -q 'Starting job' {LOG}")
        if status == 0:
            break
        screen.press('enter')
    if not checks.report(status == 0, 'the installation starts'):
        screen.picture(f'{pictures}/install-stuck.png')
        return False

    status, _ = machine.wait_until(f"grep -q -E 'completion: (succeeded|failed)|Installation failed' {LOG}", INSTALL_TIMEOUT)
    done, _ = machine.run(f"grep -q 'completion: succeeded' {LOG}")
    _, errors = machine.run(f"grep -E 'ERROR|Installation failed|Missing variables|Traceback' {LOG} | grep -v -E 'os-prober|smartctl' | tail -n 15")
    screen.picture(f'{pictures}/install-end.png')
    return checks.report(status == 0 and done == 0, 'the installation finishes without errors', errors)


def check_installed(machine, checks, secure_boot):
    """Check the system that booted from the disk, before anybody logs in."""
    status, output = machine.run("sbctl verify 2>&1 | grep -E 'grubx64|BOOTX64|vmlinuz'")
    checks.report(status == 0 and 'not signed' not in output and output.count('is signed') >= 4,
                  "the system's own keys sign GRUB and the kernels", output)
    status, output = machine.run('f=$(ls /sys/firmware/efi/efivars/SecureBoot-* 2>/dev/null); '
                                 'if [ -n "$f" ]; then od -An -t u1 -j4 -N1 "$f" | tr -d " "; else echo 0; fi; '
                                 'find /var/lib/sbctl -name db.key')
    checks.report(output.startswith('0\n') and output.endswith('db.key'), 'Secure Boot stays off, with the keys ready', output)
    for where, subvolume in SUBVOLUMES.items():
        status, output = machine.run(f'findmnt -n -o FSTYPE,OPTIONS --mountpoint {where}')
        ok = status == 0 and output.startswith('btrfs') and f'subvol={subvolume}' in output.split(',')
        checks.report(ok, f'{where} is the Btrfs subvolume {subvolume[1:]}', output)
    status, output = machine.run('findmnt -n -o FSTYPE /efi')
    checks.report(output == 'vfat', 'the EFI partition is mounted on /efi', output)

    status, output = machine.run('swapon --show=NAME --noheadings')
    checks.report(output == '/dev/zram0', 'memory is compressed in memory, with no swap on the disk', output)
    status, output = machine.run('uname -r; ls /boot | grep -c -E "^(vmlinuz|initramfs)-linux-(zen|lts)"')
    checks.report(output.endswith('-zen\n4'), 'it runs the zen kernel and the fallback kernel is in /boot', output)
    status, output = machine.run('grep -c -E "^(menuentry|submenu)" /boot/grub/grub.cfg; grep -c "vmlinuz-linux-lts" /boot/grub/grub.cfg; grep -c "EFI BootNext" /boot/grub/grub.cfg')
    entries = output.split()
    checks.report(len(entries) == 3 and int(entries[1]) > 0 and entries[2] == '0',
                  "GRUB's menu offers the fallback kernel and none of the firmware's entries", output)
    status, output = machine.run('readlink /boot/grub; ls /efi/grub/grub.cfg /efi/grub/grubenv')
    checks.report(status == 0 and output.startswith('/efi/grub\n'),
                  "GRUB's files are on the EFI partition, where GRUB can write", output)
    status, output = machine.run('snapper --no-dbus -c root list && systemctl is-active grub-btrfsd')
    checks.report(status == 0, 'snapper keeps the system snapshots and the boot menu follows them', output)
    status, output = machine.run('grep ^HOOKS /etc/mkinitcpio.conf; lsinitcpio /boot/initramfs-linux-zen.img | grep systemd-volatile-root.service')
    checks.report(status == 0 and '(systemd ' in output, "the boot image is systemd's, and can boot a read-only snapshot", output)
    status, output = machine.run('grep -o splash /proc/cmdline; grep ^HOOKS /etc/mkinitcpio.conf')
    checks.report(output.startswith('splash') and ' plymouth' in output,
                  'the loading screen is in the boot image and on the kernel line', output)
    status, output = machine.run('find /etc -name "*.pacnew"')
    checks.report(not output, 'no settings file waits to be merged', output)

    status, output = machine.run('pacman -Qq kavat-installer kavat-calamares mkinitcpio-archiso 2>/dev/null; '
                                 'id live 2>/dev/null; ls /etc/sudoers.d/live /etc/mkinitcpio.conf.d/archiso.conf 2>/dev/null')
    checks.report(not output, 'nothing of the live medium is left', output)
    status, output = machine.run("passwd -S root | cut -d' ' -f2")
    checks.report(output == 'L', 'nobody can log in as root', output)
    status, output = machine.run(f'id -nG {USER}')
    checks.report(status == 0 and 'wheel' in output.split(), f'{USER} exists and administers the machine', output)
    status, output = machine.wait_until('systemctl is-active lightdm', 60)
    checks.report(status == 0, 'the login screen is up', output)
    status, output = machine.run('systemctl --failed --no-legend')
    checks.report(not output, 'no service failed', output)


def main():
    args = sys.argv[1:]
    secure_boot = '--secure-boot' in args
    args = [arg for arg in args if arg != '--secure-boot']
    if len(args) > 1 or any(arg.startswith('-') for arg in args):
        sys.exit(__doc__)
    iso = args[0] if args else newest_iso()
    pictures = f'{ROOT}/out/test' + ('/secure-boot' if secure_boot else '')
    disk, firmware = f'{pictures}/disk.qcow2', f'{pictures}/disk-firmware.fd'
    os.makedirs(pictures, exist_ok=True)
    for old in (disk, firmware):
        if os.path.exists(old):
            os.remove(old)
    subprocess.run(['qemu-img', 'create', '-q', '-f', 'qcow2', disk, '60G'], check=True)

    checks = Checks()
    print(f'installing from {os.path.relpath(iso)}', flush=True)
    machine = Machine(iso=iso, disk=disk, firmware=firmware, secure_boot=secure_boot)
    try:
        installed = checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the ISO boots') and install(machine, checks, pictures)
        if installed:
            checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()

    if installed:
        print('booting the installed disk', flush=True)
        machine = Machine(disk=disk, firmware=firmware, secure_boot=secure_boot)
        try:
            if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
                check_installed(machine, checks, secure_boot)
                check_packages(machine, checks)
                time.sleep(5)  # the login screen takes a moment to listen
                machine.screen.picture(f'{pictures}/login.png')
                machine.screen.type(PASSWORD)
                machine.screen.press('enter')
                check_desktop(machine, checks, USER)
                # later tests start from this disk, and expect its last boot to have gone well
                status, output = machine.wait_until("! grub-editenv /boot/grub/grubenv list | grep -q '^kavat_boot='", 60)
                checks.report(status == 0, 'the next boot is told this one reached the desktop',
                              machine.run('grub-editenv /boot/grub/grubenv list')[1])
                if checks.failed:
                    print_journal(machine)
                else:
                    time.sleep(3)
            machine.screen.picture(f'{pictures}/installed.png')
        finally:
            machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
