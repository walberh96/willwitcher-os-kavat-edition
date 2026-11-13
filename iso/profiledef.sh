#!/usr/bin/env bash
# shellcheck disable=SC2034
# archiso profile of Willwitcher-OS Kavat Edition. Build it with tools/build-iso.sh, which
# fills in pacman.conf; mkarchiso cannot build this folder directly.

iso_name="willwitcher-os-kavat"
iso_label="KAVAT_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Willwitcher-OS Kavat Edition"
iso_application="Willwitcher-OS Kavat Edition live medium"
iso_version="$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)"
install_dir="kavat"
# UEFI only: the installer asks for it anyway. systemd-boot comes from the image's own
# packages; GRUB would be taken from the machine that runs the build.
bootmodes=('uefi.systemd-boot')
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '15' '-b' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/etc/gshadow"]="0:0:400"
  ["/etc/sudoers.d"]="0:0:750"
  ["/etc/sudoers.d/live"]="0:0:440"
)
