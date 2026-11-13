# Shared by tools/build-repo.sh and tools/build-iso.sh. Sourced, not run.
#
# Packages and ISO are both put together inside clean Arch chroots, by Arch's own tools,
# from Arch's repositories as they were on the day in iso/arch-snapshot. The machine that
# runs the build lends its kernel and nothing else.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
snapshot="$(<"$root/iso/arch-snapshot")"
repo="$root/out/repo"
cache="$root/vendor/pacman-cache"
arch_conf="$root/work/arch-pacman.conf"    # Arch's repositories only
kavat_conf="$root/work/kavat-pacman.conf"  # Arch's and [kavat]

mkdir -p "$repo" "$cache" "$root/work"

# iso/pacman.conf with its marks filled in, with and without [kavat].
sed -e "s|@SNAPSHOT@|$snapshot|g" -e "s|@CACHE@|$cache|g" -e "s|@KAVAT_REPO@|$repo|g" \
  "$root/iso/pacman.conf" > "$kavat_conf"
sed -e '/^# Not signed yet/,$d' -e '/^\[kavat\]/,$d' "$kavat_conf" > "$arch_conf"

# pacman refuses a repository with no database, even an empty one.
if [[ ! -e $repo/kavat.db ]]; then
  bsdtar -cf - -T /dev/null | zstd -q > "$repo/kavat.db.tar.zst"
  ln -sf kavat.db.tar.zst "$repo/kavat.db"
fi

# arch_chroot <dir> <pacman.conf> <package ...>
# Make a chroot holding those packages, or bring an existing one up to the snapshot.
arch_chroot() {
  local dir="$1" conf="$2"
  shift 2
  if [[ ! -d $dir ]]; then
    # Laying down a bare Arch is the only step done by the build machine's own pacman, and
    # CachyOS's is not Arch's: it runs its own hooks in the chroot and writes a field of its
    # own in the package database, which Arch's pacman warns about. Leave both out.
    mkdir -p "$(dirname "$dir")" "$root/work/no-hooks"
    sed "s|^\[options\]$|&\nHookDir = $root/work/no-hooks|" "$arch_conf" > "$root/work/first-pacman.conf"
    mkarchroot -C "$root/work/first-pacman.conf" -c "$cache" "$dir" base
    sudo find "$dir/var/lib/pacman/local" -name desc -exec sed -i '/^%INSTALLED_DB%$/,/^$/d' {} +
  fi
  # From here on it is the chroot's pacman, Arch's, that installs and updates.
  # -uu also goes back, so the chroot follows the snapshot whichever way the date moved.
  arch-nspawn -C "$conf" -M /usr/share/devtools/makepkg.conf.d/x86_64.conf -c "$cache" "$dir" \
    --bind-ro="$repo" pacman -Syuu --needed --noconfirm "$@"
}
