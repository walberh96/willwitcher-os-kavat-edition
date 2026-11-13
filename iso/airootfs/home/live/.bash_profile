# The live session goes straight to the desktop on the first console.
# Leaving the desktop lands here, in a shell.
[[ -f ~/.bashrc ]] && . ~/.bashrc

if [[ -z ${WAYLAND_DISPLAY:-} && ${XDG_VTNR:-} == 1 ]]; then
  # Autologin is quicker than the graphics driver: on real hardware the card may still be
  # loading from the USB stick, and Hyprland quits when it finds no screen. Wait for it.
  udevadm settle --timeout=60
  start-hyprland
fi
