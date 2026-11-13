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

root="$(cd "$(dirname "$0")/.." && pwd)"
snapshot="$(<"$root/iso/arch-snapshot")"
repo="$root/out/repo"
cache="$root/vendor/pacman-cache"
chroot="$root/work/chroot"
conf="$root/work/chroot-pacman.conf"

names=("$@")
if (( ${#names[@]} == 0 )); then
  for dir in "$root"/packages/*/; do names+=("$(basename "$dir")"); done
fi
for name in "${names[@]}"; do
  [[ -f $root/packages/$name/PKGBUILD ]] || { echo "packages/$name has no PKGBUILD" >&2; exit 1; }
done

mkdir -p "$repo" "$cache" "$chroot" "$root/work/logs"

# The chroot sees Arch only: [kavat] is what is being built.
sed -e "s|@SNAPSHOT@|$snapshot|g" -e "s|@CACHE@|$cache|g" -e '/^\[kavat\]/,$d' \
  "$root/iso/pacman.conf" > "$conf"

if [[ ! -d $chroot/root ]]; then
  mkarchroot -C "$conf" -M /usr/share/devtools/makepkg.conf.d/x86_64.conf -c "$cache" \
    "$chroot/root" base-devel
else
  # -uu also goes back, so the chroot follows the snapshot whichever way the date moved
  arch-nspawn -C "$conf" -c "$cache" "$chroot/root" pacman -Syuu --noconfirm
fi

for name in "${names[@]}"; do
  rm -f "$repo/$name"-[0-9]*.pkg.tar.zst
  (cd "$root/packages/$name" && PKGDEST="$repo" LOGDEST="$root/work/logs" makechrootpkg -c -r "$chroot")
done

rm -f "$repo"/kavat.db* "$repo"/kavat.files*
repo-add "$repo/kavat.db.tar.zst" "$repo"/*.pkg.tar.zst
