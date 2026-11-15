#!/usr/bin/env python3
"""Check the notes: nb as kavat-notes brings it, the quick note and the reminders.

usage: tests/notes.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. It logs in as a user who never took a note. A line of text
has to become a note in the inbox, with its tags, and a day and a time at its end have to
become a reminder: a line in the note and a timer of the session. nb has to list what is
due without asking anything first. A reminder set a minute ahead has to come up as a
notification, and one whose time passed has to be listed as overdue. Then, on the desktop,
Super+N has to open a line to type a note in, and Enter has to save it.

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
SESSION = (f'runuser -u {USER} -- env HOME={HOME} USER={USER} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
WINDOWS = 'hyprctl --instance 0 clients'
TIMERS = "systemctl --user list-timers --no-legend 'kavat-remind-*' | wc -l"
# A terminal that is not there: the notes' screen is run in it, keys are typed, and what it
# drew is searched for the words given after the keys.
SCREEN = r'''
import fcntl, os, pty, re, select, struct, sys, termios, time
keys, words = sys.argv[1].split(","), sys.argv[2].split(",")
pid, fd = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.execv("/usr/bin/kavat-notes-tui", ["kavat-notes-tui"])
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 34, 130, 0, 0))
drawn = b""
wide = 130
def whole():
    # a screen only draws what changed; a new size makes it draw everything again
    global wide
    wide = 261 - wide
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 34, wide, 0, 0))
    watch(0.8)
def watch(seconds):
    global drawn
    end = time.time() + seconds
    while time.time() < end:
        if select.select([fd], [], [], 0.1)[0]:
            try:
                drawn += os.read(fd, 65536)
            except OSError:
                return
watch(3)
whole()
for key in keys:
    if key == "WAIT":  # for something that takes a while to be done and drawn
        watch(6)
    else:
        os.write(fd, key.replace("ENTER", "\r").encode())
        watch(1.5)
    whole()
text = re.sub(rb"\x1b\[[0-9;?]*[A-Za-z]", b" ", drawn).decode("utf-8", "replace")
for _ in range(2):  # the first closes what is open over the screen, the second leaves
    try:
        os.write(fd, b"q")
    except OSError:
        break
    watch(1)
print("missing:", ",".join(word for word in words if word not in text))
print("ended:", os.waitstatus_to_exitcode(os.waitpid(pid, 0)[1]))
'''



def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('notes', encrypted=len(sys.argv) > 1)
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

        print('notes and reminders', flush=True)
        status, output = user('test ! -e ~/Notes && git config user.name; kavat-notes add "an idea about the greeter #kavat" < /dev/null')
        checks.report(status == 0 and output.startswith(f'{HOME}/Notes/inbox/') and output.endswith('.md'),
                      'a line of text becomes a note in the inbox, for a user who never set git up', output)
        status, output = user(f'cat "{output}"; ls ~/Notes; git -C ~/Notes/inbox log --oneline | wc -l')
        checks.report(output.startswith('# an idea about the greeter\n\n#kavat') and 'home' in output and output.endswith('2'),
                      'with its title, its tag under it, and its place in the history', output)

        status, output = user('kavat-notes add "call the bank #money tomorrow 9:00" < /dev/null 2>&1')
        note = output.splitlines()[0] if output else ''
        checks.report(status == 0 and note.endswith('.todo.md') and 'Reminder: ' in output,
                      'a day and a time at the end of the line set a reminder', output)
        status, output = user(f'head -n 4 "{note}"; {TIMERS}')
        tomorrow = user('date -d tomorrow +%Y-%m-%d')[1]
        checks.report(output == f'---\nremind: {tomorrow} 09:00\n---\n# [ ] call the bank\n1',
                      'which is a line in the note and a timer of the session', output)
        status, output = user('nb due < /dev/null 2>&1')
        checks.report(status == 0 and 'Upcoming' in output and 'call the bank' in output and 'Git requires' not in output,
                      'nb lists what is due, and asks for nothing first', output)
        status, output = user(f'nb remind --off "{note}" < /dev/null 2>&1; grep -c "^remind" "{note}"; {TIMERS}')
        checks.report(output.split() == ['0', '0'], 'taking the reminder off takes the line and the timer away', output)

        # the ways a time was really typed on the PC, which were not understood at first
        status, output = user('for line in "stretch in two minutes" "\\"today 23:59\\"" "tea in half an hour" "call mom at 9"; do '
                              'file=$(kavat-notes add "$line" 2> /dev/null < /dev/null); grep -c "^remind: " "$file"; nb remind --off "$file" > /dev/null < /dev/null; done')
        checks.report(output.split() == ['1', '1', '1', '1'], 'a time said in words, in quotes or in short is understood too', output)
        user('kavat-notes add "stand up in 1 minute" < /dev/null')
        until('a reminder comes up as a notification when its time comes',
              'pgrep -a -x notify-send', 'Reminder stand up', 90)
        time.sleep(2)
        machine.screen.picture(f'{pictures}/notes-reminder.png')
        status, output = user('kavat-notes due')
        checks.report('Overdue' in output and 'stand up' in output, 'one that was not answered is listed as overdue', output)
        user('pkill -x notify-send')

        print('the quick note', flush=True)
        machine.screen.press('super', 'n')
        opened = until('Super+N opens a line to type a note in, floating',
                       f'{WINDOWS} | grep -B6 "class: kavat-note$" | grep -E "size|floating" | tr -d "\\n\\t"',
                       'size: 900,150floating: 1')
        if opened:
            time.sleep(2)
            machine.screen.type('typed on the desktop #quick')
            time.sleep(1)
            machine.screen.picture(f'{pictures}/notes-quick.png')
            machine.screen.press('enter')
            until('Enter saves it to the inbox', 'grep -l -x "# typed on the desktop" ~/Notes/inbox/*.md | wc -l', '1')
            until('and the line closes', f'{WINDOWS} | grep -c "class: kavat-note$"; echo closed', '0\nclosed')

        print("the notes' own screen", flush=True)
        machine.run(f"cat > /var/tmp/screen.py << 'END'\n{SCREEN}END\nchmod 644 /var/tmp/screen.py")
        status, output = user('python3 /var/tmp/screen.py "3,5,1,a,bring the bins out friday,ENTER,WAIT,?" '
                              '"Notebooks,inbox,UPCOMING,call the bank,Reminders,Add:,bring the bins out,Help"', 60)
        checks.report(output == 'missing: \nended: 0', 'it shows the notes by what they are for, and adds one from a line', output)
        status, output = user('grep -l -x "# \\[ \\] bring the bins out" ~/Notes/inbox/*.md | wc -l; kavat-notes < /dev/null | head -n 1')
        checks.report(output.splitlines()[:1] == ['1'], 'the note it added is a task with its reminder, in the inbox', output)

        machine.run("printf '# Title\\n\\n- [ ] box\\n' > /tmp/box.md; chmod 666 /tmp/box.md")
        status, output = user('nvim --headless /tmp/box.md "+lua vim.api.nvim_win_set_cursor(0, {3, 0}) require([[kavat.notes]]).enter() '
                              'io.write(vim.api.nvim_get_current_line())" +qa! 2>&1', 60)
        checks.report(output == '- [x] box', 'in the editor, Enter checks the box under the cursor', output)
        status, output = user('hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/notes-*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
