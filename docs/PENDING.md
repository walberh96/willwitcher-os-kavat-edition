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
| D41 | Album covers and pictures in notifications that come from the network are no longer shown | Leave it so; a switch if you miss them |
| D42 | `kavat-fetch` and `kavat-top` without forks | fastfetch without a fork for good; btop forked in phase 4, with the agent |
| D43 | The installer has Calamares' nine steps, not the seven of the design | Keep the nine |

## D41 · Covers and pictures from the network

With D40 the shell stopped fetching anything by itself. Two things went with that, and you have not said whether you want them back:

- **The cover of what is playing.** A player gives the shell the address of the cover. When that address is on the network, as with Spotify or a browser, the bar's media widget now shows no cover. A cover that is a file on the disk is still shown.
- **The picture of a notification** that comes as an address on the network, which a few chat apps send.

**Recommendation: leave it so.** It is what "asks the network for nothing" means, and the address of a cover tells a server what you are listening to. If you miss the covers when you try it, it becomes a switch on the Bar & widgets page, "Fetch covers from the network", off as it comes.

**If you say no**, the shell fetches both again, as Noctalia did.

## D42 · `kavat-fetch` and `kavat-top` without forks

The design says both are forks. They are built without one: fastfetch and btop as Arch packages them, each with a file `kavat-theme` writes.

- **fastfetch needs no fork.** The cat as its logo, the lines you choose, the colours of the theme, and the lines of the system itself (the theme and its mode, the packages with those from the AUR) are all done from its settings file: a line can be the output of a command. What is left for a fork is the cat as an image in Kitty, which its settings also allow.
- **btop does need one** for what the design asks: asking Kavat about a process, the groups, the box of the GPU with the agent's share, the services. All of it leans on the agent or on `kavatd`, which are phase 4.

**Recommendation: fastfetch stays without a fork for good, and the design says so; btop is forked in phase 4**, when there is an agent to ask. Until then it is btop with the theme's colours.

**If you say no**, both are forked now: two more programs to build and keep up to date, with nothing to show for it in phase 3.

## D43 · The installer's steps

The design lists seven steps: language and keyboard, network, disk, user, hardware, appearance, summary. What is built has Calamares' own nine: Welcome, Location, Keyboard, Partitions, Users, Appearance, Summary, Install, Finish.

The difference is that Calamares has a page for each of its modules, and joining two of them in one page, or adding Network and Hardware, means writing those pages from nothing, as the Appearance one was.

**Recommendation: keep the nine**, and the design's list is changed to them. Hardware (the graphics driver) is detected without asking, and the network is not needed to install. The mockup's installer is redrawn with the nine steps.

**If you say no**, the pages are written to match the seven: about as much work as the whole installer look was.

Settled on 14 November 2026:

- **D37**, the settings panel as a plugin of the shell or as C++ inside it: what counts is that the panel ends up as the mockup draws it, whatever it takes. It is built as a plugin as far as a plugin reaches that, and in C++ where it does not.
- **D38**, ripdrag now that Yazi drags by itself: dropped. Dragging is Yazi's own.
- **D39**, a name of its own for the shell: yes. It is Kavat Shell, with the command `kavat-shell`.
- **D40**, what the shell fetched from the network when it started: all of it removed or turned off. What was found and what was done is in the design document, under "Desktop".
