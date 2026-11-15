#!/usr/bin/env python3
"""Give the shell its own name: rename.py <source folder>

The shell started as Noctalia and is now the system's own program, Kavat Shell. This
changes the name wherever a person or another program meets it: the command, the folders
of its settings, its cache and its files, its names on D-Bus, its window names and every
text it shows. It runs on Noctalia's source before it is built.

What keeps the old name, on purpose: the code's own identifiers, which nobody sees; the
`noctalia.` functions that plugins call, so that a plugin written for Noctalia still runs;
the glyphs named after it in the icon font; the plugins published by Noctalia's authors,
which are called noctalia/<name>; and the addresses of Noctalia's own sites.
"""
import os
import re
import sys

NAME, COMMAND = 'Kavat Shell', 'kavat-shell'


def renamed(text):
    """A piece of text, with the shell's name changed where it is the shell's name."""
    if text == 'noctalia-dev':
        return 'kavat'  # who makes it, as it tells the programs that ask
    if 'For more information and documentation, visit:' in text:
        return text.replace('For more information and documentation, visit:\\n  https://noctalia.dev',
                            'The shell of Willwitcher-OS Kavat Edition.')
    if 'noctalia.dev' in text or 'noctalia-dev' in text:
        return text  # an address of Noctalia's
    text = text.replace('dev.noctalia.Noctalia', 'dev.kavat.Shell')
    text = re.sub(r'\b(dev|org)\.noctalia\.', r'\1.kavat.shell.', text)
    text = re.sub(r'/(dev|org)/noctalia/', r'/\1/kavat/shell/', text)
    text = text.replace('_noctalia_', '_kavat_shell_')
    text = text.replace('Noctalia', NAME)
    # not the functions plugins call, the glyphs, or a plugin's id such as noctalia/notes
    return re.sub(r'noctalia(?!\.(?!svg|toml|log|fish)[A-Za-z]|-glyph|/[a-z_]+\b(?![./]))', COMMAND, text)


def code(text):
    """C++ source with the name changed inside its strings only: the code stays as it is."""
    out, at, end = [], 0, len(text)
    while at < end:
        if text.startswith('//', at):
            stop = text.find('\n', at)
            stop = end if stop < 0 else stop
            out.append(text[at:stop])
            at = stop
        elif text.startswith('/*', at):
            stop = text.find('*/', at)
            stop = end if stop < 0 else stop + 2
            out.append(text[at:stop])
            at = stop
        elif text[at] == 'R' and text.startswith('"', at + 1) and (at == 0 or not (text[at - 1].isalnum() or text[at - 1] == '_')):
            open_at = text.index('(', at)
            close = ')' + text[at + 2:open_at] + '"'
            stop = text.index(close, open_at)
            out.append(text[at:open_at + 1] + renamed(text[open_at + 1:stop]) + close)
            at = stop + len(close)
        elif text[at] == '"':
            stop = at + 1
            while text[stop] != '"':
                stop += 2 if text[stop] == '\\' else 1
            inside = text[at + 1:stop]
            # the table of functions plugins call, and the glyph of that name, keep theirs
            keeps = inside == 'noctalia' and re.search(r'(luaL_register\(L,|glyph\s*=)\s*$', text[max(0, at - 40):at])
            out.append('"' + (inside if keeps else renamed(inside)) + '"')
            at = stop + 1
        elif text[at] == "'":
            stop = at + 1
            while stop < end and text[stop] != "'" and text[stop] != '\n':
                stop += 2 if text[stop] == '\\' else 1
            out.append(text[at:stop + 1])
            at = stop + 1
        else:
            out.append(text[at])
            at += 1
    return ''.join(out)


def main():
    root = sys.argv[1]
    for folder, _, files in os.walk(f'{root}/src'):
        for file in files:
            if file.endswith(('.cpp', '.h')):
                path = f'{folder}/{file}'
                text = open(path, encoding='utf-8').read()
                # an #include names a file of the code, not the shell
                lines = [line if line.lstrip().startswith('#include') else None for line in text.split('\n')]
                changed = code(text).split('\n')
                changed = [kept if kept is not None else line for kept, line in zip(lines, changed)]
                open(path, 'w', encoding='utf-8').write('\n'.join(changed))

    # what comes with it: texts, templates and the files named after it
    for folder, _, files in os.walk(f'{root}/assets'):
        for file in files:
            path = f'{folder}/{file}'
            if file.endswith(('.json', '.toml', '.sh', '.css', '.conf', '.lua', '.desktop')) and 'fonts' not in folder:
                text = open(path, encoding='utf-8').read()
                open(path, 'w', encoding='utf-8').write(renamed_lines(text))
            fresh = renamed(file)
            if fresh != file:
                os.rename(path, f'{folder}/{fresh}')

    # the build: the command, where its files go and the files named after it
    path = f'{root}/meson.build'
    build = open(path, encoding='utf-8').read()
    for old, new in [("executable('noctalia',", f"executable('{COMMAND}',"),
                     ("get_option('datadir') / 'noctalia'", f"get_option('datadir') / '{COMMAND}'"),
                     ('dev.noctalia.Noctalia', 'dev.kavat.Shell'),
                     ("'assets/noctalia.svg'", f"'assets/{COMMAND}.svg'")]:
        if old not in build:
            sys.exit(f'meson.build no longer has {old}')
        build = build.replace(old, new)
    open(path, 'w', encoding='utf-8').write(build)


def renamed_lines(text):
    """A text file, line by line, so that an address on one line does not spare the rest."""
    return '\n'.join(renamed(line) for line in text.split('\n'))


if __name__ == '__main__':
    main()
