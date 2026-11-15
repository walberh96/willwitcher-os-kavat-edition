#!/usr/bin/env python3
"""Make kavat-theme's template for the icons out of Papirus: make-template.py <Papirus> <out>

<Papirus> is the folder of the Papirus icon theme and <out> the template's folder. Papirus
draws its folders in a colour, blue unless told otherwise; this takes each blue drawing, in
the sizes folders are shown at, and writes it with the theme's accent where the blue was.
What comes out is a small icon theme, Kavat, that holds only those and gets the rest from
Papirus.
"""
import os
import sys

SIZES = [22, 24, 32, 48, 64]
# Papirus' blues, and what takes their place: a folder's front, its back, and what is drawn on it
COLOURS = {'#5294e2': '{{ accent }}', '#4877b1': '{{ accent_deep }}', '#1d344f': '{{ on_accent }}'}

INDEX = '''# Made by kavat-theme from the theme's settings; what is changed here is lost.
[Icon Theme]
Name=Kavat
Comment=Papirus, with the folders in the accent of the system's theme
Inherits=Papirus-{{ mode_title }},Papirus,hicolor
Example=folder
Directories=%s

'''
FOLDER = '''[%dx%d/places]
Context=Places
Size=%d
Type=Fixed

'''
TEMPLATE = '''# The icons: the folders, in the theme's accent. This file is made by kavat-icons from
# Papirus, one entry for each drawing.
#
# The icon theme is called Kavat and gets every other icon from Papirus. Apps are told its
# name again on every change, which is when they look at the drawings again.
reload = [["sh", "-c", "gsettings set org.gnome.desktop.interface icon-theme Papirus; gsettings set org.gnome.desktop.interface icon-theme Kavat"]]

[[file]]
from = "Kavat/index.theme"
folder_link = "data:icons/Kavat"
'''


def main():
    papirus, out = sys.argv[1:]
    rules, made = [TEMPLATE], 0
    for size in SIZES:
        places = f'{papirus}/{size}x{size}/places'
        os.makedirs(f'{out}/Kavat/{size}x{size}/places')
        for name in sorted(os.listdir(places)):
            drawing = os.path.realpath(f'{places}/{name}')
            if 'blue' not in os.path.basename(drawing) or not name.endswith('.svg'):
                continue
            text = open(drawing, encoding='utf-8').read()
            if '{{' in text:
                sys.exit(f'{drawing} has text the template would take for a value')
            for blue, token in COLOURS.items():
                text = text.replace(blue, token).replace(blue.upper(), token)
            place = f'Kavat/{size}x{size}/places/{name}'
            open(f'{out}/{place}', 'w', encoding='utf-8').write(text)
            rules.append(f'\n[[file]]\nfrom = "{place}"\n')
            made += 1
    index = INDEX % ','.join(f'{size}x{size}/places' for size in SIZES)
    index += ''.join(FOLDER % (size, size, size) for size in SIZES)
    open(f'{out}/Kavat/index.theme', 'w', encoding='utf-8').write(index)
    open(f'{out}/template.toml', 'w', encoding='utf-8').write(''.join(rules))
    print(f'{made} drawings')


if __name__ == '__main__':
    main()
