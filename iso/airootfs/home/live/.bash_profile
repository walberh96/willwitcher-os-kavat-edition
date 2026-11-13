# The live session goes straight to the desktop on the first console.
# Leaving the desktop lands here, in a shell.
[[ -f ~/.bashrc ]] && . ~/.bashrc

if [[ -z ${WAYLAND_DISPLAY:-} && ${XDG_VTNR:-} == 1 ]]; then
  start-hyprland
fi
