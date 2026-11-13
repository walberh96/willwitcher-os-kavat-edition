#!/usr/bin/env bash
# Build the ISO from the profile in iso/ into out/.
#
# usage: tools/build-iso.sh
#
# The ISO holds Arch's packages as they were on the day in iso/arch-snapshot, plus the
# packages of the local [kavat] repository, which tools/build-repo.sh builds first.
# Needs archiso, and asks for sudo.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
snapshot="$(<"$root/iso/arch-snapshot")"
repo="$root/out/repo"
cache="$root/vendor/pacman-cache"
work="$root/work/iso"
conf="$root/work/iso-pacman.conf"

[[ -e $repo/kavat.db ]] || { echo "out/repo/ holds no [kavat] repository yet: run tools/build-repo.sh first" >&2; exit 1; }

mkdir -p "$cache" "$root/work" "$root/out"
sed -e "s|@SNAPSHOT@|$snapshot|g" -e "s|@CACHE@|$cache|g" -e "s|@KAVAT_REPO@|$repo|g" \
  "$root/iso/pacman.conf" > "$conf"

# mkarchiso skips the steps a previous run finished, so a changed profile needs a clean start.
sudo rm -rf "$work"
# The ISO carries the snapshot's date, not today's.
sudo env SOURCE_DATE_EPOCH="$(date -u -d "${snapshot//\//-}" +%s)" \
  mkarchiso -v -C "$conf" -w "$work" -o "$root/out" "$root/iso"
sudo chown "$(id -u):$(id -g)" "$root"/out/*.iso
