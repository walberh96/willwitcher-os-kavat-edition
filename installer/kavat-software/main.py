#!/usr/bin/env python3
"""Calamares step: the programs checked on the Software page are installed in the new system.

The page leaves the names of the packages in Calamares' global storage. This installs them
with pacman, working on the new system as pacstrap does, from the repositories that system is set to, so they
are the same versions the system was built with. It needs a network. It never stops the
installation: with no network, or with a name pacman does not know, the system is installed
as it comes and what was left out is written down, in /etc/kavat/kavat-install.toml, for
`kav install` later. That file also keeps what was chosen, to repeat it.
"""
import os
import re
import subprocess

import libcalamares

NAME = r'[a-z0-9][a-z0-9@._+-]*'


def pretty_name():
    return 'Adding the software you chose'


def pacman(root, *arguments, timeout=3600):
    """pacman, working on the new system as pacstrap does: its packages, its settings and
    its keys, with the network of the live session. What it printed, and whether it worked."""
    command = ['pacman', '--root', root, '--dbpath', f'{root}/var/lib/pacman', '--config', f'{root}/etc/pacman.conf',
               '--cachedir', f'{root}/var/cache/pacman/pkg', '--gpgdir', f'{root}/etc/pacman.d/gnupg', '--noconfirm', *arguments]
    # no snapshot is taken around this change: the system is not running yet
    environment = dict(os.environ, SNAP_PAC_SKIP='y')
    try:
        done = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False, env=environment)
    except (OSError, subprocess.TimeoutExpired) as error:
        return str(error), False
    return done.stdout + done.stderr, done.returncode == 0


def run():
    storage = libcalamares.globalstorage
    asked = storage.value('kavatSoftware') if storage.contains('kavatSoftware') else ''
    wanted = [name for name in str(asked or '').split() if re.fullmatch(NAME, name)]
    root = storage.value('rootMountPoint')
    installed, left_out, why = [], [], ''
    if wanted:
        libcalamares.job.setprogress(0.05)
        output, reached = pacman(root, '-Sy', timeout=120)
        if not reached:
            left_out, why = wanted, 'the repositories could not be reached: there was no network'
        else:
            # a name pacman does not know would stop the whole lot: they are tried one line at a time
            known = [name for name in wanted if pacman(root, '-Sp', '--', name, timeout=60)[1]]
            left_out = [name for name in wanted if name not in known]
            if left_out:
                why = "pacman does not know them: they may be in the AUR, which comes in through kav's reviewed list"
            if known:
                libcalamares.job.setprogress(0.2)
                output, worked = pacman(root, '-S', '--needed', '--', *known)
                if worked:
                    installed = known
                else:
                    left_out, why = wanted, 'pacman could not install them: ' + output.strip().splitlines()[-1][:200] if output.strip() else 'pacman failed'
                    libcalamares.utils.warning('kavat-software: ' + output[-2000:])
    # what was asked for, what came in and what did not, to repeat it and to finish it
    folder = os.path.join(root, 'etc/kavat')
    os.makedirs(folder, mode=0o755, exist_ok=True)
    quoted = lambda names: '[' + ', '.join(f'"{name}"' for name in names) + ']'
    with open(os.path.join(folder, 'kavat-install.toml'), 'w', encoding='utf-8') as file:
        file.write('# What was chosen when the system was installed.\n\n[software]\n'
                   f'asked     = {quoted(wanted)}\ninstalled = {quoted(installed)}\n'
                   f'# `kav install` followed by these names brings them in\nleft_out  = {quoted(left_out)}\n'
                   + (f'why       = "{why}"\n' if left_out else ''))
    libcalamares.job.setprogress(1.0)
    return None
