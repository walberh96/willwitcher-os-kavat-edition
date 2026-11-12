#!/usr/bin/env python3
"""Regenerate docs/DESIGN.md from a Markdown export of the design document, then check links.

The design document lives in claude.ai; docs/DESIGN.md is a copy and is never edited by hand.

usage: tools/sync-design.py <export> [text to count ...]

<export> is one of:
  - a Markdown file exported from the design document,
  - the JSON result of the Claude Docs export tool (it carries the Markdown as base64),
  - a directory: the newest `mcp-claude_ai_Claude_Docs-export-*.txt` inside it is used.

Each extra argument is a piece of text; the script prints how many times it appears in the
result, which is a quick way to confirm that an edit made it into the export.
"""
import base64
import glob
import json
import os
import re
import sys
import urllib.parse

if len(sys.argv) < 2:
    sys.exit(__doc__)

repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
source = sys.argv[1]
probes = sys.argv[2:]

if os.path.isdir(source):
    found = glob.glob(os.path.join(source, 'mcp-claude_ai_Claude_Docs-export-*.txt'))
    if not found:
        sys.exit('no export found in ' + source)
    source = max(found, key=os.path.getmtime)

raw = open(source, encoding='utf-8').read()
rev = '?'
try:
    data = json.loads(raw)
except ValueError:
    md = raw  # already Markdown
else:
    if isinstance(data, list):  # a list of content blocks, one of which holds the JSON
        for block in data:
            try:
                cand = json.loads(block.get('text', ''))
            except (ValueError, AttributeError):
                continue
            if isinstance(cand, dict) and 'data' in cand:
                data = cand
                break
    md = base64.b64decode(data['data']['bytes_b64']).decode('utf-8')
    rev = data.get('rev', '?')

# What the export cannot carry: the repo's own title and byline, the two diagrams as images,
# and links to files that live next to the document in the repo.
for old, new in [
    ('# Willwitcher-OS Kavat Edition — Propuestas de diseño', '# Willwitcher-OS Kavat Edition — Documento de diseño'),
    ('&#91;embedded content: arquitectura · 5 capas y el agente\\]', '![Arquitectura: cinco capas y el agente](img/architecture.png)'),
    ('&#91;embedded content: roadmap · 5 fases, 4 puertas\\]', '![Roadmap: cinco fases y cuatro puertas](img/roadmap.png)'),
    ('`reports/Ideas de otras distros Linux.md` del repo',
     '[`reports/Ideas de otras distros Linux.md`](../reports/Ideas%20de%20otras%20distros%20Linux.md)'),
    ('`reports/Modelo local para el agente Kavat.md` del repo',
     '[`reports/Modelo local para el agente Kavat.md`](../reports/Modelo%20local%20para%20el%20agente%20Kavat.md)'),
]:
    if md.count(old) != 1:
        sys.exit('expected exactly once, found %d: %s' % (md.count(old), old))
    md = md.replace(old, new)

# The byline is the line under the title: an export date and a mention chip.
byline = re.compile(r'^([A-Z][a-z]{2}) (\d{1,2}), (\d{4}) · @\S+$', re.M)
found = byline.findall(md)
if len(found) != 1:
    sys.exit('byline not found under the title')
months = {'Jan': 'enero', 'Feb': 'febrero', 'Mar': 'marzo', 'Apr': 'abril', 'May': 'mayo', 'Jun': 'junio',
          'Jul': 'julio', 'Aug': 'agosto', 'Sep': 'septiembre', 'Oct': 'octubre', 'Nov': 'noviembre', 'Dec': 'diciembre'}
month, day, year = found[0]
md = byline.sub('Actualizado el %d de %s de %s · [Mockup interactivo](mockup/index.html) · '
                '[Capturas](screenshots/) · [README](../README.md)' % (int(day), months[month], year), md)

open(os.path.join(repo, 'docs', 'DESIGN.md'), 'w', encoding='utf-8').write(md)
print('docs/DESIGN.md: rev', rev, '·', md.count('\n') + 1, 'lines', {k: md.count(k) for k in probes} if probes else '')

for name in ('README.md', 'docs/DESIGN.md'):
    text = open(os.path.join(repo, name), encoding='utf-8').read()
    links = set(re.findall(r'\]\(([^)#]+)(?:#[^)]*)?\)', text))
    base = os.path.join(repo, os.path.dirname(name))
    missing = sorted(l for l in links if not l.startswith('http')
                     and not os.path.exists(os.path.join(base, urllib.parse.unquote(l))))
    print(name + ':', len(links), 'links, missing:', missing or 'none')
