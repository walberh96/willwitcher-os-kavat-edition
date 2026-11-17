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
print(ask({"ask": "clip", "number": 4, "title": "A page  worth keeping", "url": "https://example.org/x", "quote": "first line\nsecond line", "tags": "reading"}))
print(ask({"ask": "clip", "number": 5, "title": "Not a page", "url": "file:///etc/passwd"}))
print(ask({"ask": "clip", "number": 6, "title": "A page worth keeping", "url": "https://example.org/x", "comment": "worth a\nsecond look", "tags": "#reading, rust"})["said"])
host.stdin.close()
'''
# The albums, asked of the host the same way: a site is added twice and is there once, with
# its picture; the start page's album is a line of the settings file; the notes about a site.
# The start page's background, asked of the host as the extension does: the desktop's as a
# picture, kept for the next time, then a picture handed over by the page.
BACKGROUND = r'''
import base64, importlib.machinery, importlib.util, os
loader = importlib.machinery.SourceFileLoader("host", "/usr/lib/kavat/browser-host")
host = importlib.util.module_from_spec(importlib.util.spec_from_loader("host", loader))
loader.exec_module(host)
first = host.answer({"ask": "start-background"})
print(first.get("picture", "")[:23])
print("kept" if first.get("stamp") and first["stamp"] == host.answer({"ask": "settings"})["settings"]["start_background_stamp"] and len(first["picture"]) < 1000000 else "not kept")
with open("/tmp/album.png", "rb") as file:
    own = "data:image/png;base64," + base64.b64encode(file.read()).decode()
host.answer({"ask": "start-background-set", "picture": own})
print("own", host.answer({"ask": "start-background"}).get("picture", "")[:23])
'''

ALBUMS = r'''
import base64, json, struct, subprocess, sys
host = subprocess.Popen(["/usr/lib/kavat/browser-host"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
def ask(message):
    data = json.dumps(message).encode()
    host.stdin.write(struct.pack("=I", len(data)) + data); host.stdin.flush()
    length = struct.unpack("=I", host.stdout.read(4))[0]
    return json.loads(host.stdout.read(length))
picture = "data:image/png;base64," + base64.b64encode(open("/tmp/album.png", "rb").read()).decode()
print(ask({"ask": "album-add", "number": 1, "name": "reading", "title": "A site", "url": "https://example.org/site", "picture": picture}).get("said"))
ask({"ask": "album-add", "number": 2, "name": "reading", "title": "A site, again", "url": "https://example.org/site"})
print([album for album in ask({"ask": "albums", "number": 3})["albums"] if album["name"] == "reading"])
sites = ask({"ask": "album", "number": 4, "name": "reading"})["sites"]
print(len(sites), sites[0]["title"], sites[0]["url"])
print(ask({"ask": "album-picture", "number": 5, "file": sites[0]["picture"]}).get("picture", "")[:23])
print(ask({"ask": "album-picture", "number": 6, "file": "/etc/passwd"}).get("failed"))
print(ask({"ask": "start-album", "number": 7, "name": "reading"})["settings"])
print(sorted({note["kind"] for note in ask({"ask": "about", "number": 8, "url": "https://example.org/other"})["notes"]}))
ask({"ask": "album-remove", "number": 9, "name": "reading", "url": "https://example.org/site"})
print(len(ask({"ask": "album", "number": 10, "name": "reading"})["sites"]))
host.stdin.close()
'''
# the colour of a point of the screen, as six letters
POINT = 'grim -g "{x},{y} 1x1" -t ppm - | tail -c 3 | od -An -tx1 | tr -d " \\n"'


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
                      and lines[5] == 'Saved to the notes' and lines[6] == '1' and '# A page worth keeping' in lines[7] and '> first line|> second line' in lines[7]
                      and '|#clip #reading #rust|' in lines[7] and '|worth a second look|' in lines[7],
                      "the system's side answers with the mode, the theme and your commands, and keeps a clipping as a note with its quote, "
                      'and a second one of the page adds its comment and its tags to it, each tag once', output)
        before = user(OPEN + ' | grep -c "^page "')[1]
        # the host looks for what was asked once a second, and the tab takes a moment to be there
        status, output = user('kavat-browser command "new tab"; for n in 1 2 3 4 5 6 7 8 9 10; do sleep 1.5; [ "$(' + OPEN + ' | grep -c "^page ")" != "' + before + '" ] && break; done; '
                              + OPEN + ' | grep -c "^page "; ' + OPEN + f' | grep -c "^page chrome-extension://{KAVAT}/start.html\\|^page chrome://newtab"')
        lines = output.split()
        checks.report(len(lines) == 2 and lines[0].isdigit() and before.isdigit() and int(lines[0]) == int(before) + 1 and lines[1] != '0',
                      'a command asked for from outside reaches the open browser, and a new tab is the start page of the system',
                      # when it fails: whether the command is still waiting, and which hosts there are to take it
                      f'{before} before\n{output}\n' + user('ls -la $XDG_RUNTIME_DIR/kavat-browser/; pgrep -a -f "[b]rowser-host"; date +%T')[1])
        time.sleep(2)
        machine.screen.picture(f'{pictures}/browser-tab.png')
        status, output = user('sleep 2; kavat-browser tabs | grep -c "^[0-9][0-9]*\t"; ls $XDG_RUNTIME_DIR/kavat-browser/tabs | grep -c "json$"; '
                              'kavat-browser tab 999999 2>&1 | grep -c "no such tab"; kavat-shell msg plugins list | grep -c "kavat/palette.*enabled"; '
                              'grep -c "^prefix *= *.t.$" /usr/share/kavat/shell-plugins/palette/plugin.toml')
        lines = output.split()
        checks.report(len(lines) == 5 and lines[0].isdigit() and int(lines[0]) >= 2 and lines[1:] == ['1', '1', '1', '1'],
                      "the browser tells the system its tabs, which the launcher lists behind /t, and a tab that is not there is said", output)
        # Kavat Chromium hides the parts of its windows when the system asks: the top of the
        # window is another picture in zen mode, and the window stays a window
        top = ('hyprctl --instance 0 -j clients | python3 -c "import json, sys; w = [c for c in json.load(sys.stdin) if c[\\"class\\"] == \\"kavat-browser\\"][0]; '
               'print(f\\"{w[\\"at\\"][0]},{w[\\"at\\"][1]} {w[\\"size\\"][0]}x120\\")" | { read g; grim -g "$g" -t ppm - | md5sum | cut -c1-8; }')
        status, output = user(f'pacman -Qq kavat-chromium 2> /dev/null; {top}; kavat-browser parts zen > /dev/null; sleep 3; {top}; cat $XDG_RUNTIME_DIR/kavat-browser/parts; '
                              f'{WINDOW} | grep -c "fullscreen: 0"; kavat-browser parts none > /dev/null; sleep 3; kavat-browser parts', 60)
        lines = output.split()
        checks.report(len(lines) == 6 and lines[0] == 'kavat-chromium' and lines[1] != lines[2] and lines[3:] == ['zen', '1', 'none'],
                      "the browser is Kavat Chromium, and the system hides the parts of its window, zen mode, and shows them again", output)

        print('albums, its colours, a profile for a mode, the sandbox', flush=True)
        status, output = user('ffmpeg -v error -y -f lavfi -i color=c=red:s=800x500 -frames:v 1 /tmp/album.png; cat > /tmp/albums.py << "END"\n' + ALBUMS + 'END\npython3 /tmp/albums.py; '
                              'grep -c "^start_album = \\"reading\\"" ~/.config/kavat/browser/browser.toml', 90)
        lines = output.split('\n')
        checks.report(len(lines) == 9 and lines[0] == 'Added to reading' and "'count': '1'" in lines[1] and lines[2] == '1 A site, again https://example.org/site'
                      and lines[3] == 'data:image/jpeg;base64,' and lines[4] == 'there is no such picture' and "'start_album': 'reading'" in lines[5]
                      and lines[6] == "['album', 'clip']" and lines[7] == '0' and lines[8] == '1',
                      'a site goes into an album once, with its picture; the start page takes an album; the notes about a site are found', output)
        status, output = user('kavat-browser background wallpaper; grep -c "^start_background = \\"wallpaper\\"" ~/.config/kavat/browser/browser.toml; '
                              'cat > /tmp/background.py << "END"\n' + BACKGROUND + 'END\npython3 /tmp/background.py; '
                              'kavat-browser background /tmp/nothing.png 2>&1 | grep -c "there is no file"; kavat-browser background theme', 90)
        lines = output.split('\n')
        checks.report(len(lines) == 7 and lines[:2] == ['wallpaper', '1'] and lines[2] == 'data:image/jpeg;base64,' and lines[3] == 'kept'
                      and lines[4] == 'own data:image/jpeg;base64,' and lines[5:] == ['1', 'theme'],
                      "a new tab's background is the desktop's, made a picture the page can show, or a picture of your own, or the theme's colour again", output)
        # the browser's own bars are the palette's: the frame behind the tabs is the theme's background, and follows a change
        frame = POINT.format(x=1500, y=70)
        status, output = user(f'kavat-theme tokens | grep "^bg " | cut -c5-; {frame}; echo; kavat-theme set palette nord accent default > /dev/null; sleep 4; '
                              f'kavat-theme tokens | grep "^bg " | cut -c5-; {frame}; echo; kavat-theme set palette kavat accent default > /dev/null', 60)
        lines = output.split()
        checks.report(len(lines) == 4 and lines[0] == lines[1] and lines[2] == lines[3] and lines[0] != lines[2],
                      "the browser's own bars wear the palette, and change with it while the browser is open", output)
        status, output = user('sed -i "s/^# gamer = /gamer = /" ~/.config/kavat/browser/browser.toml; kavat-mode gamer > /dev/null; '
                              'setsid -f kavat-browser > /dev/null 2>&1; sleep 8; kavat-mode coder > /dev/null; '
                              'grep -c "\\"system_theme\\":1" ~/.config/kavat-browser/Gaming/Preferences; ls ~/.config/kavat-browser/Gaming | grep -c -x History', 60)
        checks.report(output.split() == ['1', '1'], 'a window opened in a mode that has a profile of its own is that profile\'s', output)
        status, output = user('pkill -x chromium; sleep 3; mkdir -p ~/private && echo kept > ~/private/key; kavat-sandbox on browser; '
                              'kavat-sandbox run browser -- sh -c "ls ~/private 2>&1 | grep -c \\"No such\\"; ls -d ~/Notes ~/.config/kavat-browser | wc -l; '
                              'gdbus call --session -d org.freedesktop.systemd1 -o /org/freedesktop/systemd1 -m org.freedesktop.systemd1.Manager.ListUnits 2>&1 | grep -c -i \\"error\\""; '
                              'setsid -f kavat-browser --remote-debugging-port=9222 --remote-allow-origins=* > /dev/null 2>&1; sleep 10; '
                              f'pgrep -c -x bwrap; {OPEN} | grep -c "service_worker chrome-extension://{KAVAT}/"; '
                              'kavat-browser http://127.0.0.1:9/second > /dev/null 2>&1; sleep 4; ' + OPEN + ' | grep -c "127.0.0.1:9/second"; '
                              'kavat-sandbox off browser; pkill -x chromium; sleep 3; pgrep -c -x bwrap; '
                              'setsid -f kavat-browser --remote-debugging-port=9222 --remote-allow-origins=* > /dev/null 2>&1; sleep 8; rm -r ~/private', 120)
        lines = output.split()
        checks.report(len(lines) == 7 and lines[:3] == ['1', '2', '1'] and lines[3] != '0' and lines[4] != '0' and lines[5] == '1' and lines[6] == '0',
                      'in its sandbox the browser sees its own folders and the notes and nothing else of the home folder, cannot speak to the session\'s manager, and a second window joins the first', output)

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
