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
                              'kavat-shell msg config-get bar.default.end | grep -c -E "privacy|keyboard_layout"; tesseract --list-langs 2> /dev/null | grep -c -x -E "eng|spa"; '
                              'cat ~/.local/state/kavat/theme/current/lights/colour; grep -c "screen-text" /usr/share/kavat/hypr/hyprland.lua')
        checks.report(output.split() == ['coder', 'dark', '1', '2', '2', 'f2a93b', '1'],
                      "kavat is one command for the system's own, the launcher has the system's things, the bar marks who listens and which keyboard, "
                      "the screen's text can be read in two languages, and the lights have their colour", output)

        print('placing a display, encrypted names, a game, the shell, the theme by itself', flush=True)
        status, output = user('kavat-displays place Virtual-1 300 200 > /dev/null; sleep 1; kavat-displays | cut -f4; kavat-displays keep; '
                              'kavat-shell msg plugins list | grep -c "kavat/settings.*enabled"; grep -c "ui.arrange" /usr/share/kavat/shell-plugins/settings/panel.luau')
        checks.report(output.split() == ['0x0', '1', '1'], 'a display that is dropped anywhere ends where the arrangement starts when it is the only one', output)
        status, output = user('kavat-displays workspaces Virtual-1 1,2,3 > /dev/null; kavat-displays | cut -f7; grep -c "workspaces = \\"1 2 3\\"" ~/.config/hypr/monitors.lua; '
                              'hyprctl --instance 0 workspacerules | grep -c "^Workspace rule"; kavat-displays workspaces Virtual-1 none; kavat-displays | cut -f7; '
                              'kavat-displays workspaces Virtual-1 11 2>&1 | grep -c "from 1 to 10"')
        checks.report(output.split() == ['1,2,3', '1', '3', '-', '1'], 'a display is given workspaces of its own, which Hyprland holds as rules, and gives them up', output)
        status, output = user('kavat-shell msg config-get bar.default.start | grep -c wallpaper; kavat-shell msg config-get bar.default.end | grep -c -E "control-center"; '
                              'kavat-shell msg config-get bar.default.end | grep -c keyboard_layout; kavat-shell msg config-get wallpaper.fill_color; '
                              'kavat-shell msg wallpaper-get | grep -c "kavat/theme/"; '
                              'kavat-sound | cut -f1 | sort -u | tr "\\n" " "; echo; kavat-sound volume output 40; kavat-sound | grep ^output | cut -f5; kavat-sound volume output 100; '
                              'kavat-sound volume output loud 2>&1 | grep -c usage')
        lines = output.split('\n')
        checks.report(len(lines) == 8 and lines[:3] == ['0', '0', '1'] and 'surface' in lines[3] and lines[4] == '1' and 'output' in lines[5] and lines[6:] == ['40', '1'],
                      "the bar has neither the shell's background picker nor its control center, the background is the theme's, and the Sound page's program sets a volume", output)
        status, output = machine.run('printf "[Interface]\\nPrivateKey = 6NJepbdEduV97+1+dCqYLcZ3XGGXGPdkQjxgLqvdZHI=\\nAddress = 10.2.0.2/32\\n\\n[Peer]\\n'
                                     'PublicKey = GuR0sfhk5vS6mQUcXz4qR7VGzcE7H0jmAbFPMZ8bJXM=\\nAllowedIPs = 0.0.0.0/0\\nEndpoint = 192.0.2.10:51820\\n" > /tmp/kavat-test-vpn-file.conf; '
                                     'kavat-network vpn add /tmp/kavat-test-vpn-file.conf > /dev/null; kavat-network | grep ^vpn | cut -f2,3,4,5; '
                                     'nmcli -g connection.autoconnect connection show kavat-test-vpn-file; '
                                     'kavat-network vpn add /tmp/nothing.conf 2>&1 | grep -c "there is no file"; kavat-network vpn remove kavat-test-vpn-file; kavat-network | grep -c ^vpn; '
                                     'kavat-network wifi startup off; kavat-network | grep ^wifi | cut -f3; kavat-network wifi startup on; kavat-network | grep ^wifi | cut -f3; '
                                     'systemctl is-enabled bluetooth; which bluetoothctl', 90)
        checks.report(output.split('\n') == ['kavat-test-vpn-file\twireguard\t-\tincognito', 'no', '1', '0', 'off', 'on', 'enabled', '/usr/bin/bluetoothctl'],
                      "a VPN is added from its provider's file, under its own name, for Incognito, and removed; what is on at startup is kept; Bluetooth is installed", output)
        status, output = machine.run('kavat-boot-settings set timeout=3 recovery=no > /dev/null; kavat-boot-settings | tr "\\t" "=" | grep -E "timeout|recovery" | tr "\\n" " "; echo; '
                                     'grep -c "menuentry .Recovery" /boot/grub/grub.cfg; grep -c "set timeout=3" /boot/grub/grub.cfg | sed "s/[1-9][0-9]*/some/"; '
                                     'kavat-boot-settings set timeout=5 recovery=yes > /dev/null; grep -c "menuentry .Recovery" /boot/grub/grub.cfg; '
                                     'kavat-boot-settings set timeout=7 2>&1 | grep -c "not a setting"', 240)
        checks.report(output.split('\n') == ['timeout=3 recovery=no ', '0', 'some', '1', '1'],
                      'the boot menu waits as long as it is set to and leaves out an entry that is turned off, and gets it back', output)
        print('the second round: night light, a background each, the login, what starts, Qt', flush=True)
        status, output = user('kavat-displays night warmth 3500; kavat-displays night on; sleep 2; hyprctl --instance 0 hyprsunset temperature; kavat-displays night | cut -f1,2; '
                              'kavat-displays night auto; grep -c "^profile" ~/.config/hypr/hyprsunset.conf; kavat-displays night off; systemctl --user is-active hyprsunset; '
                              'kavat-displays night warmth 900 2>&1 | grep -c usage')
        checks.report(output.split('\n') == ['3500', 'on\t3500', '2', 'inactive', '1'],
                      'the night light warms the colours when on, is two profiles by the hour, and stops when off', output)
        status, output = user('mkdir -p ~/Pictures/Wallpapers/images; cd ~/Pictures/Wallpapers/images; for colour in navy teal; do '
                              'ffmpeg -loglevel error -y -f lavfi -i color=c=$colour:s=320x180 -frames:v 1 0-$colour.png; done; '
                              'hyprctl --instance 0 output create headless > /dev/null; sleep 2; second=$(kavat-wallpaper displays | grep HEADLESS | cut -f1 | head -n 1); '
                              'kavat-wallpaper set $PWD/0-teal.png $second; kavat-wallpaper displays | grep -c "HEADLESS.*0-teal.png"; kavat-shell msg wallpaper-get $second | grep -c 0-teal; '
                              'kavat-shell msg wallpaper-get Virtual-1 | grep -c "kavat/theme/"; kavat-wallpaper same $second; kavat-wallpaper displays | grep -c "0-teal"; '
                              'hyprctl --instance 0 output remove $second > /dev/null; '
                              'kavat-wallpaper option slideshow 5; systemctl --user list-timers --no-legend | grep -c kavat-wallpaper-slideshow; '
                              'kavat-wallpaper set $PWD/0-navy.png; kavat-wallpaper next; kavat-wallpaper | grep -c 0-teal.png; '
                              'kavat-wallpaper option slideshow off; systemctl --user list-timers --no-legend | grep -c kavat-wallpaper-slideshow; kavat-wallpaper theme; rm 0-navy.png 0-teal.png', 60)
        checks.report(output.split() == ['1', '1', '1', '0', '1', '1', '0'],
                      'a display takes an image of its own and gives it up, and the slideshow shows the next image of the library by a timer', output)
        status, output = machine.run('printf "client\\ndev tun\\nproto udp\\nremote 192.0.2.20 1194\\nauth-user-pass\\n<ca>\\n-----BEGIN CERTIFICATE-----\\nMIIB\\n-----END CERTIFICATE-----\\n</ca>\\n" '
                                     '> /tmp/kavat-test-corp.ovpn; kavat-network vpn add /tmp/kavat-test-corp.ovpn > /dev/null; kavat-network | grep ^vpn | cut -f2,3,6; '
                                     'kavat-network vpn up kavat-test-corp 2>&1 | grep -c "asks for a name and a password"; kavat-network vpn login kavat-test-corp will "pa,ss word" > /dev/null; '
                                     'kavat-network | grep ^vpn | cut -f6; nmcli -g vpn.data connection show kavat-test-corp | grep -c "username = will"; '
                                     'kavat-network vpn remove kavat-test-corp; kavat-network bluetooth pair 00:11:22:33:44:55 2>&1 | grep -c "no Bluetooth adapter"', 90)
        checks.report(output.split('\n') == ['kavat-test-corp\topenvpn\tlogin', '1', '-', '1', '1'],
                      'a VPN that asks for a name and a password says so, does not come up without them, and keeps them once given', output)
        status, output = machine.run('kavat-boot-settings set default=other > /dev/null; grep -c "set default=.kavat-other-systems>0" /boot/grub/grub.cfg; '
                                     'kavat-boot-settings set default=kavat follow=yes > /dev/null; grep -c "kavat-other-systems>0" /boot/grub/grub.cfg; '
                                     'runuser -l tester -c "kavat-theme set accent teal"; /usr/lib/kavat/boot-follow > /dev/null 2>&1; grep -c "accent = .teal" /var/lib/kavat/theme/settings.toml; '
                                     'systemctl is-active kavat-boot-follow.service; runuser -l tester -c "kavat-theme set accent amber"; /usr/lib/kavat/boot-follow > /dev/null 2>&1; '
                                     'grep -c "accent = .amber" /var/lib/kavat/theme/settings.toml; kavat-boot-settings set follow=no > /dev/null', 400)
        checks.report(output.split() == ['1', '0', '1', 'active', '1'],
                      "the other system can be what starts by itself, and the boot follows the desktop's theme when asked to, as the machine is turned off", output)
        status, output = machine.run('kavat-login-settings set clock=12 users=no auto=tester > /dev/null; kavat-login-settings | tr "\\t\\n" "= "; echo; '
                                     'grep -c -E "^\\[initial_session\\]|^user = .tester" /etc/kavat/greetd.toml; grep -c greetd-start /usr/lib/systemd/system/greetd.service.d/kavat-greeter.conf; '
                                     'cat /home/tester/.local/share/keyrings/default; echo; stat -c %U:%a /home/tester/.local/share/keyrings/Kavat.keyring; '
                                     'kavat-login-settings set auto=nobody-at-all 2>&1 | grep -c "not a setting"; kavat-login-settings set clock=24 users=yes auto=none > /dev/null; '
                                     'ls /etc/kavat/greetd.toml 2> /dev/null | wc -l; cat /home/tester/.local/share/keyrings/default; echo')
        checks.report(output.split('\n') == ['clock=12 users=no auto=tester ', '2', '1', 'Kavat', 'tester:600', '1', '0', 'login'],
                      "the login's clock, whether it shows who can log in, and who logs in by itself are kept, greetd is told who, and that person's keyring asks for no password", output)
        status, output = user('readlink ~/.config/qt6ct/colors/kavat.conf | grep -c "kavat/theme/current/qt/kavat.conf"; grep -c "^color_scheme_path=$HOME/.config/qt6ct/colors/kavat.conf" ~/.config/qt6ct/qt6ct.conf; '
                              'grep -c "^active_colors=#ff[0-9a-f]\\{6\\}, " ~/.config/qt6ct/colors/kavat.conf; tr "\\0" "\\n" < /proc/$(pgrep -u tester -x kavat-shell | head -n 1)/environ | grep -c "^QT_QPA_PLATFORMTHEME=qt6ct"; '
                              'nmcli -t -f TYPE connection show --active | grep -c -E "^(vpn|wireguard)"; grep -c "VPN · " /usr/share/kavat/shell-plugins/modes/widget.luau; '
                              'kavat-shell msg config-get bar.default.end | grep -c "kavat/settings:network"; kavat-network | grep -c "^proton.-.-$"; kavat-mode vpn proton 2>&1 | grep -c "is not installed"')
        checks.report(output.split() == ['1', '1', '1', '1', '0', '1', '1', '1', '1'], "Qt apps are handed the theme's colours through qt6ct, which the session names; the bar has the system's network widget; Proton VPN is offered and said not to be installed", output)
        print('the third round: shortcuts, the shell, web apps, Shorts, wallpapers online, remote access', flush=True)
        status, output = user('kavat-keys set Super+E Super+Shift+E; kavat-keys | grep -c "^Super+Shift+E.Files.Apps.Super+E$"; kavat-keys set Super+U Super+B 2>&1 | grep -c "is already"; '
                              'kavat-keys off Super+U; hyprctl --instance 0 binds | grep -c "^bind"; kavat-keys reset all; hyprctl --instance 0 binds | grep -c "^bind"; '
                              'hyprctl --instance 0 configerrors; kavat-keys set Super+Arrows Super+K 2>&1 | grep -c "several shortcuts"')
        lines = output.split()
        checks.report(len(lines) == 5 and lines[:2] == ['1', '1'] and lines[2].isdigit() and lines[3].isdigit() and int(lines[3]) == int(lines[2]) + 1 and lines[4] == '1',
                      'a shortcut is given other keys, or none, and gets its own back; keys another has are refused', output)
        status, output = user('zsh -i -c "whence -w port-kill y" 2> /dev/null | tr "\\n" " "; echo; (python3 -m http.server 58123 > /dev/null 2>&1 &); sleep 1.5; '
                              'zsh -i -c "port-kill | grep -c 58123/tcp; port-kill 58123" 2> /dev/null | tr "\\n" " "; echo; zsh -i -c "bindkey \\"\\e\\e\\"" 2> /dev/null | grep -c sudo-command-line; '
                              'kavat-conf show shell | grep -c -E "more_completions|esc_sudo"; pacman -Q zsh-completions > /dev/null 2>&1 && echo there; '
                              'kavat-fetch --pipe 2> /dev/null | grep -c -E "Health +(✓ ok|!)"', 60)
        lines = output.split('\n')
        checks.report(len(lines) == 6 and 'port-kill: function' in lines[0] and 'y: function' in lines[0] and lines[1].startswith('1 ') and 'stopped python' in lines[1]
                      and lines[2:] == ['1', '2', 'there', '1'],
                      "port-kill and y come written for the user, Esc twice puts sudo, the extra completions are there, and kavat-fetch says the system's health", output)
        status, output = user('kavat-webapp install Test https://example.invalid/ < /dev/null > /dev/null 2>&1; kavat-webapp list | tr "\\t" "|"; '
                              'grep -c -x "Exec=kavat-browser --app=\\"https://example.invalid/\\"" ~/.local/share/applications/kavat-webapp-test.desktop; '
                              'kavat-webapp install Bad "javascript:alert(1)" > /dev/null 2>&1; echo $?; kavat-webapp remove test > /dev/null; kavat-webapp list | wc -l; '
                              'kavat-browser shorts; kavat-browser shorts on; kavat-browser shorts off; test -s /usr/share/kavat-browser/extensions/kavat/shorts-rules.js && echo rules; '
                              'kavat-wallpaper sources | cut -f1,2 | tr "\\t" "|"; kavat-wallpaper search sea 2>&1 | grep -c "no network"; kavat-wallpaper option | grep -c "^lock background"; '
                              'readlink ~/.cache/kavat/lock/background.png | grep -c "kavat/lock/kavat-"', 60)
        checks.report(output.split('\n') == ['Test|https://example.invalid/|kavat-browser', '1', '2', '0', 'off', 'on', 'off', 'rules', 'motionbgs|on', 'wallhaven|on', '1', '1', '1'],
                      'a page is installed as an app and removed, only from a web address; Shorts are hidden unless asked for; the online search of wallpapers has its two sources, of images and of live ones, and says there is no network; the lock is given its picture', output)
        # remote access comes in when Sunshine, which it needs, is built: until then there is nothing to check
        if machine.run('test -x /usr/bin/kavat-remote')[0] == 0:
            status, output = machine.run('systemctl is-enabled tailscaled sshd kavat-remote-awake 2>&1 | sort -u | tr "\\n" " "; echo; ss -Hltn | grep -c -E ":(22|47984|47989|47990|48010)\\b"; '
                                         'ls /etc/kavat/firewall.d/ | grep -c kavat-remote; runuser -u tester -- kavat-remote | head -n 1 | tr "\\t" "="; '
                                         'kavat-remote on > /tmp/remote-on 2>&1; echo $?; runuser -u tester -- kavat-remote | sed -n 1p | tr "\\t" "="; '
                                         'sshd -T 2> /dev/null | grep -i -E "^(passwordauthentication|permitrootlogin) " | tr "A-Z\\n" "a-z "; echo; '
                                         'grep -L "iifname \\"tailscale0\\"" /etc/kavat/firewall.d/kavat-remote-*.nft | wc -l; nft list table inet kavat_firewall | grep -c tailscale0; '
                                         'systemctl is-active tailscaled sshd kavat-remote-awake | tr "\\n" " "; echo; '
                                         'kavat-remote off > /dev/null 2>&1; ls /etc/kavat/firewall.d/ | grep -c kavat-remote; nft list table inet kavat_firewall | grep -c tailscale0; '
                                         'systemctl is-active tailscaled sshd | sort -u | tr "\\n" " "', 120)
            lines = output.split('\n')
            checks.report(len(lines) >= 11 and lines[0].strip() == 'disabled' and lines[1:4] == ['0', '0', 'state=off'] and lines[4:6] == ['0', 'state=on'] and 'passwordauthentication no' in lines[6]
                          and 'permitrootlogin no' in lines[6] and lines[7] == '0' and lines[8].isdigit() and int(lines[8]) >= 3
                          and lines[9].split() == ['active', 'active', 'active'] and lines[10:12] == ['0', '0'] and lines[12].strip() == 'inactive',
                          'remote access is off as the system comes, with nothing listening; on, its doors are on the private network only and the terminal takes keys alone; off undoes it', output + '\n' + machine.run('cat /tmp/remote-on')[1])
        status, output = machine.run('kavat-dns; kavat-dns on > /dev/null; kavat-dns; readlink /etc/resolv.conf; resolvectl status | grep -c "+DNSOverTLS"; '
                                     'kavat-dns on nobody 2>&1 | grep -c "is not one of"; kavat-dns off > /dev/null; kavat-dns; systemctl is-active systemd-resolved; '
                                     'test -L /etc/resolv.conf || echo a-file', 90)
        checks.report(output.split('\n') == ['off', 'on\tquad9', '/run/systemd/resolve/stub-resolv.conf', '1', '1', 'off', 'inactive', 'a-file'],
                      'encrypted DNS is off as the system comes, asks one server over TLS when on, and goes back whole', output)
        status, output = user('kavat game new elden 1245620 > /dev/null; kavat-game | cut -f1,2; '
                              'SteamAppId=1245620 kavat-game show /bin/true | grep -c -E "\\"profile\\": \\"elden\\"|MangoHud is not installed"; kavat-game run echo ran 2> /dev/null; '
                              'kavat-game delete elden; kavat-game | wc -l')
        checks.report(output.split('\n') == ['elden\t1245620', '2', 'ran', '0'],
                      'a game has a profile of its own, found by its number in Steam, and starts all the same when what the profile asks for is not installed', output)
        status, output = user('kavat-conf set shell prompt full; kavat-conf set shell listing ls; kavat-conf set shell history_lines 10000; '
                              'zsh -i -c "echo \$HISTSIZE \$STARSHIP_CONFIG; alias ls" 2> /dev/null; echo "alias hey=\\"echo there\\"" > ~/.config/kavat/aliases.zsh; '
                              'kavat-conf set shell prompt short; kavat-conf set shell listing lsd; kavat-conf set shell history_lines 50000; '
                              'zsh -i -c "echo \$HISTSIZE \$STARSHIP_CONFIG; hey" 2> /dev/null; xdg-mime query default image/png; xdg-mime query default application/pdf')
        checks.report(output.split('\n') == ['10000 /usr/share/kavat/starship-full.toml', "ls='ls --color=auto'", '50000 /usr/share/kavat/starship.toml', 'there',
                                              'imv.desktop', 'org.pwmt.zathura.desktop'],
                      "the Shell page's settings are worn by the next terminal, your aliases are read, and an image and a PDF open in their viewers", output)
        status, output = user('kavat-theme profile save plain > /dev/null; kavat-theme set palette nord accent default > /dev/null; kavat-theme profile save cold > /dev/null; '
                              'kavat-theme profile plain > /dev/null; printf "[[rule]]\nprofile = \\"cold\\"\ndisplay = \\"qemu\\"\n\n[[rule]]\nprofile = \\"plain\\"\n" > ~/.config/kavat/theme-rules.toml; '
                              '/usr/lib/kavat/theme-rules show | cut -f1; /usr/lib/kavat/theme-rules; kavat-theme get palette; '
                              'kavat-theme set palette gruvbox accent default > /dev/null; /usr/lib/kavat/theme-rules; kavat-theme get palette; '
                              'rm ~/.config/kavat/theme-rules.toml; kavat-theme profile plain > /dev/null; systemctl --user is-active kavat-theme-rules.timer; '
                              'kavat-theme set terminal.palette nord > /dev/null; grep -c -i "242933" ~/.local/state/kavat/theme/current/kitty/theme.conf; '
                              'grep -c -i "f2a93b" ~/.local/state/kavat/theme/current/gtk/gtk-4.0.css; kavat-theme set terminal.palette system > /dev/null; '
                              'kavat-theme set auto.sun true auto.latitude 40.4 auto.longitude -3.7 auto.follow true > /dev/null; /usr/lib/kavat/theme-auto; '
                              'systemctl --user is-active kavat-theme-auto.timer; kavat-theme set auto.follow false auto.sun false mode dark > /dev/null; /usr/lib/kavat/theme-auto', 90)
        lines = output.split('\n')
        checks.report(len(lines) == 8 and lines[0] == '* cold' and lines[1].strip() == 'plain' and lines[2] == 'nord' and lines[3] == 'gruvbox'
                      and lines[4] == 'active' and lines[5] != '0' and lines[6] != '0' and lines[7] == 'active',
                      'a rule wears a profile when it starts to hold and leaves a change made by hand alone; the terminal can wear a palette of its own; the mode can follow the sun', output)

        for page in ('wallpaper', 'displays', 'projects', 'privacy', 'gpu', 'shell'):
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
