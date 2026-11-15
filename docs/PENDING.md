# Willwitcher-OS Kavat Edition — Pending decisions

Updated on 15 November 2026 · [Design document](DESIGN.md) · [Interactive mockup](mockup/index.html)

## How it works

Only what waits for your decision is here. Everything else lives in the [design document](DESIGN.md) and in the mockup.

- **If you approve something**, it goes to the design document and to the mockup, and leaves this file.
- **If you reject it**, it is deleted from here and is not asked again.
- **To answer**, the number in the chat is enough ("D1 yes, D5 no"), or write the answer next to the decision in this file.

## Summary

| # | Decision | Recommendation |
| --- | --- | --- |
| D44 | Review the six AUR packages of the Development category for kav's list | Yes, one by one, after the manual tests |
| D45 | Where your aliases and functions from Endgame are | Point me at them |

## D44 · The AUR packages of Development

The installer's Software page offers only what is in Arch's own repositories, because the reviewed list of `kav` is empty. Six programs the design names for your work are only in the AUR: claude-code, supabase, stripe-cli, the Android SDK with its emulator, watchman and maestro.

Each has to be read before it goes on the list, which is what the list is for: `kav review <name>` shows its build file with the lines worth a second look marked, and you approve it or not. I can read the six first and tell you what each one downloads and runs, so that you approve them knowing it.

**Recommendation: yes, one by one, after the manual tests.** Once approved they go on the system's list, `/usr/share/kav/allowlist.toml`, pinned to the commit that was read, and the Software page offers them.

**If you say no**, they stay out of the installer, and you can still ask for any of them on your machine with `kav request` and `kav review`.

## D45 · Your aliases and functions from Endgame

The design says the terminal's shell inherits your aliases and functions from Willwitcher-OS Endgame. They are not in this repository, and I have not looked for them on your disks: the only disk I read was Kavat's own, for the notes, with your permission.

**What I need**: where they are. A repository of yours, or the file itself pasted in the chat, is enough; or permission to read your Endgame home folder, if it is on one of the disks of this PC, read-only as before.

**If you say no**, the shell stays with what it has: the system's prompt, its plugins and its tools.

Settled on 15 November 2026:

- **D41**, covers and pictures that come from the network: left as it is, not fetched.
- **D42**, `kavat-fetch` and `kavat-top` without forks: fastfetch stays without a fork for good; btop is forked in phase 4, with the agent.
- **D43**, the installer's steps: Calamares' own nine are kept, and the design and the mockup list them.

Settled on 14 November 2026:

- **D37**, the settings panel as a plugin of the shell or as C++ inside it: what counts is that the panel ends up as the mockup draws it, whatever it takes. It is built as a plugin as far as a plugin reaches that, and in C++ where it does not.
- **D38**, ripdrag now that Yazi drags by itself: dropped. Dragging is Yazi's own.
- **D39**, a name of its own for the shell: yes. It is Kavat Shell, with the command `kavat-shell`.
- **D40**, what the shell fetched from the network when it started: all of it removed or turned off. What was found and what was done is in the design document, under "Desktop".
