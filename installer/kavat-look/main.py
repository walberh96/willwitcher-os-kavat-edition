#!/usr/bin/env python3
"""Calamares step: the look chosen on the Appearance page becomes the new system's.

The page leaves the palette, the mode and the accent in Calamares' global storage. This
writes them as the settings of the machine, /etc/kavat/theme.toml on the new system: they
are what the boot menu, the loading screen and the login wear, which the boot loader's step
makes right after this one, and what every user starts from on the desktop. A user's own
~/.config/kavat/theme.toml goes on top of them.
"""
import os
import re

import libcalamares

# what each setting may be; the page offers nothing else, but what is written is checked
ALLOWED = {'kavatPalette': r'[a-z][a-z0-9-]*', 'kavatMode': r'dark|light', 'kavatAccent': r'[a-z][a-z0-9-]*'}
NAMES = {'kavatPalette': 'palette', 'kavatMode': 'mode', 'kavatAccent': 'accent'}


def pretty_name():
    return 'Giving the system the look you chose'


def run():
    storage = libcalamares.globalstorage
    lines = []
    for key, name in NAMES.items():
        value = storage.value(key) if storage.contains(key) else None
        if isinstance(value, str) and re.fullmatch(ALLOWED[key], value):
            lines.append(f'{name} = "{value}"')
    if not lines:
        return None  # the page was never shown: the system keeps its own look
    folder = os.path.join(storage.value('rootMountPoint'), 'etc/kavat')
    os.makedirs(folder, mode=0o755, exist_ok=True)
    path = os.path.join(folder, 'theme.toml')
    with open(path, 'w', encoding='utf-8') as file:
        file.write('# The look chosen when the system was installed. It is what the boot menu, the loading\n'
                   '# screen and the login wear, and what each user starts from.\n' + '\n'.join(lines) + '\n')
    os.chmod(path, 0o644)
    return None
