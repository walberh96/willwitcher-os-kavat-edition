#!/usr/bin/env python3
"""Check that every relative link in the README and in the documents under docs/ points to a file that exists.

usage: tools/check-links.py

Run it after editing README.md, docs/DESIGN.md or docs/PENDING.md. It exits with an error when a link is broken.
"""
import os
import re
import sys
import urllib.parse

repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
broken = False
for name in ('README.md', 'docs/DESIGN.md', 'docs/PENDING.md'):
    text = open(os.path.join(repo, name), encoding='utf-8').read()
    links = set(re.findall(r'\]\(([^)#]+)(?:#[^)]*)?\)', text))
    base = os.path.join(repo, os.path.dirname(name))
    missing = sorted(l for l in links if not l.startswith('http')
                     and not os.path.exists(os.path.join(base, urllib.parse.unquote(l))))
    print(name + ':', len(links), 'links, missing:', missing or 'none')
    broken = broken or bool(missing)
sys.exit(1 if broken else 0)
