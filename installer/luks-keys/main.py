#!/usr/bin/env python3
"""Calamares step: the keys that let an encrypted system boot without asking for a password.

It only does something when the disk is to be encrypted. The passphrase typed in the
installer stays, as the recovery password. Next to it goes a random key, kept in two places:

- /crypto_keyfile.bin, which goes inside the boot image, so that once GRUB has opened the
  disk the system opens it again on its own. The boot image lives on the encrypted disk.
- Sealed in the TPM2, if the machine has one, in /efi/grub/kavat-root.tpm: GRUB unseals it
  and opens the disk without asking. It is sealed to PCR 7, the firmware's Secure Boot
  state, as systemd does by default.

GRUB tries the disk's keys in the order they are written, and the passphrase's key is slow to
try on purpose, so the random key has to come first: the passphrase is taken out and put back
after it. The random key is long and random, so it needs no slow rounds of its own.
"""
import os
import secrets
import subprocess
import tempfile

import libcalamares

KEYFILE = '/crypto_keyfile.bin'
SEALED = '/efi/grub/kavat-root.tpm'
TPM = '/dev/tpmrm0'


def pretty_name():
    return 'Setting up the keys of the encrypted disk'


def run_command(*command, stdin=None):
    subprocess.run(command, input=stdin, check=True, capture_output=True)


def run():
    partitions = libcalamares.globalstorage.value('partitions') or []
    root = next((p for p in partitions if p.get('mountPoint') == '/' and p.get('luksMapperName')), None)
    if root is None:
        return None
    target = libcalamares.globalstorage.value('rootMountPoint')
    device, passphrase = root['device'], root['luksPassphrase'].encode()
    keyfile = target + KEYFILE
    os.umask(0o077)
    try:
        with open(keyfile, 'wb') as f:
            f.write(secrets.token_bytes(64))
        run_command('cryptsetup', 'luksAddKey', '--batch-mode', '--key-file', '-',
                    '--pbkdf', 'pbkdf2', '--pbkdf-force-iterations', '1000', device, keyfile, stdin=passphrase)
        run_command('cryptsetup', 'luksRemoveKey', '--batch-mode', '--key-file', '-', device, stdin=passphrase)
        with tempfile.NamedTemporaryFile(dir='/run') as typed:
            typed.write(passphrase)
            typed.flush()
            run_command('cryptsetup', 'luksAddKey', '--batch-mode', '--key-file', keyfile, device, typed.name)
        if os.path.exists(TPM):
            os.makedirs(os.path.dirname(target + SEALED), exist_ok=True)
            run_command('grub-protect', '--action=add', '--protector=tpm2', '--tpm2key', f'--tpm2-device={TPM}',
                        f'--tpm2-keyfile={keyfile}', f'--tpm2-outfile={target + SEALED}', '--tpm2-pcrs=7')
    except subprocess.CalledProcessError as error:
        libcalamares.utils.warning(f'{error.cmd}: {error.stderr.decode(errors="replace")}')
        return ('Could not set up the keys of the encrypted disk',
                f'{error.cmd[0]} failed: {error.stderr.decode(errors="replace")}')
    return None
