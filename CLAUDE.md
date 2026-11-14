# Willwitcher-OS Kavat Edition

A personal distribution based on Arch Linux. Phases 1 and 2 are closed: there is an archiso profile in `iso/`, `kavat-*` packages in `packages/` and an installer in `installer/`, and the installed system boots, takes snapshots and goes back to them, on the desktop PC too. It is in **phase 3, the identity**: the theme system, `kavat-theme`, the system's own boot menu, loading screen and login screen, and `kav` on the command line already exist; Hyprland is unmodified, the shell is `kavat-shell`, Noctalia with a patch that takes out its ping, and the terminal opens zsh with the system's prompt; the rest of the design is not implemented. It is built on the desktop PC. That PC runs CachyOS, installed only for this project: the ISO and the packages are built inside clean Arch chroots, with a `pacman.conf` of their own that only has Arch's repositories and `[kavat]`. The host's `pacman` and `makepkg` are never used for what ends up in the system: CachyOS's `pacman` leaves marks of its own in what it installs.

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

The user calls both documents and the mockup "artifact". Until 11 November 2026 the two documents lived in claude.ai and `docs/DESIGN.md` was an export; those copies in claude.ai are no longer used or updated.

The two diagrams of the design document are images in `docs/img/`, each made from the HTML file of the same name beside it: that file is edited and captured with headless Chromium at scale 2, with `--system-font-family=Inter`. The roadmap is 672×466 and the architecture 672×412.

## How the design changes

1. What the user has to decide goes to `docs/PENDING.md`, not to the design document. Each point has a number (D1 to D34 are used; the next is D35), a recommendation and what happens if the answer is no.
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

- `tools/check-links.py`: checks that the links in the README, in `docs/DESIGN.md` and in `docs/PENDING.md` point to files that exist. It is run after editing any of the three.
- `tools/make-video.cjs [seconds]`: remakes `docs/video/kavat-tour.mp4`, the tour narrated by Kavat, from `docs/video/index.html`, which uses the mockup as its picture. The voice is Supertonic-3, the model the design gives Kavat. It needs `puppeteer-core`, `ffmpeg-static` and `supertonic` installed outside the repository; how to use it is in the script's header. The video presents the system as finished, by the user's decision; the README still says which phase it is really in.
- `tools/screenshots.sh [name ...]`: retakes the screenshots in `docs/screenshots/` from the mockup, with headless Chrome or Chromium. On Linux it needs the fonts Inter and JetBrains Mono installed.

## Building and testing

After touching `iso/`, `packages/` or `installer/` these are run, in this order. The first two need `devtools` and sudo; the tests need no root. The PC suspends by itself when nobody touches it, so anything that takes a while is started with `systemd-inhibit --what=sleep:idle`.

- `tools/build-repo.sh [folder ...]`: builds, in a clean Arch chroot, the packages in `packages/` and those of the pieces that carry their own PKGBUILD, like `installer/`, and gathers them in `out/repo/`, the local `[kavat]` repository. With names, only those: `tools/build-repo.sh installer`.
- `tools/build-iso.sh`: builds the ISO from `iso/`, with `mkarchiso` inside another clean Arch chroot, and leaves it in `out/`.
- `tests/boot.py [iso]`: boots the ISO in QEMU, checks from inside that it reaches the desktop and leaves a picture in `out/test/desktop.png`. It is the gate of phase 1 and has to keep passing. Needs `qemu-desktop` and `edk2-ovmf`.
- `tests/install.py [--encrypt] [iso]`: installs the ISO on a virtual disk, driving the installer with the keyboard, boots from that disk and logs in. With `--encrypt`, the machine has a TPM2 simulated with `swtpm` and the disk is encrypted: it boots without asking for a password and, without the TPM2, with the recovery one typed at GRUB; everything goes to `out/test/encrypted/`. It takes about six minutes and leaves pictures in `out/test/`.
- `tests/snapshots.py [--encrypted]`: on the disk the previous test leaves, or the encrypted one with `--encrypted`, changes a package, boots the snapshot from before and goes back to it with `kavat-rollback`.
- `tests/safemode.py [--encrypted]`: on the same disk, breaks the user's Hyprland configuration on purpose and checks that the next boot opens in safe mode and that the desktop does come up there.
- `tests/theme.py [--encrypted]`: on the same disk, logs in, changes the mode, the palette and the accent with `kavat-theme` and checks that Hyprland, Kitty and Noctalia follow the change in under a second and without restarting. It is half of the gate of phase 3.
- `tests/startup.py [--encrypted]`: on the same disk, watches the screen while it boots and checks that the GRUB menu, the loading screen and the login screen wear the theme, that the menu has the system's entries, that `kavat-theme boot` changes their look and that the Recovery entry ends in `kavat-rescue`.
- `tests/kav.py [--encrypted]`: on the same disk, asks `kav` for packages on and off the reviewed list, with a folder standing in for the AUR, and undoes a change with `kav undo`. It is the other half of the gate of phase 3. It uses the lists of packages that `tools/build-repo.sh` leaves in its chroot.

What the tests share is in `tests/machine.py`. The last five work on a layer over the installed disk, so they do not change it.

`kavat-theme` is in `themes/` and is Rust: its unit tests run with `cargo test` inside that folder, and `tools/build-repo.sh themes` runs them again when packaging. Its code is in `themes/engine/` and not in `src/`, which is the folder `makepkg` uses next to a PKGBUILD. The desktop's templates are in `themes/templates/` and those of the boot and the login in `themes/boot/`; these are applied as root with `kavat-theme boot`.

`kav` is in `kav/` and is Rust too, with its code in `kav/core/`; its unit tests run with `cargo test` inside that folder. `kav install` is not tried on the build PC: CachyOS's repositories have packages there, like `yay`, that do not exist in Kavat.

`kavat-greeter`, the login screen, is in `greeter/`: its PKGBUILD fetches web-greeter 4.0.0 by its commit, applies `system-python.patch` to it and gives it the page in `greeter/theme/`. That page can be opened in a browser, where it stands in for LightDM itself with a made-up user whose password is `kavat`. Everything in that folder is GPL-3, like web-greeter.

Arch's repositories are used as they were on the day `iso/arch-snapshot` says; changing that date is updating the system. `out/`, `work/` and `vendor/` do not go into git: they are results, temporary work and downloads.

Hyprland's configuration is Lua and its API changes: before using a new function, it is checked in the virtual machine, not assumed.
