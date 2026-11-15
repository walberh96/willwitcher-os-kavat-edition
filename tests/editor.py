#!/usr/bin/env python3
"""Check the editor: Neovim as kavat-nvim sets it up, whole on a machine with no network.

usage: tests/editor.py [--encrypted]
  --encrypted   on the encrypted disk that tests/install.py --encrypt leaves

Works on the disk that tests/install.py leaves in out/test/, through a copy-on-write layer,
so that disk does not change. The machine has no network, so every plugin has to be there
already: the test loads them all and wants no complaint. Code has to be coloured by what it
means, Markdown has to get its language server, and the VS Code-style shortcuts have to be
set. The colours have to be the theme's, and change with it while the editor is open. Then,
on the desktop, it opens a file in a terminal, types into it straight away, saves it with
Ctrl+S and opens the picker with Ctrl+P.

Needs qemu-desktop and edk2-ovmf, and tests/install.py run first. It runs as a normal user
and takes about two minutes. Pictures of the screen are left in out/test/.
"""
import os
import sys
import time

from machine import BOOT_TIMEOUT, Checks, Machine, check_desktop, log_in, over_installed, print_journal

USER, PASSWORD = 'tester', 'kavat-123'  # as tests/install.py made them
HOME = f'/home/{USER}'
PLUGINS = 18  # as editor/PKGBUILD lists them
SOCKET = '/tmp/nvim-test'  # where the test's Neovim listens
# commands as the user, inside the user's session
SESSION = (f'runuser -u {USER} -- env HOME={HOME} XDG_RUNTIME_DIR=/run/user/$(id -u {USER}) '
           f'WAYLAND_DISPLAY=wayland-1 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u {USER})/bus sh -c ')
# Lua that loads every plugin and says how many did; a complaint goes to the output too
LOAD = ('local names = {} '
        'for _, plugin in ipairs(require("lazy").plugins()) do names[#names + 1] = plugin.name end '
        'require("lazy").load({ plugins = names }) vim.wait(500) '
        'local loaded = 0 '
        'for _, plugin in ipairs(require("lazy").plugins()) do if plugin._.loaded then loaded = loaded + 1 end end '
        'io.write("loaded ", loaded, " of ", #names, " scheme ", vim.g.colors_name or "none", "\\n")')
# the description of what some keys do in normal mode, one to a line
KEYS = {'<C-P>': 'Find a file', '<C-S-P>': 'Command palette', '<C-S>': 'Save', '<C-W>': 'Close the tab',
        '<C-D>': 'A cursor at the next match', '<C-B>': 'Explorer', '<F12>': 'Go to the definition',
        '<M-Down>': 'Move the line down', '<Space>ff': 'Files'}
BACKGROUND = 'printf("%06x", nvim_get_hl(0, {"name": "Normal"}).bg)'


def main():
    if sys.argv[1:] not in ([], ['--encrypted']):
        sys.exit(__doc__)
    pictures, machine_settings = over_installed('editor', encrypted=len(sys.argv) > 1)
    checks = Checks()
    machine = Machine(**machine_settings)

    def user(command, timeout=30):
        return machine.run(SESSION + "'" + command.replace("'", "'\\''") + "'", timeout)

    def lua(code, file=''):
        """Run Lua in a Neovim of the user's with no screen, and give back what it says."""
        machine.run(f"cat > /tmp/test.lua << 'END'\n{code}\nEND")
        return user(f'cd ~ && nvim --headless {file} "+luafile /tmp/test.lua" +qa! 2>&1', 60)

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

        print('the editor with no screen', flush=True)
        status, output = user('cat ~/.config/nvim/init.lua | grep -c "require(\\"kavat\\").setup()"; echo $EDITOR')
        checks.report(output.split()[:1] == ['1'], "the user's configuration is one line that brings in the system's", output)
        status, output = lua(LOAD)
        checks.report(status == 0 and output == f'loaded {PLUGINS} of {PLUGINS} scheme kavat',
                      'every plugin loads with no network and no complaint, and the colours are the system\'s', output)
        status, output = user('ls ~/.local/share/nvim/lazy 2>/dev/null | wc -l')
        checks.report(output == '0', 'nothing was fetched: the plugins are the ones the package brings', output)
        status, output = user('nvim --headless --startuptime /tmp/startup.log +qa! && grep "NVIM STARTED" /tmp/startup.log | cut -d " " -f 1')
        took = float(output) if output.replace('.', '').isdigit() else None
        checks.report(took is not None and took < 150, 'the editor opens in under 150 ms', f'{output} ms')

        machine.run("printf 'fn main() {\\n    let x = 1;\\n}\\n' > /tmp/code.rs; printf '# Notes\\n\\nSee [other](other.md).\\n' > /tmp/notes.md; "
                    'chmod 644 /tmp/code.rs /tmp/notes.md')
        status, output = lua('vim.wait(300) io.write(tostring(vim.treesitter.highlighter.active[vim.api.nvim_get_current_buf()] ~= nil), "\\n")',
                             '/tmp/code.rs')
        checks.report(output == 'true', 'code is coloured by what it means', output)
        status, output = lua('vim.wait(4000, function() return #vim.lsp.get_clients({ bufnr = 0, name = "marksman" }) > 0 end) '
                             'local clients = vim.lsp.get_clients({ bufnr = 0, name = "marksman" }) '
                             'io.write(clients[1] and clients[1].name or "none", "\\n")',
                             '/tmp/notes.md')
        checks.report(output == 'marksman', 'Markdown gets its language server', output)
        status, output = lua('\n'.join(f'io.write(vim.fn.maparg("{keys}", "n", false, true).desc or "none", "\\n")' for keys in KEYS))
        checks.report(output.splitlines() == list(KEYS.values()), 'the VS Code-style shortcuts are set', output)

        print('the colours follow the theme', flush=True)
        user(f'rm -f {SOCKET}; cd ~ && setsid -f nvim --headless --listen {SOCKET} /tmp/code.rs > /dev/null 2>&1')
        ask = f"nvim --server {SOCKET} --remote-expr '{BACKGROUND}'"
        until("the editor's background is Kavat's dark surface", ask, '151821')
        status, output = user('kavat-theme set mode light')
        checks.report(status == 0, 'the theme changes to light', output)
        until('and the open editor turns to paper', ask, 'fbf9f4', 5)
        user(f"kavat-theme set mode dark; nvim --server {SOCKET} --remote-send '<Cmd>qa!<CR>'")

        print('the editor on the desktop', flush=True)
        machine.screen.press('super', 'enter')
        until('a terminal opens', 'hyprctl --instance 0 clients | grep -c "class: kitty"', '1')
        time.sleep(2)
        machine.screen.type('nvim typed.txt')
        machine.screen.press('enter')
        until('Neovim opens in it', 'pgrep -c -x nvim', '2')
        time.sleep(2)
        machine.screen.type('written at once')
        # the screen the test types through loses a key now and then: Ctrl+S is pressed again
        for _ in range(2):
            machine.screen.press('ctrl', 's')
            if machine.wait_until(f'grep -q "written at once" {HOME}/typed.txt', 4)[0] == 0:
                break
        until('it opens ready to type, and Ctrl+S saves', 'cat ~/typed.txt', 'written at once', 2)
        machine.screen.press('ctrl', 'p')
        until('Ctrl+P opens the picker', 'pgrep -c -x fzf', '1')
        time.sleep(1.5)
        machine.screen.picture(f'{pictures}/editor-picker.png')
        machine.screen.press('esc')
        time.sleep(1)
        machine.screen.press('ctrl', 'b')
        time.sleep(1.5)
        machine.screen.picture(f'{pictures}/editor.png')
        status, output = user('hyprctl --instance 0 configerrors')
        checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)
        if checks.failed:
            print_journal(machine)
    finally:
        machine.stop()
    print(f'pictures of the screen: {os.path.relpath(pictures)}/editor*.png')
    sys.exit(1 if checks.failed else 0)


if __name__ == '__main__':
    main()
