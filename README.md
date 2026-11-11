<div align="center">

```
 ██╗    ██╗██╗██╗     ██╗     ██╗    ██╗██╗████████╗ ██████╗██╗  ██╗███████╗██████╗
 ██║    ██║██║██║     ██║     ██║    ██║██║╚══██╔══╝██╔════╝██║  ██║██╔════╝██╔══██╗
 ██║ █╗ ██║██║██║     ██║     ██║ █╗ ██║██║   ██║   ██║     ███████║█████╗  ██████╔╝
 ██║███╗██║██║██║     ██║     ██║███╗██║██║   ██║   ██║     ██╔══██║██╔══╝  ██╔══██╗
 ╚███╔███╔╝██║███████╗███████╗╚███╔███╔╝██║   ██║   ╚██████╗██║  ██║███████╗██║  ██║
  ╚══╝╚══╝ ╚═╝╚══════╝╚══════╝ ╚══╝╚══╝ ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
                         O S   ·   K A V A T   E D I T I O N
```

**An Arch that already knows you.**

Fast like a tiling WM. Curated like one person's system. And with someone inside who listens.

`Arch Linux` · `Hyprland` · `Noctalia` · `Calamares` · `kav` · `Neovim` · `Custom Chromium` · `Local agent`

**[Interactive mockup](docs/mockup/index.html)** · **[Design document](docs/DESIGN.md)** (in Spanish) · **[Screenshots](docs/screenshots/)**

</div>

![The Willwitcher-OS Kavat Edition desktop](docs/screenshots/desktop.png)

> **Status: phase 0, design.** This README describes the finished system. The images come from an interactive mockup, not from a running system: nothing is implemented yet. Details in [Real project status](#real-project-status).

---

## What it is

Willwitcher-OS Kavat Edition is an Arch Linux based distribution that boots finished.

It is not a script that decorates a fresh Arch install. It is its own ISO, with its own installer, its own repository and a local agent that understands the system from the inside. You turn it on, pick a theme, and ten minutes later you are working on a desktop you never had to configure.

Underneath it is still Arch: rolling release, pacman, the whole wiki on your side. On top it is something else.

## See it

The mockup is a single HTML file with no dependencies: open [`docs/mockup/index.html`](docs/mockup/index.html) in a browser. Everything inside the monitor responds: switch screens, palettes, modes and wallpapers, and walk through the sixteen settings pages and the seven `kav` tabs.

| | |
| --- | --- |
| ![Login](docs/screenshots/login.png) **Login.** LightDM with its own theme, over your blurred wallpaper. | ![The agent](docs/screenshots/agent.png) **Kavat.** The agent, its clickable skills, and every tool it uses with its permission level. |
| ![Light theme](docs/screenshots/themes-light.png) **Themes.** Palette, mode, accent and shape, live. | ![Wallpapers](docs/screenshots/settings-wallpaper.png) **Wallpapers.** Image or video, from the settings panel. |
| ![Modes](docs/screenshots/settings-modes.png) **Modes.** Coder, Gamer and Incognito. | ![Skills](docs/screenshots/settings-skills.png) **Skills.** What the agent does with one click, one phrase or one key. |
| ![Agent and voice](docs/screenshots/settings-agent.png) **Agent and voice.** Model and voice share 4 GB. | ![kav](docs/screenshots/kav-dashboard.png) **`kav`.** The package manager, as a TUI. |
| ![AUR review](docs/screenshots/kav-aur-review.png) **AUR review.** The diff, with the suspicious lines flagged. | ![Editor](docs/screenshots/editor-marketplace.png) **Editor.** Neovim with a plugin marketplace. |
| ![Browser](docs/screenshots/browser.png) **Browser.** Command console and the agent's side panel. | ![Installer](docs/screenshots/installer.png) **Installer.** Calamares, appearance step. |

## Talk to it

```
 Super + Space

 › dark mode and dim the screen, I'm coding tonight
   ✓ night profile applied · brightness 35%

 › what's eating my RAM
   Kavat Browser 4.1 GB (63 tabs) · docker 2.8 GB · me 3.6 GB

 › install obs
   obs-studio is in the official Arch repos. Snapshot #215 first.
   Install? [y/N]
```

**Kavat**, the agent, runs on your machine. No cloud, no account, no telemetry. It is a compact model on Ollama, tuned for this system and only this system: it knows your shortcuts, your packages, your configuration and the OS documentation. It understands English and Spanish.

- **It fits next to a game.** Model and voice share 4 GB at most, and unload from memory when you are not using them.
- **It has a voice.** It listens and speaks through VoiceStudio, also locally.
- **It has no shell.** It talks to `kavatd`, a daemon of typed tools with four permission levels. Reading is free. Changing a theme is free and logged. Installing something asks first and takes a snapshot. Touching disks, users or the network asks for your password.
- **It cannot grant itself permissions**, and everything it does can be undone.

**Skills.** A skill is an action the agent already knows how to do, such as opening the browser. It shows up as a button in the agent's panel: click it, say it or type it.

- **A click does not need the model.** It runs the skill directly, so it answers instantly and still works in Gamer mode with the model unloaded.
- **Your own skills.** Chain steps into one: "start stream" opens OBS, switches to Gamer mode and silences notifications. A skill is a TOML file.
- **Save as skill.** When the agent solves something in several steps, one button keeps it to repeat the same way next time.
- **Pin, bind, schedule.** Pin favourites to the panel, the bar or the launcher, give them a key or a voice phrase, or let them run on their own.

## Three modes

A selector in the bar switches the whole machine. It feels like picking a power profile, but each mode decides what runs, where the network goes out and what is remembered.

| | Coder | Gamer | Incognito |
| --- | --- | --- | --- |
| Agent | On | Off, model and voice out of memory | On, keeps no history |
| Resources | Normal | GPU and memory for the game; everything spare is stopped | Normal |
| Network | Direct | Direct | Everything through the VPN, cut off if it drops |
| Browser | Normal | Tabs suspended | New windows open private |

Gamer turns on by itself when a game starts from Steam and off when it closes. Animations stay: the mode frees resources by stopping processes, not by stripping the look. A mode of your own is one more TOML file.

## `kav`: one manager, one list

The AUR is the best and the worst idea in Arch. Here it only comes in through the door.

```
$ kav install visual-studio-code-bin
  ▸ AUR · allowlisted · reviewed 2026-11-11 · commit 3f9c2ab
  ▸ building in an isolated chroot…  ✓
  ▸ snapshot #215 created · installed

$ kav install random-package-git
  ✗ Not on the allowlist.
    Run kav request random-package-git to ask for a review.
```

`kav` is a layer over pacman that resolves every package in three tiers:

| Tier | Source | Trust |
| --- | --- | --- |
| 1 | Official Arch repositories | Full |
| 2 | The `[kavat]` repository, signed binaries | Built and signed by the project |
| 3 | AUR packages on the allowlist | Pinned to the exact commit that was reviewed |

An allowed package whose PKGBUILD changes does not update until someone reviews the diff. A hijacked AUR package never reaches your disk. `kav undo` returns the system to the state before any transaction.

With no arguments, `kav` opens a full-screen TUI with seven tabs: Dashboard, Browse, Updates, AUR, Snapshots, Doctor and Settings. That is where you review a PKGBUILD diff with the suspicious lines flagged, walk the snapshot timeline and merge a `.pacnew` without leaving.

## One system, every skin

Appearance is not a folder of dotfiles. It is a set of independent axes that combine:

```
palette     kavat · catppuccin · gruvbox · tokyo-night · nord · rosé-pine · everforest · from-wallpaper
mode        light · dark · auto
accent      any colour
tone        neutral · warm · cool · oled
contrast    normal · high
shape       square ←——→ round · compact ←——→ roomy
effects     blur · transparency · shadows · animation
typography  interface · mono · scale
wallpaper   image · video
```

Change one value and everything changes at once, in under a second and without restarting anything: Hyprland, the shell, the terminal, GTK, Qt, Neovim, the browser, the login screen, the lock screen, the boot splash and even the installer. If one piece fails, none is applied.

Light mode is designed, not inverted. Profiles have names (`work`, `night`, `presenting`) and can switch on their own by time of day or by the monitor you plug in. A third-party theme is a single TOML file.

**The wallpaper can move.** An image is the default; a looping video is one more option in the same grid. The video pauses by itself when you cannot see it, when something is fullscreen and in Gamer mode, and the login screen uses a still frame.

## Everything is set in one place

A settings panel inside Noctalia configures the whole core: appearance, wallpaper, windows, keybindings, displays, terminal, editor, shell, browser, bar, modes, agent and voice, skills, packages, login and language.

- **One source.** The panel writes to `~/.config/kavat/settings.toml`, and each app's configuration is generated from it.
- **Live and reversible.** Every change shows instantly, has undo and lands in a history.
- **Your files win.** If you edit Hyprland's or Kitty's configuration by hand, the panel marks that setting as overridden instead of overwriting it.
- **Nothing is left half-installed.** Pick an option that needs a program, such as mpvpaper for video wallpapers, and `kav` installs it before the setting applies. It then refuses to remove it by accident, and `kav doctor` checks it is still there.
- **The agent uses the same thing.** "Raise the gaps to 10" changes the same value the panel does.
- **Reinstalling is restoring.** Your whole state fits in one file.

## The desktop

- **Hyprland**, with modular configuration: the system's in `/usr/share/kavat/`, yours in `~/.config/`, and yours always wins.
- **Noctalia**, in its own fork: bar, launcher, notifications, control centre, plus the agent's panel, the mode selector and the settings panel.
- **LightDM + web-greeter**, with a login screen that uses your theme and your wallpaper. Entering the desktop has no visual cut.
- **Kitty and zsh**, with a starship prompt cut down to what costs no time: opening a terminal goes unnoticed.
- **The shortcuts you already know**, inherited from Willwitcher-OS Endgame, plus `Super+Space` for Kavat.

## The editor

Neovim is part of the core, and it is useful the moment you open it.

- **Vim modes, VS Code navigation.** `Ctrl+P`, `Ctrl+Shift+P`, `Ctrl+S`, `Ctrl+/`, `Ctrl+D`, arrows and `Shift` selection work in any mode. `Esc` is still there.
- **Preinstalled.** File explorer, fuzzy finder, command palette, tabs, LSP with completion, format on save, git, floating terminal and multicursor.
- **A marketplace inside the editor.** `Ctrl+Shift+X` opens a catalogue of plugins by category. Verified ones bring their tested configuration and shortcuts, pinned to a commit; installing is one step.
- **Language packs.** "Rust" installs the LSP server, formatter, linter and debugger in one go.

## The browser

A Chromium of its own, with five changes.

- **It follows the modes.** In Incognito it opens private windows; in Gamer it suspends tabs.
- **It follows the theme.** It changes colour with the rest of the system, internal pages included.
- **It blocks out of the box.** Ads and trackers, from the first start.
- **It has Kavat beside it.** A panel that reads the page and acts in the browser, with the same permissions as in the system.
- **It has a console.** A palette of its own commands: `hide-tabs`, `hide-sidebar`, `zen`, and the ones you define.

## Unbreakable by design

- **Btrfs + snapshots** before every package transaction.
- **Bootable snapshots**: the latest system states are in the boot menu.
- **Fallback kernel**: `linux-zen` for every day, `linux-lts` for when something breaks.
- **LUKS2 encryption** with TPM2 unlock, **Secure Boot** with its own keys, firewall closed by default.
- **`kav update` reads the Arch news** and stops if an update needs manual intervention.
- **`kav doctor`** tells you what is wrong before you notice.

## Installation

A Calamares installer with its own identity. Seven screens, no network, about ten minutes. English by default, with Spanish already installed as the second option.

1. Language and keyboard
2. Disk (encryption is one switch)
3. User
4. Detected hardware and suggested drivers
5. Software profile
6. Appearance, with a live preview
7. Install

Already on Arch? `pacman -S kavat-desktop` converts it.

## Anatomy

```
┌───────────────────────────────────────────────┐ ┌──────────────────┐
│ Experience    Noctalia · greeter · Calamares  │ │  Kavat (agent)   │
├───────────────────────────────────────────────┤ │  Ollama + model  │
│ Compositor    Hyprland · hyprlock · hypridle  │ │  + voice         │
└───────────────────────────────────────────────┘ └────────┬─────────┘
┌──────────────────────────────────────────────────────────▼─────────┐
│ Kavat services      kav · kavat-theme · kavatd                     │
├────────────────────────────────────────────────────────────────────┤
│ Packages            Arch official · [kavat] · AUR by allowlist     │
├────────────────────────────────────────────────────────────────────┤
│ Base                zen + lts · Btrfs · systemd-boot · LUKS2       │
└────────────────────────────────────────────────────────────────────┘
```

## Documentation

| What | Where |
| --- | --- |
| Design document: every piece, every decision and what is still to be verified (in Spanish) | [`docs/DESIGN.md`](docs/DESIGN.md) |
| Interactive mockup of the interface | [`docs/mockup/index.html`](docs/mockup/index.html) |
| Screenshots of the mockup | [`docs/screenshots/`](docs/screenshots/) |
| The cat's ASCII art for the terminal | [`docs/assets/kavat-ascii.txt`](docs/assets/kavat-ascii.txt) |

## The repository

Today the repository holds this README and the `docs/` folder. The rest arrives with each phase:

| Folder | Contents | Phase |
| --- | --- | --- |
| `iso/` | archiso profile, package list, live system overlay | 1 |
| `packages/` | One PKGBUILD per `kavat-*` package, forks included | 1 |
| `installer/` | Calamares configuration, modules and branding | 2 |
| `themes/` | Tokens, templates and bundled themes | 3 |
| `greeter/` | web-greeter theme | 3 |
| `kav/` | The package manager, its TUI and the AUR allowlist | 3 |
| `agent/` | `kavatd`, tools, skills and the agent's Modelfile | 4 |

## Principles

1. **Arch underneath, no magic.** Everything is a package. What the system puts there, pacman can remove.
2. **Curated, not open.** Fewer options, better chosen.
3. **A single source of truth** for appearance and settings.
4. **Always reversible.** Breaking it costs a reboot.
5. **The agent is part of the system**, not an app. And it is yours.

## Why "Kavat"

A kavat is a companion feline: fast, quiet, and always beside whoever raised it. That is what this edition wants to be.

---

## Real project status

This README describes the finished system. Today the project is in **phase 0, design**: none of the above is implemented yet. What exists is the [design document](docs/DESIGN.md), with every decision made, and the [mockup](docs/mockup/index.html) of the interface.

| Phase | Deliverable | Status |
| --- | --- | --- |
| 0 | Design closed and documented | Done, pending review |
| 1 | ISO that boots to Hyprland + Noctalia | Pending |
| 2 | Calamares installer with Btrfs and snapshots | Pending |
| 3 | Themes, settings, greeter, Noctalia fork, `kav` and editor | Pending |
| 4 | Local agent, skills and voice | Pending |

The design takes several assumptions for granted that have not been tested. They are listed, with the phase in which each gets verified, at the end of the [design document](docs/DESIGN.md#decisiones).
