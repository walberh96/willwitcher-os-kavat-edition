#!/usr/bin/env bash
# Build the packages in packages/ and collect them in out/repo/, the local [kavat] repository.
#
# usage: tools/build-repo.sh [package ...]
#   no names     build all of them
#   with names   build only those, e.g. tools/build-repo.sh kavat-hyprland
#
# Every package is built in a clean Arch chroot, against Arch's repositories as they were on
# the day in iso/arch-snapshot. Nothing comes from the machine that runs this: not its
# repositories, not its makepkg.conf. Needs devtools, and asks for sudo.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

chroot="$root/work/chroot"

names=("$@")
if (( ${#names[@]} == 0 )); then
  for dir in "$root"/packages/*/; do names+=("$(basename "$dir")"); done
fi
for name in "${names[@]}"; do
  [[ -f $root/packages/$name/PKGBUILD ]] || { echo "packages/$name has no PKGBUILD" >&2; exit 1; }
done

arch_chroot "$chroot/root" base-devel

mkdir -p "$root/work/logs"
for name in "${names[@]}"; do
  rm -f "$repo/$name"-[0-9]*.pkg.tar.zst
  (cd "$root/packages/$name" && PKGDEST="$repo" LOGDEST="$root/work/logs" makechrootpkg -c -r "$chroot")
done

rm -f "$repo"/kavat.db* "$repo"/kavat.files*
repo-add "$repo/kavat.db.tar.zst" "$repo"/*.pkg.tar.zst
