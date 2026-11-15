#!/usr/bin/env python3
"""Make the theme's icons out of Papirus: make-template.py <Papirus> <template> <styles>

<Papirus> is the folder of the Papirus icon theme. Two things are made from it.

In <template>, kavat-theme's template for the Colour style. Papirus draws its folders in a
colour, blue unless told otherwise; this takes each blue drawing, in the sizes folders are
shown at, and writes it with the theme's accent where the blue was. What comes out is a
small icon theme, Kavat, that holds only those and gets the rest from Papirus.

In <styles>, the drawings of the other two styles, with marks where their colours go, for
kavat-icons-apply to fill in when one of them is chosen: there are thousands, too many to
write again on every change of theme. Tinted is every app's icon with each of its colours
replaced by the palette's colour of the same hue, in a lighter or a deeper shade as the
drawing had it. Line is Papirus' own outline drawings, which exist for the folders and for
some apps, in the colour of the text.
"""
import colorsys
import re
import shutil
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
# It is started and left to it: the other styles are thousands of drawings.
reload = [["setsid", "-f", "kavat-icons-apply"]]

[[file]]
from = "Kavat/index.theme"
folder_link = "data:icons/Kavat"

[[file]]
from = "colours"
'''


# The style and the colours, as kavat-icons-apply reads them: a name and a value to a line
COLOURS_FILE = '''style {{ icon_style }}
mode {{ mode_title }}
TEXT {{ text }}
''' + ''.join(f'{hue.upper()}{mark} {{{{ {hue}{token} }}}}\n'
                 for hue in ('red', 'yellow', 'green', 'cyan', 'blue', 'magenta')
                 for mark, token in (('_L', '_light'), ('', ''), ('_D', '_deep')))

STYLE_INDEX = '''[Icon Theme]
Name=%s
Comment=%s
Inherits=%s
Example=folder
Directories=%s

'''
SCALABLE = '''[%s]
Context=%s
Size=48
MinSize=8
MaxSize=512
Type=Scalable

'''


def mark_of(colour):
    """The mark that stands for a colour in a tinted drawing; none for one with no hue."""
    red, green, blue = (int(colour[at:at + 2], 16) / 255 for at in (1, 3, 5))
    hue, light, saturation = colorsys.rgb_to_hls(red, green, blue)
    if saturation < 0.18 or light < 0.08 or light > 0.94:
        return None
    degrees = hue * 360
    name = ('RED' if degrees < 20 or degrees >= 340 else 'YELLOW' if degrees < 70 else 'GREEN' if degrees < 165
            else 'CYAN' if degrees < 200 else 'BLUE' if degrees < 260 else 'MAGENTA')
    return f'@{name}{"_D" if light < 0.38 else "_L" if light > 0.68 else ""}@'


def tinted(text):
    return re.sub(r'#[0-9a-fA-F]{6}\b', lambda found: mark_of(found.group()) or found.group(), text)


def styles(papirus, out):
    """The Tinted and the Line drawings, with marks for their colours."""
    made = {'tinted': 0, 'line': 0}
    apps = f'{papirus}/48x48/apps'
    os.makedirs(f'{out}/tinted/apps')
    for name in sorted(os.listdir(apps)):
        path = f'{apps}/{name}'
        if os.path.islink(path):
            # another name for the same drawing stays a link, when what it names is here too
            target = os.readlink(path)
            if '/' not in target:
                os.symlink(target, f'{out}/tinted/apps/{name}')
        elif name.endswith('.svg'):
            open(f'{out}/tinted/apps/{name}', 'w', encoding='utf-8').write(tinted(open(path, encoding='utf-8').read()))
            made['tinted'] += 1
    open(f'{out}/tinted/index.theme', 'w').write(
        STYLE_INDEX % ('Kavat-Tinted', "Papirus' app icons in the palette's colours", 'Kavat,Papirus-@MODE@,Papirus,hicolor', 'apps')
        + SCALABLE % ('apps', 'Applications'))

    # the outline drawings are named after the icon they stand for, with -symbolic at the end
    for kind, context in (('places', 'Places'), ('apps', 'Applications')):
        os.makedirs(f'{out}/line/{kind}')
        folder = f'{papirus}/16x16/symbolic/{kind}'
        for name in sorted(os.listdir(folder)):
            if not name.endswith('-symbolic.svg'):
                continue
            text = open(os.path.realpath(f'{folder}/{name}'), encoding='utf-8').read()
            # Papirus names the colour of the text once, for the whole drawing
            text = re.sub(r'(\.ColorScheme-Text\s*\{\s*color:)#[0-9a-fA-F]{6}', r'\1@TEXT@', text)
            open(f"{out}/line/{kind}/{name.replace('-symbolic.svg', '.svg')}", 'w', encoding='utf-8').write(text)
            made['line'] += 1
    open(f'{out}/line/index.theme', 'w').write(
        STYLE_INDEX % ('Kavat-Line', "Papirus' outline icons in the colour of the text", 'Papirus-@MODE@,Papirus,hicolor', 'places,apps')
        + SCALABLE % ('places', 'Places') + SCALABLE % ('apps', 'Applications'))
    return made


def main():
    papirus, out, other = sys.argv[1:]
    shutil.rmtree(out, ignore_errors=True)
    shutil.rmtree(other, ignore_errors=True)
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
    open(f'{out}/colours', 'w', encoding='utf-8').write(COLOURS_FILE)
    print(f'{made} drawings of folders; {styles(papirus, other)}')


if __name__ == '__main__':
    main()
