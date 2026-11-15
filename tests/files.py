#!/usr/bin/env python3
"""Check the file manager: Yazi as kavat-files sets it up, and as the system's file window.

usage: tests/files.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. It logs in and opens Yazi with Super+E: Ctrl+C and Ctrl+V have
to copy a file and Delete has to send it to the trash. Then it asks the desktop for a file,
as an app does when it says "open": Yazi has to come up as a floating window, and the file
picked there with Enter has to be the answer the app gets. Last, it asks the file manager
to show a file, as an app does with "show in folder", and Yazi has to open there.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes. Pictures of the screen are left in out/test/.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
HOME = f'/home/{USER}'
FILE = '0-first.txt'  # sorts before anything else in the user's folder
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME={HOME} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
WINDOWS = 'hyprctl --instance 0 clients'
# An app that asks the desktop for a file, and writes down what it is given. The answer
# comes back to the one who asked and to nobody else, so it has to be a program of its own.
ASK = '''
import sys
from gi.repository import Gio, GLib

bus = Gio.bus_get_sync(Gio.BusType.SESSION)
loop = GLib.MainLoop()


def answered(connection, sender, path, interface, signal, parameters):
    code, results = parameters.unpack()
    print(code, " ".join(results.get("uris", [])), flush=True)
    loop.quit()


token = "kavat_test"
request = "/org/freedesktop/portal/desktop/request/" + bus.get_unique_name()[1:].replace(".", "_") + "/" + token
bus.signal_subscribe("org.freedesktop.portal.Desktop", "org.freedesktop.portal.Request", "Response", request, None,
                     Gio.DBusSignalFlags.NONE, answered)
bus.call_sync("org.freedesktop.portal.Desktop", "/org/freedesktop/portal/desktop", "org.freedesktop.portal.FileChooser",
              "OpenFile", GLib.Variant("(ssa{sv})", ("", "Open a file", {"handle_token": GLib.Variant("s", token)})),
              None, Gio.DBusCallFlags.NONE, -1, None)
print("asked", flush=True)
GLib.timeout_add_seconds(120, loop.quit)
loop.run()
'''
SHOW = ('gdbus call --session --dest org.freedesktop.FileManager1 --object-path /org/freedesktop/FileManager1 '
        '--method org.freedesktop.FileManager1.ShowItems "[\'file:///etc/os-release\']" ""')


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('files', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    def until(what, command, wanted, timeout=20):
        deadline = time.time() + timeout
        _, found = user(command)
        while wanted not in found and time.time() < deadline:
            time.sleep(0.5)
            _, found = user(command)
        return checks.report(wanted in found, what, f'wanted {wanted}, found {found}')

    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        log_in(machine, PASSWORD)
        check_desktop(machine, checks, USER)

        print('Yazi as the file manager', flush=True)
        status, output = user('grep -c smart-enter ~/.config/yazi/keymap.toml; ls ~/.config/yazi/plugins; '
                              'readlink ~/.config/yazi/theme.toml; grep -c "#f2a93b" ~/.config/yazi/theme.toml')
        checks.report(status == 0 and 'smart-enter.yazi' in output and '/kavat/theme/current/yazi/theme.toml' in output,
                      "the user has the system's settings for Yazi, and its colours come from the theme", output)
        user(f'echo picked > ~/{FILE}')
        machine.screen.press('super', 'e')
        until('Super+E opens Yazi', 'pgrep -c -x yazi', '1')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/files.png')
        machine.screen.press('ctrl', 'c')
        until('Ctrl+C leaves the file on the clipboard too, as a file',
              'wl-paste --type text/uri-list | tr -d "\r"', f'file://{HOME}/{FILE}')
        machine.screen.press('ctrl', 'v')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/files-paste.png')
        status, output = user('ls ~ | grep -c "^0-first"')
        checks.report(output == '1', 'Ctrl+V, where that name is taken, asks before it does anything', output)
        machine.screen.press('k')
        until('and "keep both" copies the file under a name of its own', 'ls ~ | grep -c "^0-first"', '2')
        status, output = user(f'cd ~/.config/yazi/plugins && ls; grep -c "plugin paste" ../keymap.toml')
        checks.report('paste.yazi' in output and 'copy.yazi' in output and output.endswith('1'),
                      'the copy window and the clipboard are plugins of the user, on their keys', output)
        machine.screen.press('delete')
        time.sleep(1)
        machine.screen.press('y')
        until('Delete sends one of the two to the trash', 'ls ~/.local/share/Trash/files/ 2>&1; ls ~ | grep -c "^0-first"', '0-first')
        until('and takes it out of the folder', 'echo "left: $(ls ~ | grep -c "^0-first")"', 'left: 1')
        user(f'rm -f ~/0-first*; echo picked > ~/{FILE}')
        machine.screen.press('q')
        until('q leaves Yazi', 'pgrep -c -x yazi; echo left', '0\nleft')

        print('Yazi as the window an app picks a file with', flush=True)
        machine.run(f"cat > /tmp/ask.py << 'END'\n{ASK}END\nchmod 644 /tmp/ask.py")
        user('setsid -f python3 /tmp/ask.py > /tmp/portal.log 2>&1')
        until('the desktop takes the request for a file', 'cat /tmp/portal.log', 'asked')
        opened = until('Yazi comes up for it, floating over the desktop',
                       f'{WINDOWS} | grep -B6 "class: kavat-picker" | grep -E "size|floating" | tr -d "\\n\\t"',
                       'size: 1344,756floating: 1')
        if opened:
            time.sleep(3)
            machine.screen.picture(f'{pictures}/files-picker.png')
            machine.screen.press('enter')
            until('the file picked there is the answer the app gets', 'cat /tmp/portal.log', f'0 file://{HOME}/{FILE}')
            until('and the window closes', f'{WINDOWS} | grep -c "class: kavat-picker"; echo closed', '0\nclosed')

        print('"show in folder"', flush=True)
        status, output = user(SHOW)
        checks.report(status == 0, 'the file manager answers an app that asks to show a file', output)
        until('and Yazi opens at that file', 'pgrep -a -x yazi', '/etc/os-release')
        time.sleep(2)
        machine.screen.picture(f'{pictures}/files-show.png')
        status, output = user('hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/files*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
