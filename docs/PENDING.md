# Willwitcher-OS Kavat Edition — Pending decisions

Updated on 15 November 2026 · [Design document](DESIGN.md) · [Interactive mockup](mockup/index.html)

## How it works

Only what waits for your decision is here. Everything else lives in the [design document](DESIGN.md) and in the mockup.

- **If you approve something**, it goes to the design document and to the mockup, and leaves this file.
- **If you reject it**, it is deleted from here and is not asked again.
- **To answer**, the number in the chat is enough ("D1 yes, D5 no"), or write the answer next to the decision in this file.

## Summary

There are no pending decisions.

Settled on 15 November 2026:

- **D41**, covers and pictures that come from the network: left as it is, not fetched.
- **D42**, `kavat-fetch` and `kavat-top` without forks: fastfetch stays without a fork for good; btop is forked in phase 4, with the agent.
- **D43**, the installer's steps: Calamares' own nine are kept, and the design and the mockup list them.

Settled on 14 November 2026:

- **D37**, the settings panel as a plugin of the shell or as C++ inside it: what counts is that the panel ends up as the mockup draws it, whatever it takes. It is built as a plugin as far as a plugin reaches that, and in C++ where it does not.
- **D38**, ripdrag now that Yazi drags by itself: dropped. Dragging is Yazi's own.
- **D39**, a name of its own for the shell: yes. It is Kavat Shell, with the command `kavat-shell`.
- **D40**, what the shell fetched from the network when it started: all of it removed or turned off. What was found and what was done is in the design document, under "Desktop".
