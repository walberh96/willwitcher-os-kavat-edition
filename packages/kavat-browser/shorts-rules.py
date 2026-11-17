#!/usr/bin/env python3
"""Make the rules that hide YouTube's Shorts into what the Kavat extension can apply.

usage: shorts-rules.py <list.txt>      the rules, as a script for the extension, on stdout

The list is gijsdev/ublock-hide-yt-shorts, written for uBlock Origin. The blocker of this
browser is uBlock Origin Lite, which takes no filters of one's own from the system's files,
so the Kavat extension hides the same parts of the page itself. A rule of the list is one of
three things here:

  css    a selector the browser understands, hidden with a style sheet. `A:upward(B)`, which
         is not CSS, is `B:has(:is(A))`, which is.
  path   the same, in force only where the page's path matches: `:matches-path(...)`.
  text   an element hidden for the words in it, `:has-text(...)`, which no style sheet can
         ask: the extension's script looks for them.

A rule that is none of these is left out and named on stderr; so is a rule that is not a
cosmetic one, since hiding is all the extension does.
"""
import json
import re
import sys

# what uBlock Origin adds to CSS and this does not translate
PROCEDURAL = (':has-text(', ':upward(', ':matches-path(', ':matches-css', ':matches-attr(', ':matches-media(',
              ':matches-prop(', ':min-text-length(', ':others(', ':remove(', ':remove-attr(', ':remove-class(',
              ':shadow(', ':spath(', ':style(', ':watch-attr(', ':xpath(', ':contains(', ':-abp-')


def closing(text, start):
    """Where the parenthesis opened at `start` closes; a regular expression may hold others."""
    depth = 0
    for at in range(start, len(text)):
        if text[at] == '\\':
            continue
        if text[at] == '(' and text[at - 1] != '\\':
            depth += 1
        elif text[at] == ')' and text[at - 1] != '\\':
            depth -= 1
            if depth == 0:
                return at
    return -1


def parts(selector):
    """A list of selectors, split at the commas that are not inside anything."""
    found, depth, last = [], 0, 0
    for at, letter in enumerate(selector):
        if letter in '([':
            depth += 1
        elif letter in ')]':
            depth -= 1
        elif letter == ',' and depth == 0:
            found.append(selector[last:at].strip())
            last = at + 1
    return found + [selector[last:].strip()]


def pattern(text):
    """What a rule matches against, as a regular expression and its flags: uBlock Origin
    takes /an expression/ or plain words."""
    found = re.fullmatch(r'/(.*)/([a-z]*)', text, re.S)
    if found:
        return found.group(1), found.group(2)
    return re.escape(text.strip('\'"')), ''


def rule(selector):
    """One selector of the list as a rule of ours; None when it cannot be one."""
    made = {}
    if selector.startswith(':matches-path('):
        end = closing(selector, len(':matches-path'))
        if end < 0:
            return None
        made['path'], flags = pattern(selector[len(':matches-path('):end])
        if flags:
            made['path_flags'] = flags
        selector = selector[end + 1:].strip()
    above = None
    if selector.endswith(')') and ':upward(' in selector:
        start = selector.rindex(':upward(')
        if closing(selector, start + len(':upward')) == len(selector) - 1:
            above = selector[start + len(':upward('):-1].strip()
            selector = selector[:start]
            if above.isdigit():
                return None  # so many elements up: not something a selector says
    if ':has-text(' in selector:
        # the words are looked for in `find`; what is hidden is the element around it that
        # the rule names, or the one with the words itself
        start = selector.rindex(':has-text(')
        end = closing(selector, start + len(':has-text'))
        if end < 0:
            return None
        made['text'], flags = pattern(selector[start + len(':has-text('):end])
        if flags:
            made['flags'] = flags
        holder, rest = selector[:start], selector[end + 1:]
        if rest == '':
            made['find'] = holder
            made['hide'] = above or ''
        elif rest == ')' and ':has(' in holder:
            # A:has(B:has-text(...)): the words are in a B inside an A, and the A is hidden
            open_at = holder.index(':has(')
            if closing(holder + selector[start:], open_at + len(':has')) != len(selector) - 1:
                return None
            outer, inner = holder[:open_at], holder[open_at + len(':has('):]
            made['find'] = f'{outer} {inner}'
            made['hide'] = above or outer
        else:
            return None
        if any(word in made['find'] + made['hide'] for word in PROCEDURAL):
            return None
        return made
    if above:
        if ':has(' in selector:
            return None  # CSS takes no :has() inside another
        # :is(), because what is inside :has() is only looked for within the element, and a
        # selector of the list may begin above it
        selector = f'{above}:has(:is({selector}))'
    if not selector or any(word in selector for word in PROCEDURAL):
        return None
    made['css'] = selector
    return made


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__.strip())
    rules, left = {}, []
    with open(sys.argv[1], encoding='utf-8') as file:
        for line in file:
            line = line.strip()
            if not line or line.startswith('!'):
                continue
            hosts, mark, selector = line.partition('##')
            if not mark or not hosts or '#' in hosts or '/' in hosts or '$' in hosts:
                left.append(line)  # a network rule, an exception, or a rule for every site
                continue
            for one in parts(selector):
                made = rule(one)
                if made is None:
                    left.append(f'{hosts}##{one}')
                    continue
                for host in hosts.split(','):
                    rules.setdefault(host.strip(), []).append(made)
    for line in left:
        print(f'shorts-rules: left out: {line}', file=sys.stderr)
    if not rules:
        sys.exit('shorts-rules: the list has no rule this can apply')
    print('// The rules that hide YouTube\'s Shorts, made from gijsdev/ublock-hide-yt-shorts when the')
    print('// package is built: for each site, what shorts.js hides there.')
    print(f'const KAVAT_SHORTS = {json.dumps(rules, indent=2)};')


if __name__ == '__main__':
    main()
