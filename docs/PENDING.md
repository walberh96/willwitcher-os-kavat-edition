# Willwitcher-OS Kavat Edition — Pending decisions

Updated on 16 November 2026 · [Design document](DESIGN.md) · [Interactive mockup](mockup/index.html)

## How it works

Only what waits for your decision is here. Everything else lives in the [design document](DESIGN.md) and in the mockup.

- **If you approve something**, it goes to the design document and to the mockup, and leaves this file.
- **If you reject it**, it is deleted from here and is not asked again.
- **To answer**, the number in the chat is enough ("D1 yes, D5 no"), or write the answer next to the decision in this file.

## Summary

Nothing waits for a decision.

Settled on 16 November 2026:

- **D50**, remote access: yes, as proposed. Tailscale, the SSH server and Sunshine behind one switch, off as the system comes, on a page of its own; Tailscale's servers coordinate, with a field for a server of yours, and the terminal comes in with your keys. It is in the design document, under "Security and recovery".
- **D45**, the aliases and functions from Endgame, gone over: two for now, `port-kill <port>` and `y`, Yazi's wrapper. They are in the design document, under "Preinstalled software".
- **D46**, Kavat Browser's last part, which needs Chromium compiled: yes, it is compiled here. It was known from the start.
- **D47**, the network said in the installer: as the mockup draws it, on the bar of the live session; and the bar's network widget, there and on the installed system, opens the Network page when pressed. The installer's side bar does not say it, and nothing is asked of a mirror.
- **D48**, logging in by itself and the keyring: nothing asks for a password after that login. It is chosen in the installer, on the page where the password is set; that person's passwords are kept in a keyring with no password.
- **D49**, a VPN from its provider's file: yes, and Proton VPN's own app is an option too, in the installer, on the Network page and as the VPN of Incognito.

Settled on 15 November 2026:

- **D44**, the six AUR packages of the Development category (claude-code, supabase, stripe-cli, the Android SDK with its emulator, watchman and maestro): yes, reviewed one by one for kav's list, after the manual tests. Until then the installer's Software page offers only what is in Arch's repositories.
- **D45**, the aliases and functions from Endgame: they are in the `.zshrc` of <https://github.com/walberh96/willwitcher-OS-Endgame>, which may be read. They are left for later: many of them will be changed and some added first. Their place is ready, `~/.config/kavat/aliases.zsh`.
- **D41**, covers and pictures that come from the network: left as it is, not fetched.
- **D42**, `kavat-fetch` and `kavat-top` without forks: fastfetch stays without a fork for good; btop is forked in phase 4, with the agent.
- **D43**, the installer's steps: Calamares' own nine are kept, and the design and the mockup list them.

Settled on 14 November 2026:

- **D37**, the settings panel as a plugin of the shell or as C++ inside it: what counts is that the panel ends up as the mockup draws it, whatever it takes. It is built as a plugin as far as a plugin reaches that, and in C++ where it does not.
- **D38**, ripdrag now that Yazi drags by itself: dropped. Dragging is Yazi's own.
- **D39**, a name of its own for the shell: yes. It is Kavat Shell, with the command `kavat-shell`.
- **D40**, what the shell fetched from the network when it started: all of it removed or turned off. What was found and what was done is in the design document, under "Desktop".
