#!/usr/bin/env bash
# Build the kavat-* packages and collect them in out/repo/, the local [kavat] repository.
#
# usage: tools/build-repo.sh [folder ...]
#   no folders     build all of them
#   with folders   build only those, e.g. tools/build-repo.sh kavat-hyprland installer
#
# A package is a folder with a PKGBUILD: one of those in packages/, or a piece of the system
# that carries its own, like installer/. They are built in that order, and each one can
# depend on the ones before it.
#
# Every package is built in a clean Arch chroot, against Arch's repositories as they were on
# the day in iso/arch-snapshot. Nothing comes from the machine that runs this: not its
# repositories, not its makepkg.conf. Needs devtools, and asks for sudo.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

chroot="$root/work/chroot"

dirs=()
if (( $# == 0 )); then
  for file in "$root"/packages/*/PKGBUILD "$root"/*/PKGBUILD; do dirs+=("$(dirname "$file")"); done
else
  for name in "$@"; do
    if [[ -f $root/packages/$name/PKGBUILD ]]; then dirs+=("$root/packages/$name")
    elif [[ -f $root/$name/PKGBUILD ]]; then dirs+=("$root/$name")
    else echo "no PKGBUILD in packages/$name or in $name" >&2; exit 1
    fi
  done
fi

arch_chroot "$chroot/root" "$kavat_conf" base-devel

mkdir -p "$root/work/logs"
for dir in "${dirs[@]}"; do
  name="$(sed -n "s/^pkgname=//p" "$dir/PKGBUILD" | tr -d "'\"")"
  rm -f "$repo/$name"-[0-9]*.pkg.tar.zst
  # -u brings the build's copy of the chroot up to date, so it sees the packages built so far
  (cd "$dir" && PKGDEST="$repo" LOGDEST="$root/work/logs" makechrootpkg -c -u -r "$chroot" -D "$repo")
  rm -f "$repo"/kavat.db* "$repo"/kavat.files*
  repo-add -q "$repo/kavat.db.tar.zst" "$repo"/*.pkg.tar.zst
done
