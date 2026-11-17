#!/usr/bin/env python3
"""Calamares step: the answers given to the installer are kept on the new system.

Every page leaves what was answered in Calamares' global storage. This writes it down, in
/etc/kavat/kavat-install.toml on the new system: given back to the installer, that file
fills its pages in with the same answers. The kavat-software step, which runs after this
one, adds the programs to it. No password is written: those are asked for again. The disk
is written as what it is, its model, its serial number and its size, and not as its name,
which changes from one boot to the next: the installer comes up with that disk chosen when
it finds it, and still asks before it touches it. It never stops the installation.
"""
import json
import os
import subprocess

import libcalamares

HEADER = '''# What was answered when the system was installed. Given back to the installer, it fills
# its pages in with the same answers: copy it to the top folder of a USB stick that is
# plugged in when the installer opens, or start it with `kavat-installer --answers <file>`.
# It holds no password: those are asked for again. The disk is here as what it is, not as
# its name: when one disk of the machine is that one, the installer comes up with it chosen,
# and still asks before it touches it.
'''


def pretty_name():
    return 'Keeping the answers of this installation'


def disk_of(partitions):
    """What the disk the system went to is: {'model': …, 'serial': …, 'size': bytes}, without what it has not."""
    root = next((part.get('device') for part in partitions
                 if isinstance(part, dict) and part.get('mountPoint') == '/'), None)
    if not isinstance(root, str) or not root:
        return {}
    try:
        disk = subprocess.run(['lsblk', '-ndo', 'PKNAME', root], capture_output=True, text=True, check=True,
                              timeout=30).stdout.split()[0]
        listed = subprocess.run(['lsblk', '--json', '-b', '-d', '-o', 'MODEL,SERIAL,SIZE', f'/dev/{disk}'],
                                capture_output=True, text=True, check=True, timeout=30)
        found = json.loads(listed.stdout)['blockdevices'][0]
    except (OSError, ValueError, KeyError, IndexError, subprocess.SubprocessError):
        return {}
    what = {name: found[name].strip() for name in ('model', 'serial')
            if isinstance(found.get(name), str) and found[name].strip()}
    if isinstance(found.get('size'), int) and found['size'] > 0:
        what['size'] = found['size']
    return what


def answers(storage):
    """What was answered, as (section, name, value) lines; only what a page did leave."""
    def value(key):
        return storage.value(key) if storage.contains(key) else None

    def text(section, name, key):
        found = value(key)
        return [(section, name, found)] if isinstance(found, str) and found else []

    lines = []
    locale = value('localeConf')
    if isinstance(locale, dict) and isinstance(locale.get('LANG'), str):
        lines.append(('system', 'locale', locale['LANG']))
    region, zone = value('locationRegion'), value('locationZone')
    if isinstance(region, str) and isinstance(zone, str) and region and zone:
        lines.append(('system', 'timezone', f'{region}/{zone}'))
    lines += text('system', 'keyboard', 'keyboardLayout')
    variant = value('keyboardVariant')
    if isinstance(variant, str) and variant:
        lines.append(('system', 'keyboard_variant', variant))
    lines += text('system', 'hostname', 'hostname')
    lines += text('user', 'name', 'fullname') + text('user', 'login', 'username')
    lines.append(('user', 'autologin', bool(value('autoLoginUser'))))
    choices = value('partitionChoices')
    if isinstance(choices, dict) and isinstance(choices.get('install'), str):
        lines.append(('disk', 'method', choices['install']))
    partitions = value('partitions')
    if isinstance(partitions, list):
        # only whether it is encrypted: its password stays in the installer's memory
        lines.append(('disk', 'encrypt', any(isinstance(part, dict) and 'luksMapperName' in part for part in partitions)))
        lines += [('disk', name, value) for name, value in disk_of(partitions).items()]
    lines += text('appearance', 'palette', 'kavatPalette') + text('appearance', 'mode', 'kavatMode')
    lines += text('appearance', 'accent', 'kavatAccent')
    return lines


def run():
    storage = libcalamares.globalstorage
    try:
        written, section = HEADER, None
        for name_of_section, name, value in answers(storage):
            if name_of_section != section:
                section = name_of_section
                written += f'\n[{section}]\n'
            # a string of JSON is a string of TOML too
            written += f'{name} = {json.dumps(value, ensure_ascii=False)}\n'
        folder = os.path.join(storage.value('rootMountPoint'), 'etc/kavat')
        os.makedirs(folder, mode=0o755, exist_ok=True)
        path = os.path.join(folder, 'kavat-install.toml')
        with open(path, 'w', encoding='utf-8') as file:
            file.write(written)
        os.chmod(path, 0o644)
    except Exception as error:  # the system is installed all the same
        libcalamares.utils.warning(f'kavat-answers: the answers were not kept: {error}')
    # "Log in automatically" on the Users page: the login screen is told who, in the new system
    person = storage.value('autoLoginUser') if storage.contains('autoLoginUser') else None
    if isinstance(person, str) and person:
        try:
            libcalamares.utils.target_env_call(['kavat-login-settings', 'set', f'auto={person}'])
        except Exception as error:
            libcalamares.utils.warning(f'kavat-answers: {person} does not log in by itself: {error}')
    return None
