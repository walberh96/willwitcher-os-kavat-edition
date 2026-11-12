#!/usr/bin/env bash
# Retake the screenshots in docs/screenshots/ from the interactive mockup.
#
# usage: tools/screenshots.sh [name ...]
#   no names     retake all of them
#   with names   retake only those, e.g. tools/screenshots.sh notes settings-agent
#
# Needs Chrome or Chromium; it runs without a window. The mockup shows the current
# time in its bar, so a retaken screenshot always differs from the committed one.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

browser=""
for candidate in google-chrome-stable google-chrome chromium chromium-browser \
                 "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"; do
  if command -v "$candidate" >/dev/null 2>&1 || [ -x "$candidate" ]; then
    browser="$candidate"
    break
  fi
done
[ -n "$browser" ] || { echo "Chrome or Chromium not found" >&2; exit 1; }

# name|state of the mockup (the part of the address after #bare=true&)
shots=(
  "agent|scene=agent"
  "agent-voice|scene=voice&voice=speak"
  "boot-menu|scene=grub&gsel=safe"
  "browser|scene=browser"
  "browser-albums|scene=albums"
  "browser-clip|scene=browser&clip=true"
  "claude-panel|scene=claude"
  "config-in-neovim|scene=editor&cfg=windows"
  "desktop|scene=desk"
  "editor-marketplace|scene=editor"
  "files|scene=files"
  "files-copy|scene=files&copy=true"
  "files-picker|scene=files&pick=true"
  "installer|scene=install&pal=tokyo"
  "installer-software|scene=install&istep=software"
  "installer-software-offline|scene=install&istep=software&inet=false"
  "kav-aur-review|scene=kav&ktab=aur"
  "kav-dashboard|scene=kav&ktab=dash"
  "kav-updates|scene=kav&ktab=updates"
  "launcher|scene=launcher"
  "launcher-calc|scene=launcher&calc=true"
  "loading|scene=boot"
  "login|scene=login"
  "note-rendered|scene=note"
  "notes|scene=notes"
  "notes-quick|scene=desk&cap=true&rem=true"
  "notices|scene=desk&notices=true"
  "password-dialog|scene=auth"
  "settings-agent|scene=settings&spage=agent"
  "settings-backup|scene=settings&spage=backup"
  "settings-boot|scene=settings&spage=boot"
  "settings-displays|scene=settings&spage=displays&dsel=1"
  "settings-gpu|scene=settings&spage=gpu"
  "settings-modes|scene=settings&spage=modes"
  "settings-network|scene=settings&spage=network"
  "settings-notes|scene=settings&spage=notes"
  "settings-privacy|scene=settings&spage=privacy"
  "settings-projects|scene=settings&spage=projects"
  "settings-shell|scene=settings&spage=shell"
  "settings-skills|scene=settings&spage=skills"
  "settings-terminal|scene=settings&spage=terminal"
  "settings-wallpaper|scene=settings&spage=wallpaper&wall=aurora"
  "settings-windows|scene=settings&spage=windows"
  "system-monitor|scene=top"
  "system-monitor-history|scene=top&ttab=history"
  "system-monitor-services|scene=top&ttab=services"
  "themes-light|scene=themes&pal=rose&mode=light"
)

taken=0
for spec in "${shots[@]}"; do
  name="${spec%%|*}"
  state="${spec#*|}"
  if [ "$#" -gt 0 ]; then
    case " $* " in *" $name "*) ;; *) continue ;; esac
  fi
  "$browser" --headless=new --disable-gpu --hide-scrollbars --window-size=1600,1000 \
    --screenshot="$root/docs/screenshots/$name.png" \
    "file://$root/docs/mockup/index.html#bare=true&$state" >/dev/null 2>&1
  echo "$name"
  taken=$((taken + 1))
done

if [ "$taken" -eq 0 ]; then
  echo "no screenshot with that name; see the list in this script" >&2
  exit 1
fi
