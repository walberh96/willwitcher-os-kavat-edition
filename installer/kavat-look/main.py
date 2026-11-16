#!/usr/bin/env python3
"""Calamares step: the look chosen on the Appearance page becomes the new system's.

The page leaves the palette, the mode and the accent in Calamares' global storage. This
writes them as the settings of the machine, /etc/kavat/theme.toml on the new system: they
are what the boot menu, the loading screen and the login wear, which the boot loader's step
makes right after this one, and what every user starts from on the desktop. A user's own
~/.config/kavat/theme.toml goes on top of them.

Two choices of the page are not written as they are. The mode "auto" is the mode by the
hour: auto.follow is turned on, which the session reads when it starts, and the screens
before the desktop, which do not change by themselves, wear the side this hour asks for.
The palette "wallpaper" is the colours of the background, and there is none until somebody
logs in: the machine keeps the system's own palette, and the wish is left in the settings
of the background every new user starts with, where `kavat-wallpaper restore` finds it at
the first session and makes the palette.
"""
import datetime
import os
import re
import zoneinfo

import libcalamares

# what each setting may be; the page offers nothing else, but what is written is checked
ALLOWED = {'kavatPalette': r'[a-z][a-z0-9-]*', 'kavatMode': r'dark|light|auto', 'kavatAccent': r'[a-z][a-z0-9-]*'}
NAMES = {'kavatPalette': 'palette', 'kavatMode': 'mode', 'kavatAccent': 'accent'}
# the hours the theme's own settings give the day, auto.light_from and auto.dark_from
LIGHT_FROM, DARK_FROM = 7, 19


def side_of_now(storage):
    """Light or dark, by the hour it is in the time zone chosen for the new system."""
    now = datetime.datetime.now().astimezone()
    try:
        now = now.astimezone(zoneinfo.ZoneInfo(f"{storage.value('locationRegion')}/{storage.value('locationZone')}"))
    except Exception:  # no zone was chosen, or it is not one: the live session's clock
        pass
    return 'light' if LIGHT_FROM <= now.hour < DARK_FROM else 'dark'


def pretty_name():
    return 'Giving the system the look you chose'


def run():
    storage = libcalamares.globalstorage
    chosen = {}
    for key, name in NAMES.items():
        value = storage.value(key) if storage.contains(key) else None
        if isinstance(value, str) and re.fullmatch(ALLOWED[key], value):
            chosen[name] = value
    if not chosen:
        return None  # the page was never shown: the system keeps its own look
    root = storage.value('rootMountPoint')
    auto = chosen.get('mode') == 'auto'
    if auto:
        chosen['mode'] = side_of_now(storage)
    if chosen.get('palette') == 'wallpaper':
        # the palette and its accents are made from the background, at the first session
        del chosen['palette']
        chosen.pop('accent', None)
        folder = os.path.join(root, 'etc/skel/.config/kavat')
        os.makedirs(folder, mode=0o755, exist_ok=True)
        with open(os.path.join(folder, 'wallpaper.json'), 'w', encoding='utf-8') as file:
            file.write('{\n  "colours": "background"\n}\n')
    lines = [f'{name} = "{value}"' for name, value in chosen.items()]
    if auto:
        lines += ['', '[auto]', 'follow = true']
    folder = os.path.join(root, 'etc/kavat')
    os.makedirs(folder, mode=0o755, exist_ok=True)
    path = os.path.join(folder, 'theme.toml')
    with open(path, 'w', encoding='utf-8') as file:
        file.write('# The look chosen when the system was installed. It is what the boot menu, the loading\n'
                   '# screen and the login wear, and what each user starts from.\n' + '\n'.join(lines) + '\n')
    os.chmod(path, 0o644)
    return None
