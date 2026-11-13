# Shared by tools/build-repo.sh and tools/build-iso.sh. Sourced, not run.
#
# Packages and ISO are both put together inside clean Arch chroots, by Arch's own tools,
# from Arch's repositories as they were on the day in iso/arch-snapshot. The machine that
# runs the build lends its kernel and nothing else.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
snapshot="$(<"$root/iso/arch-snapshot")"
repo="$root/out/repo"
cache="$root/vendor/pacman-cache"
arch_conf="$root/work/arch-pacman.conf"

mkdir -p "$repo" "$cache" "$root/work"

# iso/pacman.conf with its marks filled in. What the chroots themselves use is Arch only.
sed -e "s|@SNAPSHOT@|$snapshot|g" -e "s|@CACHE@|$cache|g" -e '/^\[kavat\]/,$d' \
  "$root/iso/pacman.conf" > "$arch_conf"

# arch_chroot <dir> <package ...>
# Make a chroot holding those packages, or bring an existing one up to the snapshot.
arch_chroot() {
  local dir="$1"
  shift
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
  arch-nspawn -C "$arch_conf" -M /usr/share/devtools/makepkg.conf.d/x86_64.conf -c "$cache" "$dir" \
    pacman -Syuu --needed --noconfirm "$@"
}
