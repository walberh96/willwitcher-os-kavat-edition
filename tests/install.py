#!/usr/bin/env python3
"""Install the system from the ISO onto a virtual disk, then boot the disk and log in.

usage: tests/install.py [--encrypt] [iso]
  --encrypt   the machine has a TPM2, and the installer is told to encrypt the disk: the
              system must boot without asking for a password, and, once the TPM2 is gone,
              open with the recovery password typed at GRUB; the disk, the TPM2's state and
              the pictures then go in out/test/encrypted/
  iso         the image to install from; the newest one in out/ when not given

The machine boots from the ISO with an empty 60 GB disk and no network. The installer is
driven as a person would drive it, with the keyboard; then the machine is switched off and
started again from the disk alone, and the test logs in at the login screen. Pictures of
the screen are left in out/test/. The exit status is 0 only when every check passes.

Needs qemu-desktop and edk2-ovmf. It runs as a normal user and takes about ten minutes.
"""
import os
import shutil
import subprocess
import sys
import time

from machine import (BOOT_TIMEOUT, ROOT, Checks, Machine, check_desktop, check_packages, newest_iso, print_journal,
                     wait_for_login_screen)

USER = 'tester'
PASSWORD = 'kavat-123'
DISK_PASSWORD = 'kavat-disk-456'  # the recovery password of the encrypted disk
ENCRYPT_BOX = (282, 854)  # where the disk page shows Encrypt system, on a 1920x1080 screen
INSTALL_TIMEOUT = 900  # seconds for the installer to do its work
LOG = '/tmp/installer.log'
# what the installed system's root and its other subvolumes are mounted on
SUBVOLUMES = {'/': '/@', '/home': '/@home', '/games': '/@games', '/var/log': '/@log',
              '/var/cache/pacman/pkg': '/@pkg', '/.snapshots': '/@snapshots'}


# where the installer's side bar names each step, on a 1920x1080 screen; the one it is on
# is painted in the accent colour
STEPS = {'location': 195, 'keyboard': 234, 'partitions': 273, 'users': 312, 'summary': 351}


def next_page(screen, step, timeout=60):
    """Press Next and wait until the installer shows that step. True when it does.

    Some pages take a while to come up, and keys typed before they do are lost. So is, now
    and then, the press of Next itself: it is pressed again when nothing has moved for a while.
    """
    deadline = time.time() + timeout
    while time.time() < deadline:
        screen.press('alt', 'n')
        again = time.time() + 12
        while time.time() < min(again, deadline):
            time.sleep(1)
            red, green, blue = screen.pixel(30, STEPS[step])
            if red > 200 and green > 130 and blue < 110:
                time.sleep(1)
                return True
    return False


def install(machine, checks, pictures, encrypt):
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
    shown = all(next_page(screen, step) for step in ('location', 'keyboard', 'partitions'))
    # the disk page, where erasing the disk is already chosen
    if encrypt:
        # Encrypt system, then the recovery password twice. The box is clicked: its letter, C,
        # is also Cancel's, so Alt+C sometimes lands on Cancel.
        screen.click(*ENCRYPT_BOX)
        time.sleep(1)
        screen.press('tab')
        screen.type(DISK_PASSWORD)
        screen.press('tab')
        screen.type(DISK_PASSWORD)
        time.sleep(1)
    screen.picture(f'{pictures}/install-disk.png')
    shown = shown and next_page(screen, 'users')
    # the users page: name, then login name and computer name already filled in, then the password twice
    screen.type(USER)
    for _ in range(3):
        screen.press('tab')
    screen.type(PASSWORD)
    screen.press('tab')
    screen.type(PASSWORD)
    time.sleep(1)
    shown = shown and next_page(screen, 'summary')
    screen.picture(f'{pictures}/install-summary.png')
    if not checks.report(shown, 'the installer goes through its pages'):
        return False
    # Install, which asks to confirm, with Install now chosen. A busy machine can lose either
    # key, so both are pressed again until the work starts.
    for _ in range(5):
        for keys in (('alt', 'i'), ('enter',)):
            screen.press(*keys)
            time.sleep(4)
            status, output = machine.run(f"grep -q 'Starting job' {LOG}")
            if status == 0:
                break
        if status == 0:
            break
    if not checks.report(status == 0, 'the installation starts'):
        screen.picture(f'{pictures}/install-stuck.png')
        return False

    status, _ = machine.wait_until(f"grep -q -E 'completion: (succeeded|failed)|Installation failed' {LOG}", INSTALL_TIMEOUT)
    done, _ = machine.run(f"grep -q 'completion: succeeded' {LOG}")
    _, errors = machine.run(f"grep -E 'ERROR|Installation failed|Missing variables|Traceback' {LOG} | grep -v -E 'os-prober|smartctl' | tail -n 15")
    screen.picture(f'{pictures}/install-end.png')
    return checks.report(status == 0 and done == 0, 'the installation finishes without errors', errors)


def check_encrypted(machine, checks):
    """Check that the system is on an encrypted disk, and how it opens."""
    root = 'r=$(findmnt -n -o SOURCE / | sed "s/\\[.*//"); '
    status, output = machine.run(root + 'echo "$r"; lsblk -n -s -o TYPE "$r"')
    checks.report(status == 0 and output.startswith('/dev/mapper/luks-') and 'crypt' in output.split(),
                  'the system is on an encrypted disk', output)
    # the order the keys are written in, which is the order GRUB tries them in
    status, output = machine.run(root + 'd=$(cryptsetup status "$r" | awk \'$1 == "device:" {print $2}\'); '
                                 'cryptsetup luksDump --dump-json-metadata "$d" | grep -A1 \'"kdf":{\' | grep -o \'"type":"[a-z0-9]*"\' | cut -d\'"\' -f4')
    checks.report(output.split() == ['pbkdf2', 'argon2id'], 'two keys open it, and GRUB tries the quick one first', output)
    status, output = machine.run('stat -c %a /crypto_keyfile.bin; lsinitcpio /boot/initramfs-linux-zen.img | grep -x crypto_keyfile.bin; '
                                 'grep -o "rd.luks.key=[^ ]*" /proc/cmdline')
    checks.report(output == '600\ncrypto_keyfile.bin\nrd.luks.key=/crypto_keyfile.bin',
                  'the boot image opens the disk again with its own key, so nobody is asked twice', output)
    status, output = machine.run('test -s /efi/grub/kavat-root.tpm && grep -c tpm2_key_protector_init /boot/grub/grub.cfg')
    checks.report(status == 0, 'GRUB opens the disk with the key sealed in the TPM2', output)


def check_installed(machine, checks):
    """Check the system that booted from the disk, before anybody logs in."""
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
    as_user = f'runuser -u {USER} -- env HOME=/home/{USER} TERM=xterm-kitty'
    status, output = machine.run(f'getent passwd {USER} | cut -d: -f7; {as_user} zsh -i -c "whence -w z ll; echo \\$STARSHIP_CONFIG"; '
                                 f'cd /home/{USER} && {as_user} STARSHIP_CONFIG=/usr/share/kavat/starship.toml starship prompt --status 1')
    checks.report(output.startswith('/usr/bin/zsh\nz: function\nll: alias\n/usr/share/kavat/starship.toml') and USER in output
                  and '❯' in output, f"{USER}'s shell is zsh, with the system's prompt and tools", output)
    # the fastest of five: the first one also builds what the later ones reuse
    status, output = machine.run(f'for n in 1 2 3 4 5; do a=$(date +%s%N); {as_user} zsh -i -c exit 2> /dev/null; '
                                 'echo $(( ($(date +%s%N) - a) / 1000000 )); done | sort -n | head -n 1', 60)
    checks.report(output.isdigit() and int(output) < 100, 'a terminal opens its shell in under 100 ms', f'{output} ms')
    status, output = machine.run("stat -c '%a' /etc/fstab /etc/locale.conf /etc/default/grub /var/lib/kavat/theme | sort -u | tr '\\n' ' '")
    checks.report(output == '644 755', 'what the installer wrote can be read by everyone', output)


def boot_with_password(disk, firmware, checks, pictures):
    """Boot the encrypted disk with no TPM2: GRUB has to ask for the recovery password."""
    print('booting the encrypted disk without its TPM2', flush=True)
    machine = Machine(disk=disk, firmware=firmware)
    try:
        time.sleep(25)  # firmware, then GRUB, which tries the TPM2 and then asks
        machine.screen.picture(f'{pictures}/password.png')
        machine.screen.type(DISK_PASSWORD)
        machine.screen.press('enter')
        if checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'without the TPM2, the recovery password typed at GRUB opens the disk'):
            status, output = machine.run('cat /proc/cmdline')
            checks.report('kavat.safe' not in output and '@snapshots' not in output, 'it is a normal boot', output)
            status, output = machine.wait_until('systemctl is-active lightdm', 60)
            checks.report(status == 0, 'the login screen is up', output)
        else:
            machine.screen.picture(f'{pictures}/password-stuck.png')
    finally:
        machine.stop()


def main():
    args = sys.argv[1:]
    encrypt = '--encrypt' in args
    args = [arg for arg in args if arg != '--encrypt']
    if len(args) > 1 or any(arg.startswith('-') for arg in args):
        sys.exit(__doc__)
    iso = args[0] if args else newest_iso()
    pictures = f'{ROOT}/out/test' + ('/encrypted' if encrypt else '')
    disk, firmware = f'{pictures}/disk.qcow2', f'{pictures}/disk-firmware.fd'
    tpm = f'{pictures}/tpm' if encrypt else None
    os.makedirs(pictures, exist_ok=True)
    for old in (disk, firmware):
        if os.path.exists(old):
            os.remove(old)
    if tpm:
        shutil.rmtree(tpm, ignore_errors=True)
    subprocess.run(['qemu-img', 'create', '-q', '-f', 'qcow2', disk, '60G'], check=True)

    checks = Checks()
    print(f'installing from {os.path.relpath(iso)}', flush=True)
    machine = Machine(iso=iso, disk=disk, firmware=firmware, tpm=tpm)
    try:
        installed = (checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the ISO boots')
                     and install(machine, checks, pictures, encrypt))
        if installed:
            checks.report(machine.power_off(), 'the machine switches off')
    finally:
        machine.stop()

    if installed:
        print('booting the installed disk', flush=True)
        machine = Machine(disk=disk, firmware=firmware, tpm=tpm)
        try:
            if checks.report(machine.wait_for_agent(BOOT_TIMEOUT),
                             'the installed system boots' + (', asking for no password' if encrypt else '')):
                if encrypt:
                    check_encrypted(machine, checks)
                check_installed(machine, checks)
                check_packages(machine, checks)
                checks.report(wait_for_login_screen(machine), 'the login screen shows who logs in')
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
        if encrypt and not checks.failed:
            boot_with_password(disk, firmware, checks, pictures)
    print(f'pictures of the screen: {os.path.relpath(pictures)}/')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
