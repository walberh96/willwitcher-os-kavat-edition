#!/usr/bin/env bash
# Build the ISO from the profile in iso/ into out/.
#
# usage: tools/build-iso.sh
#
# The ISO holds Arch's packages as they were on the day in iso/arch-snapshot, plus the
# packages of the local [kavat] repository, which tools/build-repo.sh builds first.
# mkarchiso runs inside a clean Arch chroot, so the pacman that installs the system and the
# tools that pack the image are Arch's, not this machine's. Needs devtools, and asks for sudo.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

chroot="$root/work/iso-chroot"
work="$root/work/iso"
conf="$root/work/iso-pacman.conf"

[[ -e $repo/kavat.db ]] || { echo "out/repo/ holds no [kavat] repository yet: run tools/build-repo.sh first" >&2; exit 1; }

sed -e "s|@SNAPSHOT@|$snapshot|g" -e "s|@CACHE@|$cache|g" -e "s|@KAVAT_REPO@|$repo|g" \
  "$root/iso/pacman.conf" > "$conf"

arch_chroot "$chroot" base archiso

# mkarchiso skips the steps a previous run finished, so a changed profile needs a clean start.
sudo rm -rf "$work"
mkdir -p "$work"

# The chroot sees the profile, the pacman.conf, the work folder and out/, at the same paths
# they have here. The ISO carries the snapshot's date, not today's.
arch-nspawn -C "$arch_conf" -c "$cache" "$chroot" \
  --bind-ro="$root/iso" --bind-ro="$conf" --bind="$work" --bind="$root/out" \
  --setenv=TZ=UTC --setenv=SOURCE_DATE_EPOCH="$(date -u -d "${snapshot//\//-}" +%s)" \
  mkarchiso -v -C "$conf" -w "$work" -o "$root/out" "$root/iso"
sudo chown "$(id -u):$(id -g)" "$root"/out/*.iso
