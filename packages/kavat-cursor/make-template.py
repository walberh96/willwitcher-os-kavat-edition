#!/usr/bin/env python3
"""Make kavat-theme's template for the cursor out of Bibata: make-template.py <Bibata> <out>

<Bibata> is the source of the Bibata cursors and <out> the template's folder. Bibata draws
each cursor once, in three stand-in colours, and says where its tip is; this writes those
drawings with the theme's colours in their place, laid out as hyprcursor wants a cursor
theme before it is packed: a folder for each cursor, with its drawings and a meta.hl.
"""
import glob
import os
import sys
import tomllib

# Bibata's stand-in colours, and what takes their place: the body follows the mode, light on
# a dark theme and dark on a light one; its edge is the accent; the turning part of the
# waiting cursors is the accent on the body
COLOURS = {'#00FF00': '{{ text }}', '#0000FF': '{{ accent }}', '#FF0000': '{{ accent }}'}
SIDE = 256  # the side of Bibata's drawings, which the tips are measured on

MANIFEST = '''# Made by kavat-theme from the theme's settings; what is changed here is lost.
name = Kavat
description = The cursor of Willwitcher-OS Kavat Edition, drawn after Bibata Modern
version = 0.1
cursors_directory = hyprcursors
'''
TEMPLATE = '''# The cursor: its body follows the mode and its edge is the accent. This file is made by
# kavat-cursor from Bibata, one entry for each drawing.
#
# hyprcursor wants a theme packed, so kavat-cursor-apply packs these and tells Hyprland.
reload = [["kavat-cursor-apply"]]

[[file]]
from = "Kavat/manifest.hl"
'''


def main():
    bibata, out = sys.argv[1:]
    cursors = tomllib.load(open(f'{bibata}/configs/normal/x.build.toml', 'rb'))['cursors']
    fallback = cursors.pop('fallback_settings')
    rules, made = [TEMPLATE], 0
    for cursor in cursors.values():
        name = cursor['x11_name']
        frames = sorted(glob.glob(f"{bibata}/svg/modern/{cursor['png'].replace('.png', '.svg')}")
                        + glob.glob(f"{bibata}/svg/modern/{name}/{cursor['png'].replace('.png', '.svg')}"))
        if not frames:
            sys.exit(f'no drawing for {name}')
        folder = f'Kavat/hyprcursors/{name}'
        os.makedirs(f'{out}/{folder}')
        meta = ['resize_algorithm = bilinear',
                f"hotspot_x = {cursor.get('x_hotspot', fallback['x_hotspot']) / SIDE:.4f}",
                f"hotspot_y = {cursor.get('y_hotspot', fallback['y_hotspot']) / SIDE:.4f}"]
        for other in cursor.get('x11_symlinks', []):
            meta.append(f'define_override = {other}')
        for frame in frames:
            text = open(frame, encoding='utf-8').read()
            if '{{' in text:
                sys.exit(f'{frame} has text the template would take for a value')
            for stand_in, token in COLOURS.items():
                text = text.replace(stand_in, token).replace(stand_in.lower(), token)
            file = os.path.basename(frame)
            open(f'{out}/{folder}/{file}', 'w', encoding='utf-8').write(text)
            rules.append(f'\n[[file]]\nfrom = "{folder}/{file}"\n')
            # a cursor of several drawings moves: each one stays this many milliseconds
            meta.append(f'define_size = 0, {file}' + (f", {fallback['x11_delay']}" if len(frames) > 1 else ''))
            made += 1
        open(f'{out}/{folder}/meta.hl', 'w', encoding='utf-8').write('\n'.join(meta) + '\n')
        rules.append(f'\n[[file]]\nfrom = "{folder}/meta.hl"\n')
    open(f'{out}/Kavat/manifest.hl', 'w', encoding='utf-8').write(MANIFEST)
    open(f'{out}/template.toml', 'w', encoding='utf-8').write(''.join(rules))
    print(f'{len(cursors)} cursors, {made} drawings')


if __name__ == '__main__':
    main()
