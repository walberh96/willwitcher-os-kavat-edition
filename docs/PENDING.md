# Willwitcher-OS Kavat Edition — Pending decisions

Updated on 14 November 2026 · [Design document](DESIGN.md) · [Interactive mockup](mockup/index.html)

## How it works

Only what waits for your decision is here. Everything else lives in the [design document](DESIGN.md) and in the mockup.

- **If you approve something**, it goes to the design document and to the mockup, and leaves this file.
- **If you reject it**, it is deleted from here and is not asked again.
- **To answer**, the number in the chat is enough ("D35 no, D36 yes"), or write the answer next to the decision in this file.

## Summary

| # | Decision | Recommendation |
| --- | --- | --- |
| D35 | Hold the loading screen for a few seconds so its animation can be seen | No, for now: try the fixed boot first |
| D36 | One file search for the whole system: terminal, Neovim, launcher and Kavat | Yes, as described below |

## D35 · A minimum time for the loading screen

You asked whether the loading screen could stay 3 to 5 seconds, because the PC boots so fast that you never saw it.

**What happened.** You did not see it because of a fault, not because of speed. The loading screen was waiting for the graphics driver, which on the RX 6800 is ready 8.5 seconds into the boot, and the login arrives at second 11. With the fix that is being tested, it draws from second 3.6: about seven seconds of animation with no wait added.

**Recommendation: no, for now.** Try the fixed boot first. A wait makes every boot slower on purpose, and the reason for it may be gone.

**If you still want it after that**, it becomes a setting of the Boot page, "Keep the loading screen at least N seconds", off by default. The login simply waits until the loading screen has been up that long.

**If you say no**, nothing changes.

## D36 · File search

You asked for a file search that is instant, in the terminal, in Neovim and through Kavat, with the files indexed during the installation.

**The proposal: one engine, four doors.**

- **The engine** is two tools that are already in the system, plus an index:
  - `fd` walks your home folder live. On an NVMe disk it lists hundreds of thousands of files in a fraction of a second, and it is never out of date, so your own files need no index.
  - `plocate` keeps an index of the whole disk and answers in milliseconds. The installer builds it as its last step, so the first search after installing is already instant; after that it is rebuilt once a day and after every `kav` transaction.
  - `ripgrep` searches inside files, live.
- **The picker is `fzf` everywhere**, so the keys and the look are the same in all four doors. It is already installed, it is the fastest of its kind in practice, and both zsh and Neovim have first-class support for it. The alternatives (skim, television) bring nothing that pays for a second tool.
- **Door 1, the terminal.** A command, `kavat-find`, opens the search full screen: type to filter, a preview of the file on the right, `Enter` opens it, `Ctrl+O` opens its folder in Yazi. One key switches between your files, the whole disk and text inside files.
- **Door 2, zsh.** `Tab` shows the completions in the same picker instead of zsh's own menu, with a preview: files, folders, options, processes to kill. This is the `fzf-tab` plugin. `Ctrl+R` already searches the history this way; `Ctrl+T` pastes a file's path into the command; `Alt+C` jumps to a folder; and one new key searches text inside files and opens the result in Neovim at that line.
- **Door 3, Neovim.** `fzf-lua` is the one picker: files, text, buffers, recent files, help. It takes the place of Telescope, which the design listed beside it and which does the same more slowly. Your `<leader>ff`, `fg`, `fb` and `fr` stay as they are.
- **Door 4, the desktop and Kavat.** A shortcut opens the search as a floating window from anywhere, and the launcher gets a files mode. Kavat gets a `find_files` tool in phase 4 that asks the same engine, so "find the invoice from March" works by voice.

**What it does not do.** It does not index what files say, only their names: searching inside files stays live with `ripgrep`, which is fast enough for a home folder and always correct.

**Open points inside it**, each with my default if you say nothing:

- The key that opens the search on the desktop: one that is free in your Endgame shortcuts. I propose it when I draw the mockup.
- `fzf-tab` may not be in Arch's repositories: if it is not, it is packaged in `[kavat]`.

**If you say no**, the terminal keeps `Ctrl+R`, `Ctrl+T` and `Alt+C` as they are today, and Neovim keeps the list of plugins the design has.

**Once you approve it**, I draw it in the mockup before building anything.
