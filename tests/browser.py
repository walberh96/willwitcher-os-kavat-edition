#!/usr/bin/env python3
"""Check the browser: Chromium made the system's own, with its blocker, its console and
its line to the system.

usage: tests/browser.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. Super+B has to open the browser, as a window of its own
name, with the blocker and the Kavat extension loaded from the system's files and the
start page of the system. The program the extension talks to has to answer with the mode
and the theme's colours, read your own commands, and save a clipping to the notes. A
command asked for from outside has to reach the open browser. A mode that hides the
browser has to put its window out of sight and bring it back. Widevine has to be there,
and links have to open in this browser.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about three minutes. Pictures of the screen are left in out/test/.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
HOME = f'/home/{USER}'
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME={HOME} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
KAVAT, BLOCKER = 'aelpfjkmocfiooliocnpmmbgcmbpjcne', 'ofdabgbaeilkjjdadjindcdlfonikhmo'
# what the browser has open, as it tells whoever debugs it: one line for each, its kind and its address
OPEN = 'curl -s http://127.0.0.1:9222/json/list | python3 -c "import json, sys; [print(t[\\"type\\"], t[\\"url\\"]) for t in json.load(sys.stdin)]"'
TITLES = OPEN.replace('"url', '"title')  # the same, with each page's title in place of its address
WINDOW = 'hyprctl --instance 0 clients | grep -A12 "class: kavat-browser$"'
# the workspace its window is on, which Hyprland says before the window's name
WORKSPACE = 'hyprctl --instance 0 clients | grep -B8 "class: kavat-browser$" | grep "workspace:"'
# Speak to the host as the extension does: each message is its length and then JSON.
HOST = r'''
import json, struct, subprocess, sys
host = subprocess.Popen(["/usr/lib/kavat/browser-host"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
def ask(message):
    data = json.dumps(message).encode()
    host.stdin.write(struct.pack("=I", len(data)) + data); host.stdin.flush()
    length = struct.unpack("=I", host.stdout.read(4))[0]
    return json.loads(host.stdout.read(length))
print(ask({"ask": "mode", "number": 1})["mode"])
theme = ask({"ask": "theme", "number": 2})["theme"]
print(theme["accent"], theme["mode"])
print(ask({"ask": "commands", "number": 3})["commands"])
print(ask({"ask": "clip", "number": 4, "title": "A page  worth keeping", "url": "https://example.org/x", "quote": "first line\nsecond line"}))
print(ask({"ask": "clip", "number": 5, "title": "Not a page", "url": "file:///etc/passwd"}))
host.stdin.close()
'''


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('browser', encrypted=len(sys.argv) > 1)
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

        print('the browser', flush=True)
        status, output = user('xdg-settings get default-web-browser; ls /usr/lib/chromium/WidevineCdm/_platform_specific/linux_x64/ | wc -l; '
                              f'grep -c "{BLOCKER}" /etc/chromium/policies/managed/kavat.json; grep -c "chrome-extension://{KAVAT}/" /etc/chromium/native-messaging-hosts/dev.kavat.browser.json; '
                              'ls /usr/share/kavat-browser/extensions/ublock/manifest.json /usr/share/kavat-browser/extensions/kavat/icon.png | wc -l')
        checks.report(output.split() == ['kavat-browser.desktop', '1', '1', '1', '2'],
                      'links open in it, and it comes with Widevine, its policies, its two extensions and its line to the system', output)
        machine.screen.press('super', 'b')
        until('Super+B opens it, as a window of its own name', f'{WINDOW} | grep -c "class: kavat-browser"', '1', 60)
        time.sleep(6)
        machine.screen.picture(f'{pictures}/browser-start.png')
        status, output = user('pgrep -f -c "[c]hromium.*--user-data-dir=/home/tester/.config/kavat-browser"; ls ~/.config/kavat-browser | grep -c Default')
        checks.report(output.split()[1:] == ['1'] and output.split()[0] != '0', 'it keeps its settings in a folder of its own', output)
        machine.screen.press('ctrl', 'q')
        machine.run('sleep 2; pkill -f user-data-dir=/home/tester/.config/kavat-browser; sleep 1; true')

        # opened again with a door to look in through, which only this machine can reach
        user('setsid -f kavat-browser --remote-debugging-port=9222 --remote-allow-origins=* > /dev/null 2>&1')
        until('its two extensions are loaded from the files of the system', OPEN + ' | grep -c -E "service_worker chrome-extension://(' + KAVAT + '|' + BLOCKER + ')/"', '2', 60)
        until("its first tab is the system's start page, not the browser's own",
              TITLES + ' | grep -c -x "page New tab"', '1', 20)
        status, output = user('printf "[[command]]\\nname = \\"docs\\"\\nopen = \\"https://wiki.archlinux.org\\"\\n" > ~/.config/kavat/browser/commands.toml; '
                              'cat > /tmp/host.py << "END"\n' + HOST + 'END\npython3 /tmp/host.py; grep -l "example.org/x" ~/Notes/inbox/*.md | wc -l; '
                              'cat "$(grep -l "example.org/x" ~/Notes/inbox/*.md | head -n 1)" | tr "\\n" "|"', 60)
        lines = output.split('\n')
        checks.report(lines[:2] == ['coder', '#f2a93b dark'] and "'name': 'docs'" in lines[2] and 'Saved to the notes' in lines[3] and 'failed' in lines[4]
                      and lines[5] == '1' and '# A page worth keeping' in lines[6] and '> first line|> second line' in lines[6] and '#clip' in lines[6],
                      "the system's side answers with the mode, the theme and your commands, and keeps a clipping as a note with its quote", output)
        before = user(OPEN + ' | grep -c "^page "')[1]
        status, output = user('kavat-browser command "new tab"; sleep 3; ' + OPEN + ' | grep -c "^page "; ' + OPEN + f' | grep -c "^page chrome-extension://{KAVAT}/start.html\\|^page chrome://newtab"')
        lines = output.split()
        checks.report(len(lines) == 2 and lines[0].isdigit() and before.isdigit() and int(lines[0]) == int(before) + 1 and lines[1] != '0',
                      'a command asked for from outside reaches the open browser, and a new tab is the start page of the system', f'{before} before\n{output}')
        time.sleep(2)
        machine.screen.picture(f'{pictures}/browser-tab.png')

        print('the modes', flush=True)
        status, output = user('mkdir -p ~/.config/kavat/modes && printf "name = \\"Quiet\\"\\nabout = \\"The browser out of sight\\"\\nhide_browser = true\\n" > ~/.config/kavat/modes/quiet.toml; '
                              f'kavat-mode quiet && sleep 1 && {WORKSPACE} | grep -c "special:kavat-hidden"; kavat-mode coder && sleep 1 && '
                              f'{WORKSPACE} | grep -c "special"')
        checks.report(output.split() == ['1', '0'], 'a mode that hides the browser puts its window out of sight, and leaving it brings it back', output)

        user('kavat-shell msg panel-open kavat/settings:panel browser > /dev/null')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/browser-page.png')
        status, output = user('kavat-shell msg panel-close kavat/settings:panel > /dev/null; grep -c -i "panel.luau:" ~/.cache/kavat-shell/kavat-shell.log; hyprctl --instance 0 configerrors')
        checks.report(output == '0', "the Browser page of the settings is drawn, and Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/browser-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
