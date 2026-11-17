#!/usr/bin/env python3
"""Check the system's modes: Coder, Gamer and Incognito, and their switch on the bar.

usage: tests/modes.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. The session has to start in Coder, with the three modes on
the bar. Pressing Gamer there has to stop what the mode names and silence the
notifications, and going back to Coder has to undo both. A process that looks like a game
started by Steam has to bring Gamer by itself. Incognito has to refuse to start without a
VPN; with one, a WireGuard connection made for the test that leads nowhere, it has to bring
it up, let nothing out but through it, and keep the terminal from writing its history; and
leaving it has to take all of that back.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes. Pictures of the screen are left in out/test/.
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
# the three modes on the bar, on a 1920x1080 screen
CODER, GAMER, INCOGNITO = (226, 17), (279, 17), (341, 17)
TIMERS = 'systemctl is-active plocate-updatedb.timer snapper-cleanup.timer | tr "\\n" " "; echo'
# A VPN that leads nowhere: it comes up, as WireGuard does before it has heard from the
# other end, which is all the mode needs to be tried
VPN = r'''
key=$(wg genkey); peer=$(wg genkey | wg pubkey)
cat > /etc/NetworkManager/system-connections/nowhere.nmconnection << END
[connection]
id=nowhere
type=wireguard
interface-name=wg-nowhere
autoconnect=false

[wireguard]
private-key=$key

[wireguard-peer.$peer]
endpoint=192.0.2.1:51820
allowed-ips=10.99.0.0/24;

[ipv4]
address1=10.99.0.2/24
method=manual

[ipv6]
method=disabled
END
chmod 600 /etc/NetworkManager/system-connections/nowhere.nmconnection
nmcli connection reload
# somewhere to try to send to that is not the VPN
ip link add outside type dummy && ip addr add 10.77.0.1/24 dev outside && ip link set outside up
'''
# what the guard dropped: it counts what it does not let out
DROPPED = 'ping -c 1 -W 1 10.77.0.2 > /dev/null 2>&1; nft list table inet kavat_incognito 2> /dev/null | grep -c "counter packets [1-9]"'
# the history function of the system's zshrc, by itself: the whole file waits for a terminal
HISTORY = ('zsh -fc \'HISTFILE=/tmp/history; source <(sed -n "/^_kavat_history()/,/^}/p" /etc/zsh/zshrc); _kavat_history; '
           'print -r -- history:${HISTFILE:-none}\'')


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('modes', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    def until(what, command, wanted, timeout=20, as_root=False):
        ask = machine.run if as_root else user
        deadline = time.time() + timeout
        _, found = ask(command)
        while wanted not in found and time.time() < deadline:
            time.sleep(0.5)
            _, found = ask(command)
        return checks.report(wanted in found, what, f'wanted {wanted}, found {found}')

    try:
        if not checks.report(machine.wait_for_agent(BOOT_TIMEOUT), 'the installed system boots'):
            sys.exit(1)
        log_in(machine, PASSWORD)
        check_desktop(machine, checks, USER)

        print('the modes', flush=True)
        status, output = user('kavat-mode; kavat-mode list | cut -f1 | tr -d "\\n"; echo; kavat-shell msg plugins list | grep -c "kavat/modes.*enabled"; '
                              'kavat-shell msg config-get bar.default.start | grep -c "kavat/modes:mode"; pgrep -f -c "[k]avat-mode watch"')
        checks.report(output.split('\n') == ['coder', '* coder  gamer  incognito', '1', '1', '1'],
                      'the session starts in Coder, with the three modes on the bar and the watch for games running', output)
        machine.screen.picture(f'{pictures}/modes-bar.png')

        print('Gamer', flush=True)
        machine.screen.click(*GAMER)
        until('pressing Gamer on the bar enters the mode', 'kavat-mode', 'gamer')
        status, output = user(f'{TIMERS}; kavat-shell msg notification-dnd-status; powerprofilesctl list | grep -c "performance:"; powerprofilesctl get')
        lines = output.split('\n')
        checks.report(lines[0].split() == ['inactive', 'inactive'] and 'on' in lines[1],
                      "the disk's index and the scheduled snapshots are stopped, and the notifications are silent", output)
        if lines[2:3] == ['1']:  # a machine that has that profile; a virtual one may not
            checks.report(lines[3:4] == ['performance'], 'the processor is on its performance profile', output)
        machine.screen.picture(f'{pictures}/modes-gamer.png')
        machine.screen.click(*CODER)
        until('pressing Coder goes back', 'kavat-mode', 'coder')
        status, output = user(f'{TIMERS}; kavat-shell msg notification-dnd-status')
        lines = output.split('\n')
        checks.report(lines[0].split() == ['active', 'active'] and 'off' in lines[1], 'and what Gamer stopped runs again', output)

        user('setsid -f sh -c "exec -a reaper python3 -c \\"import time; time.sleep(600)\\" SteamLaunch" > /dev/null 2>&1')
        until('a game started by Steam brings Gamer by itself', 'kavat-mode', 'gamer', 15)
        machine.run('pkill -x -f "reaper -c import time; time.sleep.600. SteamLaunch"; pkill -f SteamLaunch')
        until('and the end of the game brings Coder back', 'kavat-mode', 'coder', 15)

        print('Incognito', flush=True)
        machine.screen.press('super', 'shift', 'm')
        until('Super+Shift+M goes to the next mode', 'kavat-mode', 'gamer')
        machine.screen.click(*INCOGNITO)
        time.sleep(4)
        status, output = user('kavat-mode; kavat-mode incognito 2>&1 | head -n 1; sleep 1; '
                              'grep -c "opened \\"kavat/settings:panel\\"" ~/.cache/kavat-shell/kavat-shell.log; kavat-shell msg panel-close > /dev/null')
        lines = output.split('\n')
        checks.report(len(lines) == 3 and lines[0] == 'coder' and 'this machine has none yet' in lines[1] and 'kavat-network vpn add' in lines[1] and lines[2] != '0',
                      'without a VPN Incognito is not entered: it says how one is added, opens the Network page, and the machine is left in Coder', output)
        status, output = machine.run(VPN)
        status, output = user('kavat-mode vpns; kavat-mode vpn elsewhere 2>&1 | head -n 1; kavat-mode vpn nowhere; kavat-mode vpn')
        checks.report(output.split('\n')[0] == 'nowhere' and 'has no VPN called elsewhere' in output and output.endswith('nowhere'),
                      "the VPN for it is chosen from NetworkManager's", output)
        machine.screen.click(*INCOGNITO)
        until('with a VPN, pressing Incognito enters the mode', 'kavat-mode', 'incognito', 30)
        status, output = machine.run(f'nmcli -t -f NAME connection show --active | grep -c "^nowhere$"; '
                                     f'nft list table inet kavat_incognito | grep -c -E "oifname \\"wg-nowhere\\" accept|192.0.2.1|policy drop"; {DROPPED}')
        checks.report(output.split() == ['1', '3', '1'], 'the VPN is up, and nothing goes out but through it and to its server', output)
        status, output = user(HISTORY + '; (echo secret | wl-copy) > /dev/null 2>&1 < /dev/null')
        checks.report(output == 'history:none', 'the terminal writes no history while in it', output)
        status, output = user('kavat-shell msg config-get notification.hide_content; gsettings get org.gnome.desktop.privacy remember-recent-files; '
                              'kavat-remote 2> /dev/null | head -n 1 | cut -f2')
        checks.report(output.split()[:2] == ['true', 'false'], 'notifications do not show what they say, and apps keep no recent files', output)
        # the browser of the mode is a browser apart, with the blocker loaded, whose settings are in memory
        status, output = user('hyprctl --instance 0 dispatch "hl.dsp.exec_cmd(\\"kavat-browser about:blank\\")" > /dev/null; sleep 8; '
                              'pgrep -u $(id -u) -a -f "user-data-dir=$XDG_RUNTIME_DIR/kavat-browser-incognito" | grep -c -- "--load-extension=.*ublock"; '
                              'pgrep -u $(id -u) -a -x chromium | grep -c -- "--incognito"; ls -d $XDG_RUNTIME_DIR/kavat-browser-incognito | wc -l', 40)
        lines = output.split()
        checks.report(len(lines) == 3 and lines[0] != '0' and lines[1:] == ['0', '1'], 'in the mode the browser is one apart, with its blocker, and not a private window without it', output)
        machine.screen.picture(f'{pictures}/modes-incognito.png')
        machine.screen.click(*CODER)
        until('pressing Coder leaves it', 'kavat-mode', 'coder')
        status, output = user('sleep 2; pgrep -u $(id -u) -c -f "user-data-dir=$XDG_RUNTIME_DIR/kavat-browser-incognito"; ls -d $XDG_RUNTIME_DIR/kavat-browser-incognito 2> /dev/null | wc -l')
        checks.report(output.split() == ['0', '0'], 'and leaving the mode closes that browser and deletes all it kept', output)
        status, output = machine.run(f'nmcli -t -f NAME connection show --active | grep -c "^nowhere$"; nft list tables | grep -c kavat_incognito')
        checks.report(output.split() == ['0', '0'], 'the VPN comes down and the guard with it', output)
        status, output = user(HISTORY + '; timeout 3 wl-paste 2>&1 | grep -c secret')
        checks.report(output.split() == ['history:/tmp/history', '0'], 'the terminal keeps its history again, and what was copied in the mode is gone', output)

        user('kavat-shell msg panel-open kavat/settings:panel modes > /dev/null')
        time.sleep(3)
        machine.screen.picture(f'{pictures}/modes-page.png')
        status, output = user('kavat-shell msg panel-close kavat/settings:panel > /dev/null; hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/modes-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
