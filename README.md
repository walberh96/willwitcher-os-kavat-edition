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

`Arch Linux` · `Hyprland` · `Noctalia` · `Calamares` · `kav` · `Neovim` · `Yazi` · `Custom Chromium` · `Local agent`

**[Watch the tour](docs/video/kavat-tour.mp4)** · **[Interactive mockup](docs/mockup/index.html)** · **[Design document](docs/DESIGN.md)** (in Spanish) · **[Screenshots](docs/screenshots/)**

</div>

![The Willwitcher-OS Kavat Edition desktop](docs/screenshots/desktop.png)

> **Status: phase 0, design.** This README describes the finished system. The images come from an interactive mockup, not from a running system: nothing is implemented yet. Details in [Real project status](#real-project-status).

---

## What it is

Willwitcher-OS Kavat Edition is an Arch Linux based distribution that boots finished.

It is not a script that decorates a fresh Arch install. It is its own ISO, with its own installer, its own repository and a local agent that understands the system from the inside. You turn it on, pick a theme, and ten minutes later you are working on a desktop you never had to configure.

Underneath it is still Arch: rolling release, pacman, the whole wiki on your side. On top it is something else.

## See it

The mockup is a single HTML file with no dependencies: open [`docs/mockup/index.html`](docs/mockup/index.html) in a browser. Everything inside the monitor responds: switch screens, palettes, modes and wallpapers, and walk through the twenty-four settings pages and the seven `kav` tabs.

| | |
| --- | --- |
| ![Boot menu](docs/screenshots/boot-menu.png) **Boot menu.** GRUB with the system theme, safe mode and bootable snapshots. | ![Loading screen](docs/screenshots/loading.png) **Loading screen.** Plymouth, in the same identity. |
| ![Login](docs/screenshots/login.png) **Login.** LightDM with its own theme, over your blurred wallpaper. | ![The agent](docs/screenshots/agent.png) **Kavat.** The agent, its clickable skills, and every tool it uses with its permission level. |
| ![Light theme](docs/screenshots/themes-light.png) **Themes.** Palette, mode, accent and shape, live. | ![Wallpapers](docs/screenshots/settings-wallpaper.png) **Wallpapers.** Image or video, from the settings panel. |
| ![Modes](docs/screenshots/settings-modes.png) **Modes.** Coder, Gamer and Incognito. | ![Skills](docs/screenshots/settings-skills.png) **Skills.** What the agent does with one click, one phrase or one key. |
| ![Agent and voice](docs/screenshots/settings-agent.png) **Agent and voice.** At most 4 GB of the GPU. | ![kav](docs/screenshots/kav-dashboard.png) **`kav`.** The package manager, as a TUI. |
| ![AUR review](docs/screenshots/kav-aur-review.png) **AUR review.** The diff, with the suspicious lines flagged. | ![Editor](docs/screenshots/editor-marketplace.png) **Editor.** Neovim with a plugin marketplace. |
| ![Browser](docs/screenshots/browser.png) **Browser.** Command console and the agent's side panel. | ![Installer](docs/screenshots/installer.png) **Installer.** Calamares, appearance step. |
| ![Claude panel](docs/screenshots/claude-panel.png) **Claude beside the code.** A Kitty split next to Neovim; edits arrive as diffs. | ![Copy pop-up](docs/screenshots/files-copy.png) **Copying you can see.** Progress and name conflicts, in a pop-up. |
| ![Files](docs/screenshots/files.png) **Files.** Yazi, with previews and a drag-and-drop shelf. | ![File picker](docs/screenshots/files-picker.png) **File picker.** The browser asks for a file and Yazi answers. |
| ![Notes](docs/screenshots/notes.png) **Notes.** To-dos with dates, reminders and links to folders, on top of `nb`. | ![Quick note](docs/screenshots/notes-quick.png) **Quick note and reminders.** `Super+N` from anywhere; a reminder arrives as a notification. |
| ![A note in Neovim](docs/screenshots/note-rendered.png) **A note in Neovim.** Rendered in place: headings, checkboxes, tables and captures. | ![Clip to notes](docs/screenshots/browser-clip.png) **Clip to notes.** The page, your selection and a capture, from the browser. |
| ![Albums](docs/screenshots/browser-albums.png) **Albums.** Favourite sites as a grid of thumbnails. | ![Notes settings](docs/screenshots/settings-notes.png) **Notes settings.** Reminders, links, what the agent may read, and sync. |
| ![Launcher](docs/screenshots/launcher.png) **One palette.** Apps, files, settings, skills, packages and windows from one place. | ![Boot settings](docs/screenshots/settings-boot.png) **Boot settings.** The menu and the loading screen, from the panel. |
| ![GPU settings](docs/screenshots/settings-gpu.png) **GPU.** Fan curve, limits and per-game profiles. | ![Projects](docs/screenshots/settings-projects.png) **Projects.** A whole workspace, saved and reopened. |
| ![Backup and sync](docs/screenshots/settings-backup.png) **Backup and sync.** Home snapshots, external copies and configuration history. | ![Privacy and network](docs/screenshots/settings-privacy.png) **Privacy.** Who used the microphone, camera or screen, and the per-app firewall. |
| ![Windows settings](docs/screenshots/settings-windows.png) **Two-way settings.** Change a value in the panel or in the file; each follows the other. | ![Config in Neovim](docs/screenshots/config-in-neovim.png) **One click to the file.** Every page opens its config in Neovim. |
| ![Kavat speaking](docs/screenshots/agent-voice.png) **Kavat speaking.** An island under the bar: its whiskers move with its voice, and captions follow word by word. | ![Displays](docs/screenshots/settings-displays.png) **Displays.** One, two side by side, or landscape next to portrait: placed by dragging, remembered for each set of displays. |
| ![System monitor](docs/screenshots/system-monitor.png) **System monitor.** A fork of btop: processes grouped by what they belong to, and Kavat explains any of them. | ![Terminal settings](docs/screenshots/settings-terminal.png) **Terminal settings.** Kitty, plus the system info and the monitor. |
| ![Services](docs/screenshots/system-monitor-services.png) **Services.** What failed, its log, and Kavat explaining the error. | ![History](docs/screenshots/system-monitor-history.png) **History.** The last 24 hours, to ask what happened at three. |
| ![Network settings](docs/screenshots/settings-network.png) **Network and Bluetooth.** Connections, VPNs and devices on one page. | ![Password dialog](docs/screenshots/password-dialog.png) **Password dialog.** Drawn by the shell: who asks, and what for. |
| ![Notices](docs/screenshots/notices.png) **Notices.** A process running hot, a disk just plugged in. | ![Calculator](docs/screenshots/launcher-calc.png) **Calculator.** Sums and unit conversions in the palette. |
| ![Updates](docs/screenshots/kav-updates.png) **Updates.** Packages and firmware in one pass. | ![Shell settings](docs/screenshots/settings-shell.png) **Shell settings.** Prompt, plugins and the default apps. |
| ![Software step](docs/screenshots/installer-software.png) **Choose your software.** Tick programs by category while installing; `kav` installs them. | ![Software step, offline](docs/screenshots/installer-software-offline.png) **No network.** The base system installs as it comes and the step is skipped. |
| ![Icons and cursor](docs/screenshots/settings-icons.png) **Icons and cursor.** Made from the theme: folders and the cursor take the accent. | ![Tinted icons](docs/screenshots/settings-icons-tinted.png) **Tinted icons.** An optional style that paints every icon with the palette. |

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

**Kavat**, the agent, runs on your machine. No cloud, no account, no telemetry. It is a compact model on Ollama, tuned for this system and only this system: it knows your shortcuts, your packages, your configuration and the OS documentation. You talk to it in English. Its job is running the system, not writing code: that is what Claude in the editor is for. The model can be swapped for another from the settings panel.

- **It fits next to a game.** The agent takes at most 4 GB of the GPU's memory. It stays loaded while you work, so it answers at once, and leaves memory in Gamer mode. Its voice runs on the CPU.
- **Small model, made reliable.** It starts on Qwen3.5 4B. For each request it sees only the five to eight tools that fit, a skill with several steps is a single call, and an exact phrase never reaches the model.
- **It has a voice.** It listens and speaks through VoiceStudio, also locally.
- **You can see it speak.** While it talks, an island drops from the bar with Kavat's face. Its whiskers are the sound of its voice, and the words appear as captions, lit one by one. The same island shows what it heard you say and which tool it is calling, and the keyboard and case lights pulse along.
- **It has no shell.** It talks to `kavatd`, a daemon of typed tools with four permission levels. Reading is free. Changing a theme is free and logged. Installing something asks first and takes a snapshot. Touching disks, users or the network asks for your password.
- **It cannot grant itself permissions**, and everything it does can be undone.
- **What it reads is never an order.** A web page, a file or a command's output is only information. Only your messages, your clicks and the skills you launch can make it act.
- **It speaks up only when it matters.** Push to talk by default, a key in the terminal that explains the last error, and quiet notices that arrive with the skill that fixes them.

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

Gamer turns on by itself when a game starts from Steam and off when it closes. Each game can carry its own profile: frame cap, overlay and fan curve. Animations stay: the mode frees resources by stopping processes, not by stripping the look. A mode of your own is one more TOML file.

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

Updates do not leave you halfway. Changes that pacman does not manage run as one-time migrations after each update; Arch's manual interventions become guided steps instead of a notice to go and read; and a health check remembers the last good snapshot and goes back to it when an update breaks the system.

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

Icons and the cursor are made from the same theme: folders, system icons and the cursor's edge take the accent, and two optional icon styles paint everything with the palette or draw it in plain lines. Light mode is designed, not inverted. Profiles have names (`work`, `night`, `presenting`) and can switch on their own by time of day or by the monitor you plug in. A third-party theme is a single TOML file.

**The wallpaper can move.** An image is the default; a looping video is one more option in the same grid. The video pauses by itself when you cannot see it, when something is fullscreen and in Gamer mode, and the login screen uses a still frame.

## Everything is set in one place

A settings panel inside the shell configures the whole core, on twenty-six pages: from appearance, windows and keybindings to GPU, sound, notes, projects, backups, privacy and boot.

- **One place per setting.** Each setting lives in its app's own config file, or in `~/.config/kavat/` for the system's own.
- **Live and reversible.** Every change shows instantly, has undo and lands in a history.
- **The config file is the truth, and it is a two-way street.** Edit a value by hand and the panel shows it at once; change it in the panel and the panel rewrites that one line in your file, leaving the rest and your comments alone. There is no second copy to drift.
- **Open the config from anywhere.** Every settings page and every piece of the system (the agent's panel, the theme picker, `kav`, Yazi, the browser, the plugin marketplace) has a button that opens its config file in Neovim.
- **Nothing is left half-installed.** Pick an option that needs a program, such as mpvpaper for video wallpapers, and `kav` installs it before the setting applies. It then refuses to remove it by accident, and `kav doctor` checks it is still there.
- **The agent uses the same thing.** "Raise the gaps to 10" changes the same value the panel does.
- **Reinstalling is restoring.** Your whole state fits in one file.

## The desktop

- **Hyprland**, with modular configuration: the system's in `/usr/share/kavat/`, yours in `~/.config/`, and yours always wins. Its configuration is Lua, and the settings panel only ever writes plain values for that Lua to load.
- **Noctalia**, in its own fork: bar, launcher, notifications, control centre, plus the agent's panel, the mode selector and the settings panel.
- **LightDM + web-greeter**, with a login screen that uses your theme and your wallpaper. Entering the desktop has no visual cut.
- **Kitty and zsh**, with a starship prompt cut down to what costs no time: opening a terminal goes unnoticed.
- **One command palette.** `Super+D` searches apps, files, settings, skills, packages, windows and browser tabs, works out sums and unit conversions, with the same commands as the browser console and Neovim.
- **Projects.** A project saves a whole workspace (folder, terminals, editor, browser profile, window layout) and brings it back as you left it.
- **System info with its own face.** `kavat fetch` is a fork of fastfetch: the Kavat cat as its logo, the lines you choose, the colours of your theme. In Kitty the logo is an image, and a health line shows pending updates and `kav doctor` notices.
- **A monitor that knows the system.** `kavat top` is a fork of btop: processes grouped by what they belong to, the agent's share of the GPU in sight, and `K` asks Kavat what a process is and why it is busy. It also lists failed services with their log, keeps 24 hours of history, and warns when a process runs away.
- **One `kavat` command.** `kavat theme`, `mode`, `wallpaper`, `project`, `ask`, `fetch`, `top`, `doctor`: everything the panel does, from the terminal.
- **The shortcuts you already know**, inherited from Willwitcher-OS Endgame, plus `Super+Space` for Kavat.

## The editor

Neovim is part of the core, and it is useful the moment you open it.

- **Vim modes, VS Code navigation.** `Ctrl+P`, `Ctrl+Shift+P`, `Ctrl+S`, `Ctrl+/`, `Ctrl+D`, arrows and `Shift` selection work in any mode. `Esc` is still there.
- **Preinstalled.** File explorer, fuzzy finder, command palette, tabs, LSP with completion, format on save, git, floating terminal and multicursor.
- **A marketplace inside the editor.** `Ctrl+Shift+X` opens a catalogue of plugins by category. Verified ones bring their tested configuration and shortcuts, pinned to a commit; installing is one step.
- **Language packs.** "Rust" installs the LSP server, formatter, linter and debugger in one go.
- **Claude beside the code.** `Ctrl+Alt+B` opens Claude Code in a side panel that is a real Kitty split, connected to the editor: it sees your file and selection, and its edits open as native diffs you accept or reject. The same panel system hosts Kavat, a terminal or Lazygit. Claude Code is a cloud service that uses your own account; it is separate from Kavat, which stays local.

## Files

Yazi is the file manager, and the file explorer for the whole system.

- **It follows the theme**, like everything else.
- **Drag and drop.** `Alt+D` lifts your selection out so you can drag it into any app, and whatever you drop there is copied into the current folder.
- **Every app picks files with it.** "Open" and "Save as" open Yazi instead of a foreign dialog, in the browser too.
- **Downloads land in it.** Click a download in the browser and Yazi opens in the downloads folder with that file selected.
- **No Vim keys required.** Arrows, `Enter`, `Backspace`, `Ctrl+C` / `Ctrl+V`, `Del` and `F2` work; the Vim ones are still there.
- **Copying you can see.** Pasting opens a pop-up with per-file and total progress, and asks what to do when a name already exists.
- **Previews in the terminal.** Images, video thumbnails, PDFs, archives and code.

## Notes

Notes are part of the system, built on a fork of [`nb`](https://github.com/xwmx/nb). A note is a Markdown file in `~/Notes`, every notebook is a git repository, and the command is still `nb`.

- **To-dos with dates.** Due dates the system understands, with Today, Upcoming and Overdue views and a count in the bar.
- **Reminders.** A date and time on any note. The reminder arrives as a notification you can open, snooze or mark as done, and it waits while you play.
- **Links to folders.** A link to a folder opens it in Yazi, and a link to `file:line` opens Neovim on that line. From Yazi, one key opens the note of the folder you are in.
- **Kavat takes notes.** "Note that…", "remind me on Friday…", "what did I write about the TPM?". What it writes is marked as its own, every change is a commit you can undo, and private notebooks stay closed to it.
- **Quick note.** `Super+N` from anywhere: one line, read for tags and dates as you type, saved to the inbox.
- **Rendered in Neovim.** A note opens with its headings, checkboxes, tables, callouts and captures drawn in place. The line under the cursor stays raw Markdown, so you edit where you read.
- **Indexed.** Titles, tags, links and dates are indexed as you save, so the palette, the to-do views and the agent all search the same thing, by words or by meaning.
- **And more.** A morning summary, reminders tied to a situation ("when I open the kav project"), system notices turned into to-dos with one click, the last terminal command saved with one key, templates, and a note pinned to the desktop.

## The browser

A Chromium of its own, with seven changes.

- **It follows the modes.** In Incognito it opens private windows; in Gamer it suspends tabs.
- **It follows the theme.** It changes colour with the rest of the system, internal pages included.
- **It blocks out of the box.** Ads and trackers, from the first start.
- **It has Kavat beside it.** A panel that reads the page and acts in the browser, with the same permissions as in the system.
- **It has a console.** A palette of its own commands: `hide-tabs`, `hide-sidebar`, `zen`, and the ones you define.
- **It uses Yazi.** Uploading, "Save as" and clicking a download all go through the system's file manager.
- **It clips into your notes.** A page goes into a note with your selection as a quote and a capture. Favourite sites gather in albums with thumbnails, your open tabs become an album in one step, and a panel shows the notes you already have about the site you are on.

## Built for a gaming desktop

- **A GPU page.** Fan curve, power limit and sensors in the bar for the Radeon RX 6800.
- **A game layer.** MangoHud draws the per-game overlay and frame cap, and gamescope sets each game's resolution and scaling.
- **Network and Bluetooth on one page.** Wired, Wi-Fi, saved VPNs, and controllers and headphones with their battery.
- **Disks that mount themselves.** Plug in a USB disk and a notice offers to open it in Yazi or eject it.
- **Any number of displays.** Place them by dragging in the settings panel, landscape or portrait. Each set of displays keeps its own layout, so one synced configuration fits a PC with one screen and another with two.
- **Lights follow the theme.** Keyboard and case take the accent colour, change with the mode and pulse with Kavat's voice.
- **A clean microphone.** Noise suppression is on out of the box, for the agent's voice and for streams.
- **Easier on the eyes.** Reduce motion, text scale and high contrast, on one page.

## Unbreakable by design

- **Btrfs + snapshots** before every package transaction.
- **Bootable snapshots**: the latest system states are in the boot menu.
- **Fallback kernel**: `linux-zen` for every day, `linux-lts` for when something breaks.
- **LUKS2 encryption** with TPM2 unlock, **Secure Boot** with its own keys, firewall closed by default.
- **`kav update` reads the Arch news** and stops if an update needs manual intervention.
- **Config is checked before it is applied.** A bad edit rolls back to the last good version and tells you which line failed.
- **`kav doctor`** tells you what is wrong before you notice.
- **A health check after every update**, with an offer to roll back if it fails.
- **Your files too.** Hourly snapshots of `/home`, browsable from Yazi, and an encrypted copy to an external disk. The Steam library has its own subvolume and stays out of the snapshots.
- **Configuration history.** Every change, from the panel or by hand, is a commit; the history can sync to a private repository.
- **Secrets kept by the system.** A keyring that opens when you log in holds browser passwords and git tokens, and an agent holds your SSH and GPG keys.
- **A password dialog of its own.** When something needs your password, the shell draws the dialog and says who is asking and what for.
- **Firmware too.** `kav update` offers firmware for the board, the SSD and peripherals as one more step.
- **Privacy in plain sight.** The bar shows when an app uses the microphone, camera or screen, and a per-app firewall asks the first time an app reaches for the network. Games launched by Steam use Steam's rule.

## From power button to desktop

Three screens, one identity: the boot menu, the loading screen and the login.

- **A boot menu of its own.** GRUB with the system's theme and your wallpaper: normal start, fallback kernel, safe mode, bootable snapshots, recovery and firmware.
- **Safe mode.** Starts with the factory configuration and ignores your config files without deleting them. If a boot fails, the menu opens by itself with safe mode already selected.
- **A loading screen of its own.** Plymouth in four styles (logo, spinner, bar or boot messages) that follows the theme. The disk password prompt lives there too.
- **Set from the panel.** Timeout, default entry, which entries show, look and loading style, all on the Boot page.

## Installation

A Calamares installer with its own identity. Seven screens, about ten minutes, and the base system installs with no network. English by default, with Spanish already installed as the second option.

1. Language and keyboard
2. Disk (encryption is one switch)
3. User
4. Detected hardware and suggested drivers
5. Software: tick the programs you want, by category, or search for any package; `kav` installs them. This step needs a network
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
│ Base                zen + lts · Btrfs · GRUB · LUKS2               │
└────────────────────────────────────────────────────────────────────┘
```

## Documentation

| What | Where |
| --- | --- |
| Design document: every piece, every decision and what is still to be verified (in Spanish) | [`docs/DESIGN.md`](docs/DESIGN.md) |
| Pending decisions: what is still waiting for an answer (in Spanish) | [`docs/PENDING.md`](docs/PENDING.md) |
| A two-minute tour, narrated by Kavat | [`docs/video/kavat-tour.mp4`](docs/video/kavat-tour.mp4) |
| Interactive mockup of the interface | [`docs/mockup/index.html`](docs/mockup/index.html) |
| Screenshots of the mockup | [`docs/screenshots/`](docs/screenshots/) |
| The cat's ASCII art for the terminal | [`docs/assets/kavat-ascii.txt`](docs/assets/kavat-ascii.txt) |
| Research: what Omarchy and other distributions do, and which ideas are worth taking (in Spanish) | [`reports/Ideas de otras distros Linux.md`](reports/Ideas%20de%20otras%20distros%20Linux.md) |
| Research: which local model should be the agent, how to split the 4 GB with voice, and the test that confirms it (in Spanish) | [`reports/Modelo local para el agente Kavat.md`](reports/Modelo%20local%20para%20el%20agente%20Kavat.md) |

## The repository

Today the repository holds this README, the `docs/` folder, the research reports in `reports/` and three maintenance scripts in `tools/`, which check the documents' links, retake the screenshots and render the tour video. The rest arrives with each phase:

| Folder | Contents | Phase |
| --- | --- | --- |
| `iso/` | archiso profile, package list, live system overlay | 1 |
| `packages/` | One PKGBUILD per `kavat-*` package, forks included | 1 |
| `installer/` | Calamares configuration, modules and branding | 2 |
| `themes/` | Tokens, templates and bundled themes | 3 |
| `greeter/` | web-greeter theme | 3 |
| `kav/` | The package manager, its TUI and the AUR allowlist | 3 |
| `notes/` | `kavat-notes`: the `nb` fork, its TUI, the index and the reminders | 3 |
| `agent/` | `kavatd`, tools, skills and the agent's Modelfile | 4 |
| `tests/` | Boots and installs every ISO build in a virtual machine | 1 |

## Principles

1. **Arch underneath, no magic.** Everything is a package. What the system puts there, pacman can remove.
2. **Curated, not open.** Fewer options, better chosen.
3. **A single source of truth** for appearance and settings.
4. **Always reversible.** Breaking it costs a reboot.
5. **The agent is part of the system**, not an app. And it is yours.
6. **Config files are the truth.** The settings panel is one more editor of them, not a second copy.

## Why "Kavat"

A kavat is a companion feline: fast, quiet, and always beside whoever raised it. That is what this edition wants to be.

---

## Real project status

This README describes the finished system. Today the project is in **phase 0, design**: none of the above is implemented yet. What exists is the [design document](docs/DESIGN.md), with the decisions made so far, and the [mockup](docs/mockup/index.html) of the interface.

| Phase | Deliverable | Status |
| --- | --- | --- |
| 0 | Design documented | Done |
| 1 | ISO that boots to Hyprland + Noctalia | Pending |
| 2 | Calamares installer with Btrfs and snapshots | Pending |
| 3 | Themes, settings, greeter, shell fork, `kav`, editor and notes | Pending |
| 4 | Local agent, skills and voice | Pending |

A study of Omarchy and other distributions reopened eight of those decisions. All are now answered. The current Noctalia was read against what this shell needs: plugins could host almost all of it, but not the settings panel, so the shell is a fork of Noctalia 5 and from that point on it is this system's own application. They are listed, with their status, in the design document under «Decisiones en revisión».

The design also takes several assumptions for granted that have not been tested. They are listed, with the phase in which each gets verified, at the end of the [design document](docs/DESIGN.md#decisiones).
