#!/usr/bin/env python3
"""Check the desktop's background, the displays, the disks that are plugged in and the firewall.

usage: tests/desktop.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. The background starts as the theme's picture. An image put
in the library has to become the background, and the theme has to take a palette from it.
A video, made for the test, has to play under mpvpaper over a still frame of it, stop in
Gamer mode, and play again after a new login. A change of the display's mode has to undo
itself when it is not kept, and stay when it is, in a file Hyprland reads. The watch for
disks has to be running, and the firewall has to be up with everything incoming closed.

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
GAMER, CODER = (279, 17), (226, 17)  # the modes on the bar, on a 1920x1080 screen
IMAGES, LIVE = f'{HOME}/Pictures/Wallpapers/images', f'{HOME}/Pictures/Wallpapers/live'
PLAYING = 'kavat-wallpaper playing'


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('desktop', encrypted=len(sys.argv) > 1)
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

        print('the background', flush=True)
        status, output = user(f'kavat-wallpaper | cut -d" " -f1; ls -d {IMAGES} {LIVE} | wc -l; '
                              # a picture with one strong colour, and a short video
                              f'ffmpeg -loglevel error -f lavfi -i color=c=0x2060c0:s=640x360 -frames:v 1 {IMAGES}/sea.png && '
                              f'ffmpeg -loglevel error -f lavfi -i testsrc=duration=4:size=640x360:rate=24 -pix_fmt yuv420p {LIVE}/bars.mp4 && '
                              'kavat-wallpaper list | cut -f1 | tr "\\n" " "', 90)
        checks.report(output.split() == ['theme', '2', 'theme', 'image', 'video'],
                      "the background starts as the theme's picture, and the library lists what is put in its two folders", output)
        status, output = user(f'kavat-wallpaper set {IMAGES}/sea.png && kavat-wallpaper && kavat-shell msg wallpaper-get')
        checks.report(output.split('\n') == [f'image {IMAGES}/sea.png', f'{IMAGES}/sea.png'], 'an image of the library becomes the background', output)
        status, output = user('kavat-wallpaper palette && kavat-theme get palette && kavat-theme get accent && '
                              'grep -c "^name = \\"Wallpaper\\"" ~/.config/kavat/palettes/wallpaper.toml')
        checks.report(output.split() == ['wallpaper', 'blue', '1'], 'the theme takes a palette from it, with the colour of the picture as its accent', output)
        time.sleep(2)
        machine.screen.picture(f'{pictures}/desktop-palette.png')

        status, output = user(f'kavat-wallpaper set {LIVE}/bars.mp4 && kavat-wallpaper | cut -d" " -f1 && '
                              'kavat-shell msg wallpaper-get | grep -c "/.cache/kavat/wallpaper/.*png"', 60)
        checks.report(output.split() == ['video', '1'], 'a video becomes the background, over a still frame of it', output)
        until('mpvpaper plays it', 'pgrep -c -x mpvpaper; ' + PLAYING, 'playing', 15)
        time.sleep(2)
        machine.screen.picture(f'{pictures}/desktop-video.png')
        machine.screen.click(*GAMER)
        until('Gamer mode stops the video where it is', 'kavat-mode; ' + PLAYING, 'gamer\npaused', 15)
        machine.screen.click(*CODER)
        until('and Coder lets it go on', 'kavat-mode; ' + PLAYING, 'coder\nplaying', 15)
        status, output = user('kavat-wallpaper option fit stretch && kavat-wallpaper option | tr "\\n" " "; kavat-wallpaper option fit round 2>&1 | head -n 1')
        checks.report(output.startswith('sound off fit stretch gpu on') and 'the options are' in output, "the video's settings are kept, and a value that is not one is refused", output)
        status, output = user('pkill -x mpvpaper; sleep 1; kavat-wallpaper restore; sleep 2; pgrep -c -x mpvpaper')
        checks.report(output == '1', 'the video is put back as a new session would', output)
        status, output = user('kavat-wallpaper theme && kavat-theme set palette kavat accent amber && kavat-wallpaper | cut -d" " -f1 && '
                              'pgrep -c -x mpvpaper; kavat-shell msg wallpaper-get | grep -c "/kavat/theme/current/wallpaper/"')
        checks.report(output.split() == ['theme', '0', '1'], "going back to the theme's picture stops the video", output)

        print('the displays', flush=True)
        status, output = user('kavat-displays | cut -f1,3; kavat-displays modes Virtual-1 | grep -c "^1920x1080@"')
        lines = output.split('\n')
        checks.report(len(lines) == 2 and lines[0].startswith('Virtual-1\t1920x1080@') and lines[1] != '0',
                      'the display is listed with its mode, and with the modes it can show', output)
        before = lines[0].split('\t')[-1]
        status, output = user('kavat-displays set Virtual-1 mode=1280x720 > /dev/null && sleep 2 && kavat-displays | cut -f3 && '
                              'grep -c "desc:.*1280x720" ~/.config/hypr/monitors.lua')
        checks.report(output.startswith('1280x720@') and output.endswith('\n1'), "a change of mode is worn at once, from a file Hyprland reads", output)
        until('and undone by itself when it is not kept', 'kavat-displays | cut -f3; ls ~/.config/hypr/monitors.lua 2>&1 | grep -c "No such"', f'{before}\n1', 20)
        status, output = user('kavat-displays set Virtual-1 mode=1600x900 scale=1.25 > /dev/null && kavat-displays keep && sleep 12 && kavat-displays | cut -f3,5')
        checks.report(output.startswith('1600x900@') and output.endswith('\t1.25'), 'a change that is kept stays', output)
        status, output = user('kavat-displays set Virtual-1 at=right-of:Nothing 2>&1 | head -n 1; '
                              f'kavat-displays set Virtual-1 mode={before} scale=1 > /dev/null && kavat-displays keep && sleep 1 && kavat-displays | cut -f3,5; '
                              'hyprctl --instance 0 configerrors')
        checks.report('is not something a display can be set to' in output and output.endswith(f'{before}\t1'),
                      'a place that makes no sense is refused, and the display goes back to what it was', output)

        print('disks and the firewall', flush=True)
        status, output = user('pgrep -c -f "[u]diskie --config /usr/share/kavat/udiskie.yml"; grep -c "kitty yazi" /usr/lib/kavat/open-folder')
        checks.report(output.split() == ['1', '1'], 'the watch for disks that are plugged in is running, and opens them in Yazi', output)
        status, output = machine.run('systemctl is-active kavat-firewall; nft list table inet kavat_firewall | grep -c -E "policy drop|established,related accept"; '
                                     'mkdir -p /etc/kavat/firewall.d && echo "add rule inet kavat_firewall in tcp dport 22 accept" > /etc/kavat/firewall.d/ssh.nft && '
                                     'systemctl restart kavat-firewall && nft list table inet kavat_firewall | grep -c "tcp dport 22 accept"')
        checks.report(output.split() == ['active', '2', '1'], 'the firewall is up with everything incoming closed, and a door of your own can be opened', output)

        print('keys', flush=True)
        status, output = user('systemctl --user is-active ssh-agent.service; '
                              'tr "\\0" "\\n" < /proc/$(pgrep -x kavat-shell | head -n 1)/environ | grep -c "^SSH_AUTH_SOCK=$XDG_RUNTIME_DIR/ssh-agent.socket$"; '
                              'SSH_AUTH_SOCK=$XDG_RUNTIME_DIR/ssh-agent.socket ssh-add -l 2>&1 | grep -c "no identities"; '
                              'grep -c pam_gnome_keyring /etc/pam.d/greetd; pgrep -c -x gnome-keyring-d')
        checks.report(output.split() == ['active', '1', '1', '2', '1'],
                      "an agent of the session holds the SSH keys and every app is pointed at it, and the keyring opened with the login", output)
        status, output = user('pw-cli ls Node 2> /dev/null | grep -c "kavat_noise_source"; kavat-conf show sound; kavat-conf set sound noise_suppression off && sleep 2 && '
                              'pw-cli ls Node 2> /dev/null | grep -c "kavat_noise_source"; kavat-conf set sound noise_suppression on')
        checks.report(output.split('\n')[:3] == ['1', 'noise_suppression on', '0'], "the microphone without its noise is there for a new user, and the Sound page's setting takes it away", output)

        print('projects, the one command and the palette', flush=True)
        status, output = user('mkdir -p ~/code/thing && cd ~/code/thing && kavat project new thing > /dev/null && kavat-project | cut -f1,2 && '
                              'kavat project open thing && sleep 4 && hyprctl --instance 0 activeworkspace | head -n 1 | cut -d" " -f2 && '
                              'hyprctl --instance 0 clients | grep -c "class: kitty$"; kavat project open thing; kavat-project | cut -c1; kavat nothing 2>&1 | head -n 1')
        lines = output.split('\n')
        checks.report(len(lines) == 7 and lines[0].strip() == f'thing\t{HOME}/code/thing' and lines[1].startswith('thing: workspace ') and lines[2] == lines[1].split()[-1]
                      and lines[3] == '2' and lines[4] == lines[1] and lines[5] == '*' and 'there is no nothing' in lines[6],
                      'a project opens its terminal and its editor on a workspace of its own, and opening it again only goes there', output)
        status, output = user('hyprctl --instance 0 dispatch "hl.dsp.focus({ workspace = 1 })" > /dev/null; kavat mode; kavat theme get mode; kavat-shell msg plugins list | grep -c "kavat/palette.*enabled"; '
                              'kavat-shell msg config-get bar.default.end | grep -c -E "privacy|keyboard-layout"; tesseract --list-langs 2> /dev/null | grep -c -x -E "eng|spa"; '
                              'cat ~/.local/state/kavat/theme/current/lights/colour; grep -c "screen-text" /usr/share/kavat/hypr/hyprland.lua')
        checks.report(output.split() == ['coder', 'dark', '1', '2', '2', 'f2a93b', '1'],
                      "kavat is one command for the system's own, the launcher has the system's things, the bar marks who listens and which keyboard, "
                      "the screen's text can be read in two languages, and the lights have their colour", output)

        for page in ('wallpaper', 'displays', 'projects', 'privacy', 'gpu'):
            user(f'kavat-shell msg panel-open kavat/settings:panel {page} > /dev/null')
            time.sleep(3)
            machine.screen.picture(f'{pictures}/desktop-{page}.png')
        status, output = user('kavat-shell msg panel-close kavat/settings:panel > /dev/null; hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/desktop-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
