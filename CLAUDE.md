# Willwitcher-OS Kavat Edition

A personal distribution based on Arch Linux. Phases 1 and 2 are closed: there is an archiso profile in `iso/`, `kavat-*` packages in `packages/` and an installer in `installer/`, and the installed system boots, takes snapshots and goes back to them, on the desktop PC too. It is in **phase 3, the identity**: the theme system, `kavat-theme`, the system's own boot menu, loading screen and login screen, and `kav` on the command line already exist; Hyprland is unmodified, the shell is Kavat Shell, the system's own program, whose command is `kavat-shell`: it began as Noctalia 5.4.1 and is built from it with two patches and a renaming, the terminal opens zsh with the system's prompt, `kavat-find` is the file search, `kavat-nvim` sets Neovim up `kavat-files` sets Yazi up as the file manager and the file window of every app, the folders' icons and the cursor take the theme's colours, `kavat-notes` brings the notes, on a fork of `nb`, with reminders, and `kavat-settings` is the first pages of the settings panel, as a plugin of the shell; the rest of the design is not implemented. It is built on the desktop PC. That PC runs CachyOS, installed only for this project: the ISO and the packages are built inside clean Arch chroots, with a `pacman.conf` of their own that only has Arch's repositories and `[kavat]`. The host's `pacman` and `makepkg` are never used for what ends up in the system: CachyOS's `pacman` leaves marks of its own in what it installs.

## Languages

- The user is spoken to in Spanish.
- Everything written in the repository and in the system is in English: code, comments, messages, interface texts, commits and documents. The system carries no texts of its own in another language. The user ruled this on 13 November 2026.
- The installer offers English by default and Spanish as the second option: the user does install it.
- Keyboards: `us` by default and `es`. No others.

## Where the design lives

The whole design lives in the repository, so that it reaches any machine the same. It is three places, and the first two must say the same:

1. **`docs/DESIGN.md`.** The design document. It is the source and is edited directly; editing it brings the date in its header up to date.
2. **`docs/mockup/index.html`.** The interactive mockup of the interface, in a single HTML file. When the user asks for a "proposed design" they want to see it drawn there, not described in text.
3. **`docs/PENDING.md`.** The "Pending decisions" document: what waits for a decision of the user's.

`docs/MANUAL-TESTS.md` is the list of what the user tries by hand on the PC at the end of a phase: what the tests in the virtual machine cannot see. A piece that is built adds its lines there.

The user calls both documents and the mockup "artifact". Until 11 November 2026 the two documents lived in claude.ai and `docs/DESIGN.md` was an export; those copies in claude.ai are no longer used or updated.

The two diagrams of the design document are images in `docs/img/`, each made from the HTML file of the same name beside it: that file is edited and captured with headless Chromium at scale 2, with `--system-font-family=Inter`. The roadmap is 672×466 and the architecture 672×412.

## How the design changes

1. What the user has to decide goes to `docs/PENDING.md`, not to the design document. Each point has a number (D1 to D43 are used; the next is D44), a recommendation and what happens if the answer is no.
2. Before working, that file is read: the user answers in the chat or by writing in it.
3. What is approved goes to `docs/DESIGN.md`, to the mockup and, when it applies, to the README, and leaves the pending list. The document and the mockup change in the same commit: neither is left for later.
4. What is rejected is deleted from the pending list and is not asked again. If the user approves one part and says "not the rest", that no covers everything else that was on the table.
5. After touching the mockup, the screenshots it affects are retaken with `tools/screenshots.sh <name>`.
6. A commit goes on `main` after each change to the design, without asking. Pushing is not included: it is done when the user asks.

## The user's criteria

- Do not complicate: add no pieces and no questions that were not asked for.
- The size of the ISO or of the installed system is not a cost to weigh: it is the user's own system.
- Nothing is taken from Omarchy's shortcuts or defaults. Its mechanisms only come in one by one and with approval.
- The Hyprland shortcuts are those of the user's earlier system, Willwitcher-OS Endgame. New ones only take keys that were free.
- The local agent is spoken to in English. For programming, the user uses Claude and not the local agent.

## Tools

- `tools/check-links.py`: checks that the links in the README, in `docs/DESIGN.md`, in `docs/PENDING.md` and in `docs/MANUAL-TESTS.md` point to files that exist. It is run after editing any of them.
- `tools/make-video.cjs [seconds]`: remakes `docs/video/kavat-tour.mp4`, the tour narrated by Kavat, from `docs/video/index.html`, which uses the mockup as its picture. The voice is Supertonic-3, the model the design gives Kavat. It needs `puppeteer-core`, `ffmpeg-static` and `supertonic` installed outside the repository; how to use it is in the script's header. The video presents the system as finished, by the user's decision; the README still says which phase it is really in.
- `tools/screenshots.sh [name ...]`: retakes the screenshots in `docs/screenshots/` from the mockup, with headless Chrome or Chromium. On Linux it needs the fonts Inter and JetBrains Mono installed.

## Building and testing

After touching `iso/`, `packages/` or `installer/` these are run, in this order. The first two need `devtools` and sudo; the tests need no root. The PC suspends by itself when nobody touches it, so anything that takes a while is started with `systemd-inhibit --what=sleep:idle`.

- `tools/build-repo.sh [folder ...]`: builds, in a clean Arch chroot, the packages in `packages/` and those of the pieces that carry their own PKGBUILD, like `installer/`, and gathers them in `out/repo/`, the local `[kavat]` repository. Each one is built after those of the repository it depends on. With names, only those: `tools/build-repo.sh installer`.
- `tools/build-iso.sh`: builds the ISO from `iso/`, with `mkarchiso` inside another clean Arch chroot, and leaves it in `out/`.
- `tests/boot.py [iso]`: boots the ISO in QEMU, checks from inside that it reaches the desktop and leaves a picture in `out/test/desktop.png`. It is the gate of phase 1 and has to keep passing. Needs `qemu-desktop` and `edk2-ovmf`.
- `tests/install.py [--encrypt] [iso]`: installs the ISO on a virtual disk, driving the installer with the keyboard, boots from that disk and logs in. With `--encrypt`, the machine has a TPM2 simulated with `swtpm` and the disk is encrypted: it boots without asking for a password and, without the TPM2, with the recovery one typed at GRUB; everything goes to `out/test/encrypted/`. It takes about six minutes and leaves pictures in `out/test/`.
- `tests/snapshots.py [--encrypted]`: on the disk the previous test leaves, or the encrypted one with `--encrypted`, changes a package, boots the snapshot from before and goes back to it with `kavat-rollback`.
- `tests/safemode.py [--encrypted]`: on the same disk, breaks the user's Hyprland configuration on purpose and checks that the next boot opens in safe mode and that the desktop does come up there.
- `tests/theme.py [--encrypted]`: on the same disk, logs in, changes the mode, the palette and the accent with `kavat-theme` and checks that Hyprland, Kitty and the shell follow the change in under a second and without restarting. It is half of the gate of phase 3.
- `tests/startup.py [--encrypted]`: on the same disk, watches the screen while it boots and checks that the GRUB menu, the loading screen and the login screen wear the theme, that the menu has the system's entries, that `kavat-theme boot` changes their look and that the Recovery entry ends in `kavat-rescue`.
- `tests/kav.py [--encrypted]`: on the same disk, asks `kav` for packages on and off the reviewed list, with a folder standing in for the AUR, and undoes a change with `kav undo`. It is the other half of the gate of phase 3. It uses the lists of packages that `tools/build-repo.sh` leaves in its chroot.
- `tests/find.py [--encrypted]`: on the same disk, checks that the installer left the index of the whole disk built and that a change of packages brings it up to date, then logs in, opens the file search with `Super+Shift+F` and finds a file by name, in the index and by a word inside it.
- `tests/editor.py [--encrypted]`: on the same disk, loads every plugin of `kavat-nvim` with no network, checks treesitter, Marksman and the shortcuts, changes the theme under an open editor, and types into a file on the desktop and saves it.
- `tests/files.py [--encrypted]`: on the same disk, opens Yazi with `Super+E`, copies a file onto its own name, answers the copy window and trashes a file with the system's keys, asks the desktop for a file as an app does and picks it in Yazi, and asks for a file to be shown in its folder.
- `tests/notes.py [--encrypted]`: on the same disk, saves notes with `kavat-notes`, checks that a day and a time become a reminder with its timer, waits for a reminder to come up as a notification, and types a quick note with `Super+N`.
- `tests/settings.py [--encrypted]`: on the same disk, checks that the shell has the settings plugin on, opens the panel with `Super+T`, presses Light and a palette in it, and checks that the theme of the system changed; changes settings of Kitty, Yazi, the keyboard and the lock with `kavat-conf`; opens every page that is built; and locks the screen and opens it with the password.

What the tests share is in `tests/machine.py`. The last ten work on a layer over the installed disk, so they do not change it.

`kavat-theme` is in `themes/` and is Rust: its unit tests run with `cargo test` inside that folder, and `tools/build-repo.sh themes` runs them again when packaging. Its code is in `themes/engine/` and not in `src/`, which is the folder `makepkg` uses next to a PKGBUILD. The desktop's templates are in `themes/templates/` and those of the boot and the login in `themes/boot/`; these are applied as root with `kavat-theme boot`. Two templates come from packages of their own, which make them when they are built: the icons, from Papirus, in `packages/kavat-icons/`, and the cursor, from Bibata, in `packages/kavat-cursor/`.

`kav` is in `kav/` and is Rust too, with its code in `kav/core/`; `tui.rs` there is the full screen that `kav` opens with no arguments in a terminal; its unit tests run with `cargo test` inside that folder. `kav install` is not tried on the build PC: CachyOS's repositories have packages there, like `yay`, that do not exist in Kavat.

`kavat-fetch` and `kavat-top` are in `packages/kavat-shell-config/`: two small scripts over fastfetch and btop, whose looks are the templates `themes/templates/fastfetch/` and `themes/templates/btop/`. The mode by the hour is `themes/kavat-theme-auto`, installed as `/usr/lib/kavat/theme-auto`. `kavat-files` is in `packages/kavat-files/`, with its Yazi plugins in `plugins/`; Yazi's own source documents what a plugin can call.

`kavat-find`, the file search, is in `packages/kavat-find/`: one bash script that drives `fzf`, which calls the same script back for its lists, its preview and what a key does. `fzf-tab`, which Arch's repositories do not have, is built in `packages/fzf-tab/` from its release.

`kavat-nvim` is in `editor/`: `lua/kavat/` is the configuration, `colors/kavat.lua` the colour scheme made from the theme's palette, and the PKGBUILD lists each plugin with its commit and compiles the treesitter parsers named in `parsers.txt`. `work/nvim/run.sh`, when it exists, opens it on the build PC apart from the host's own Neovim.

`kavat-notes` is in `packages/kavat-notes/`: `kavat.patch` is the whole fork of `nb`, and `kavat-notes` the script with the quick note, the reminders and what is due. It can be tried on the build PC with `NB_DIR` and `HOME` pointed at a temporary folder; the timers it makes there are the build PC's, so `kavat-notes sync` is run after deleting that folder.

`kavat-settings` is in `packages/kavat-settings/`: `settings/` is a plugin of the shell, with `plugin.toml` and `panel.luau`, which is Luau, and `kavat-conf` is the program that reads and changes the settings each app keeps in its own file; the functions a plugin calls are still named `noctalia.`. What a plugin can do is in Noctalia's own documents, inside its source, under `docs/user/plugins/development/`: they are read before using a control or a function that is not already in the script.

Kavat Shell is in `packages/kavat-shell/`: Noctalia's source, `no-ping.patch`, `no-network.patch` and `rename.py`, which changes the name in the source before it is built. A patch is made against the source with the patches before it applied, and before the renaming. Its build takes about a quarter of an hour.

The installer's look is in `installer/branding/kavat/`: `stylesheet.qss` for Calamares' own pages, `calamares-sidebar.qml` for the steps, `appearance.qml` for the Appearance page and `show.qml` for the pictures shown while it installs, which are copies of some of `docs/screenshots/` in `slides/`: when one of those screenshots is retaken, its copy is too. `work/vm/live.py`, when it exists, keeps a machine running from a copy of the ISO, to try the installer's files in without building.

`kavat-greeter`, the login screen, is in `greeter/`: `greeter.py` is its program, which greetd starts inside a small Hyprland session (`hyprland.lua`), and `page/` is the web page it shows. That page can be opened in a browser, where it stands in for the program itself with a made-up user whose password is `kavat`.

Arch's repositories are used as they were on the day `iso/arch-snapshot` says; changing that date is updating the system. `out/`, `work/` and `vendor/` do not go into git: they are results, temporary work and downloads.

Hyprland's configuration is Lua and its API changes: before using a new function, it is checked in the virtual machine, not assumed.
