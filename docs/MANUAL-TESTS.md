# Willwitcher-OS Kavat Edition — What to try by hand

Written on 14 November 2026 and brought up to date on 15 November, twice, for phase 3 · [Design document](DESIGN.md) · [Pending decisions](PENDING.md)

The automatic tests run in a virtual machine with one screen, no network and a keyboard that types through a remote console. This is the list of what they cannot see: the real graphics card, two monitors, the mouse, the network, and how things feel. It is meant to be gone through once, from top to bottom, after one installation.

Mark each line as you go. For anything that fails, the line says what to note down.

## Before starting

- Write the ISO in `out/` to the USB stick, or ask for it to be written.
- Install as before, on the 230.8 GiB Btrfs partition of the 465.8 GiB disk. **Do not touch the Windows disk** (476.9 GiB, NTFS).
- The first part needs no network. The last part does.

## 1. The installer, and from the boot menu to the desktop

- [ ] **The installer** is a dark window in the middle of the screen, with its steps down the left. Compare with `docs/screenshots/installer.png`.
- [ ] **Appearance.** Choose a palette that is not Kavat, Light, and another accent: the small desktop on the right follows. After installing, the boot menu, the loading screen, the login and the desktop wear what you chose.
- [ ] **Software.** With the network cable in, or Wi-Fi joined from the bar before opening the installer: check a few programs in two categories, and type `htop` in "Any other package". After installing, they are there (`pacman -Q firefox htop`), and `/etc/kavat/kavat-install.toml` says what was asked for and what came in. *This was only tried without a network, where it leaves everything out and says so: with a network it has never run. It makes the installation longer by what has to be downloaded.*
- [ ] **While it installs**, pictures of the system go by, each with its line.
- [ ] **The disk page**: the legend under the two bars can be read.

- [ ] **Boot menu.** The picture fills the screen, the first entry is "Willwitcher-OS Kavat Edition" and the countdown reads "Booting in N s".
- [ ] **Loading screen.** After Enter, the picture stays: logo and ring, no black square. *Checked on this PC on 14 November, before the login moved to Wayland; this is to see that nothing changed.*
- [ ] **Login, two monitors.** The form is on the main one and only the clock on the other.
- [ ] **Login keys.** A wrong password says so; the US/ES pill changes the keyboard; the power buttons work.
- [ ] **The moment after the login.** There is a second or two of console text before the desktop. You said it is fine; note it only if it got longer.

## 2. The look

- [ ] **`Super+T`** opens the settings panel on Appearance. It should look like the mockup: the pages on the left, palette cards, Mode, the accent dots, two columns, the file at the bottom.
- [ ] **Mode.** Press Light, then Dark: the bar, the terminal, the borders and the panel itself change at once.
- [ ] **Palette and accent.** Press a few palette cards and accent dots. Everything follows, including the panel.
- [ ] **Tone, contrast, OLED black, fonts, scale.** Each control changes something you can see.
- [ ] **Windows page.** Gaps, corner radius and border move the windows as the slider is let go; Blur, Transparency, Shadows and Animations switch.
- [ ] **The cursor.** Its body is light on a dark theme and dark on a light one, and its edge is the accent. It changes when the accent does. *Note any app where the cursor is a different one.*
- [ ] **The folders' icons.** Open the launcher (`Super+D`) or any list with folder icons: they are the accent's colour and change with it. *Nothing but the launcher shows icons yet, so this may be hard to see.*
- [ ] **Boot and login look.** On the Boot page, press "Give them the look in force" with a light theme: it asks for your password in a dialog of the shell. Restart: the boot menu, the loading screen and the login are light. Put it back to dark the same way.
- [ ] **The shell's name.** `kavat-shell --help` in a terminal answers; nothing on the screen says Noctalia. *Note any place that still does.*
- [ ] **Music.** Play something in an app: the bar's media widget shows it. A cover that comes from the network is not shown, on purpose; say if you want it back.
- [ ] **Icon style and cursor size.** Tinted paints the app icons in the launcher with the palette; Line draws the folders as outlines; 32 and 48 make the cursor bigger at once.
- [ ] **The background** is the same picture as the login's and changes colours with the theme. Choose one of your own in the shell's wallpaper picker, change the theme, and yours stays.
- [ ] **`Super+/`** opens the list of shortcuts. *Note any that is wrong or missing.*
- [ ] **Terminal page**: the cursor's shape and the margin change in the open terminals at once.
- [ ] **Editor and Files pages**: a change shows the next time Neovim or Yazi opens.
- [ ] **Lock**: `Super+Shift+L` shows the lock screen, like the login; a wrong password stays, yours opens. On Login & lock, set "Lock after" to 1 minute and wait. *On two monitors: say what the second one shows.*
- [ ] **Auto**, beside Light and Dark: the mode becomes the one the hour asks for, light from 7 to 19. Leave it on and look at the desktop after 19:00, or the next morning. Light or Dark turns it off.
- [ ] **Accessibility page**: Reduce motion stops the windows' animations; Text scale makes the text bigger; High contrast and the cursor's size as on Appearance.
- [ ] **Wallpaper page**: "Choose a picture" opens the shell's chooser; "Use the theme's picture" puts the theme's back.
- [ ] **Displays page** shows your two monitors with their real sizes and rates. *It only shows: nothing can be changed there yet.*
- [ ] **Sound, Network and Bar & widgets** each open the shell's own controls. *Wi-Fi and Bluetooth were never seen: the test machine has neither.*
- [ ] **Language & region**: choose Spanish first: the keyboard is Spanish at once. **`Alt+Shift`** changes between the two anywhere. *If you installed with the Spanish keyboard, it should already be the first.*
- [ ] **Pages not built** say what they wait for in one line. The greyed controls (profiles, Undo, History, Export) cannot be pressed.

What to compare with: `docs/screenshots/settings-icons.png` and `settings-windows.png`.

## 3. The file search

- [ ] **`Super+Shift+F`** opens a floating window with your files. Type to filter; the preview is on the right.
- [ ] **Enter** opens the file in a new window and the search closes. A text file opens in Neovim.
- [ ] **`Ctrl+F`** goes to the whole disk, then to text inside files, then back. The whole disk answers at once, on the first try after installing.
- [ ] **Text search**: type a word, Enter opens Neovim at that line.
- [ ] **`Ctrl+Y`** copies the path (paste it somewhere). **`Ctrl+O`** opens the folder in Yazi. *`Ctrl+O` was never tried.*
- [ ] **Tab** marks several, Enter opens them all.
- [ ] **An image or a PDF** under the cursor shows in the preview. *Never tried on real hardware.*

## 4. The terminal

- [ ] **Tab** after `cd ` or `ls ` opens the picker with a preview of each folder; `/` goes into the folder under the cursor.
- [ ] **`Ctrl+R`** searches the history, **`Ctrl+T`** puts a file's path in the command, **`Alt+C`** jumps to a folder, **`Ctrl+G`** searches text in the folder you are in. *None of the four was tried by hand.*
- [ ] **`kavat-find disk`** and **`kavat-find text`** open the search on that source, full screen.
- [ ] **`kavat-fetch`** shows the cat in the accent and the machine: check that the graphics card, the memory and the disk are yours. Change the accent and run it again.
- [ ] **`kavat-top`** opens btop in the theme's colours. *After a change of theme, it has the new colours the next time it is opened.*
- [ ] The terminal still opens instantly.

## 5. Neovim

Open a file of code in a project of yours with `nvim`.

- [ ] It opens ready to type; `Esc` goes to normal mode.
- [ ] **`Ctrl+S`** saves, **`Ctrl+Z`** / **`Ctrl+Y`** undo and redo, **`Ctrl+C`** / **`Ctrl+X`** / **`Ctrl+V`** copy, cut and paste with the desktop's clipboard (paste into another app and back).
- [ ] **Shift with the arrows** selects; typing replaces the selection.
- [ ] **`Ctrl+P`** finds a file, **`Ctrl+Shift+P`** opens the commands, **`Ctrl+Shift+F`** finds text in the project. *The two with Shift depend on Kitty telling them apart: this is the check the design asks for.*
- [ ] **`Ctrl+B`** opens the explorer, **`Ctrl+Tab`** changes tab, **`Ctrl+W`** closes the tab, **Ctrl + backtick** opens the terminal.
- [ ] **`Ctrl+/`** comments, **`Alt+Up/Down`** moves the line, **`Shift+Alt+Up/Down`** duplicates it, **`Ctrl+D`** adds a cursor at the next match, **`Ctrl+L`** selects the line.
- [ ] **`Ctrl+F`** finds, **`Ctrl+H`** replaces, **`Ctrl+G`** goes to a line.
- [ ] **Colours** are the theme's, and change when the theme does while the editor is open.
- [ ] **A Markdown file**: headings and checkboxes are drawn; `Ctrl+K V` switches to plain text; Enter on `- [ ]` checks it; Enter on `[[~/Documents]]` opens Yazi there.
- [ ] *Note every shortcut of yours from Endgame that you miss: only `<leader>ff`, `fg`, `fb` and `fr` were carried over.*

## 6. Files

- [ ] **`Super+E`** opens Yazi. Arrows move, Enter goes in or opens, Backspace goes up.
- [ ] **`Ctrl+C`**, **`Ctrl+X`**, **`Ctrl+V`**, **Delete** (asks, then the trash), **`F2`**, **`Ctrl+N`**, **`Ctrl+H`** for hidden files.
- [ ] **A name that is taken**: copy a file and paste it in a folder that has one of that name: a window says what is there and asks. Try `r` (replace), `s` (skip) and `k` (keep both), and `a` before answering with several files.
- [ ] **A big copy**: copy something of several gigabytes to another disk or to the USB stick: the window shows a bar for each item, the total, the speed and the time left, like `docs/screenshots/files-copy.png`. `Esc` hides it and a notice says when it is done; on another copy, `c` cancels and the half-copied file is gone. *On the same disk a copy is instant and the window never shows: that is Btrfs sharing the data, not a failure.*
- [ ] **Cut and paste** a folder onto a folder of the same name, answering `r`: what was there stays, what came is added, and the original is gone.
- [ ] **Copy to another program**: `Ctrl+C` on a file in Yazi, then paste in another app that takes files. *Never tried: nothing in the test machine takes a pasted file.*
- [ ] **Dragging**: drag a file from Yazi to another window with the mouse, and drop a file from elsewhere onto Yazi. *Never tried: it is Yazi's own, through Kitty. If it does not work, say so: ripdrag was dropped on its account.*
- [ ] **`C`** compresses the selection (type `name.zip`), Enter on an archive offers to extract.
- [ ] **`M`** lists the disks: plug a USB stick, `m` mounts it, Right goes into it, `u` unmounts and `e` ejects. *Never tried: the virtual machine has no disk to plug.*
- [ ] **Previews**: an image, a PDF, a video, an archive.
- [ ] **`e`** opens in the editor, **`T`** a terminal there, **`Ctrl+P`** the file search, **`g` `t`** the trash.

The window apps use to open and save files cannot be tried until there is an app that asks: the browser comes later.

## 7. Notes

- [ ] **`Super+N`**: a line appears. Type `try the notes #kavat` and Enter: a notification says it was saved.
- [ ] **A reminder**: `Super+N`, type `stand up in 2 minutes`, Enter. Two minutes later a notification comes up with Open, In 10 minutes, In 1 hour, Tomorrow and Done. **Press each button on different reminders.** *The buttons were never pressed: the test only sees the notification come up.*
- [ ] **`Super+Shift+N`** opens the notes on their own screen, like `docs/screenshots/notes.png`: `1` to `5` change tab, Enter opens the note in Neovim, `a` adds one, `Space` marks a task as done, `r` sets a reminder, `d` a day, `/` filters.
- [ ] **A reminder across a restart**: set one for five minutes ahead, switch off, come back after its time: it shows at login.
- [ ] **In a terminal**: `nb`, `nb due`, `nb remind 1 tomorrow 9:00`.

## 8. kav

- [ ] **`Super+U`** opens `kav` in a floating window. It should look like `docs/screenshots/kav-dashboard.png`: four boxes with numbers, Health, Recent activity, the keys at the bottom.
- [ ] **`1` to `7`** change tab; **`/`** then a name and Enter searches; Enter shows the record.
- [ ] **`5`**, Enter: asks for your password and lists the snapshots.
- [ ] **`7`**, Enter on "Packages downloaded at once": a window says what it will run; Enter, your password, and the number changed. The same for "Snapshots kept".

With the network:

- [ ] **`kav update`** fetches the lists and updates. *Never run: the test machine has no network.*
- [ ] **`kav install htop`**, then **`kav remove htop`**, then **`kav undo`** and restart.
- [ ] **An AUR package**: `kav request <name>`, `kav review <name>`, approve it, `kav install <name>`. *The build itself was never run: it needs the network. This is the most likely place for something to fail.*
- [ ] **`kav doctor`**, and on the Doctor tab, `f` on a line that failed. To have one: `sudo touch /var/lib/pacman/db.lck`.

## 9. Modes

- [ ] **The bar** has Coder, Gamer and Incognito, with Coder lit. Compare with the mockup's bar.
- [ ] **Gamer**: press it. A notice says so; notifications are silent (send one: `notify-send test`); `powerprofilesctl get` says performance. Press Coder: all of it is back. **`Super+Shift+M`** goes to the next mode.
- [ ] **Gamer by itself**: start a game from Steam, once Steam is installed: the bar goes to Gamer, and back when the game closes. *Never tried with a real game: the test uses a process that looks like one.*
- [ ] **Incognito without a VPN**: press it: a notice says it needs one, and the bar stays on Coder.
- [ ] **Incognito with your VPN**: import your Proton VPN or other file (`nmcli connection import type wireguard file <file>`, or `type openvpn`), choose it on the Modes page of the settings, and press Incognito. Check your address on a web page: it is the VPN's. Then bring the VPN down by hand (`nmcli connection down <name>`): **nothing should load** until you go back to Coder. *This is the most important check of this part: the test machine has no network, so the guard was only tried against a VPN that leads nowhere. Note exactly what happens.*
- [ ] **In Incognito**, type a command in a new terminal, leave the mode, and look for it with the up arrow in another new terminal: it is not there. What you copied in the mode cannot be pasted after leaving.
- [ ] **Modes page** (`Super+T`, Modes): the three cards say what each does; "New mode" opens a file of your own.

## 10. The background and the displays

- [ ] **Wallpaper page**: put a few images in `~/Pictures/Wallpapers/images` and a video in `~/Pictures/Wallpapers/live`. They show as pictures; pressing one sets it. In Yazi, `W` on a file sets it too.
- [ ] **A video** plays as the background, on both monitors. Open a window full screen (`Super+F`): the video should stop, to save the graphics card. In Gamer mode it stops too. Try Sound and Fit. *Watch `kavat-top` while it plays and say how much of the processor and the graphics card it takes: the virtual machine has no graphics card to measure.*
- [ ] **After a restart** the video plays again.
- [ ] **Colours from it**: turn it on with a colourful image: the whole system takes its colours. Change the image: the colours change. Say whether the result is good to look at with three or four different pictures; this is the part most likely to need tuning by eye.
- [ ] **Displays page**: your two monitors are drawn. Press one, change its resolution: it changes, and a line asks to keep it. Do not press Keep: after ten seconds it goes back. Change it again and press Keep.
- [ ] **Place**: put one monitor to the left of the other, and above. Keep the one you like. Restart: it is as you left it. *Two monitors were never tried: the virtual machine has one.*
- [ ] **Rotation and scale** on one monitor.
- [ ] **GPU page**: it names your RX 6800 and shows its temperature, use, memory, fan and power, moving as you look. Start something heavy and watch them rise. "Install LACT" brings the program for the fan curve; try setting a curve there. *The readings were tried on this PC's card from CachyOS; the page itself only in the virtual machine, which has no card.*
- [ ] **A USB stick**: plug it in: a notice offers to open it in Yazi and to take it out. Try both buttons. Try an NTFS stick if you have one.
- [ ] **Profiles**: on Appearance, type a name and press "Save current"; change the look; press the profile: the look comes back.

## 11. Backups and history

- [ ] **Previous versions**: an hour after installing, or after `sudo snapper -c home create`, change a file in your home folder; in Yazi press `V` on it: the list shows the version from before, with what changed. `Ctrl+R` puts it back beside the file. Delete a file, press `V` on its folder, Enter: a new tab opens inside the snapshot, and the file can be copied back from there.
- [ ] **Undo**: change a few things in the settings panel, then press **Undo** at the foot of the page, twice: each press takes one change back, and the system follows. **History** lists them all; Enter on one goes back to it.
- [ ] **Export** leaves a file in your home folder; say whether you want Import in the panel too.
- [ ] **A backup**: on Backup & sync, type a folder of an external disk as the place and press Enter; press "Back up now". Then `kavat-backup list`, and `kavat-backup restore latest ~/Documents/<a file> /tmp/back`. **Write the key down** (`kavat-backup key`). *A backup to another machine over SSH (`sftp:you@host:/folder`) was never tried.*
- [ ] **Sync**: with a private repository of yours, type its address in "Sent to", press Enter, then "Send now". *Never tried against a real server; it needs your SSH key to be set up for git.*
- [ ] **Keys**: `ssh-add -l` in a terminal answers (it says the agent has no keys, not that there is no agent). After the first `git push` over SSH that asks for the key's password, the next one does not ask. `secret-tool store --label test a b`, then `secret-tool lookup a b`, gives it back without asking for a password: the keyring opened with your login.
- [ ] **The microphone**: in the sound controls there is "Microphone, noise removed". Record yourself with it while typing on the keyboard (any recorder, or a call): the keys should be gone. *Never heard: the test machine has no microphone.* Turn it off on the Sound page if it gets in the way.
- [ ] **`kav news`** shows Arch's news since the installation, once there is some.
- [ ] **The firewall**: `sudo nft list table inet kavat_firewall` shows it. From another machine of your network, `ping` this one works and nothing else answers. *If something you use needs an open door (a local server, KDE Connect, a game), say which: a door is one line in `/etc/kavat/firewall.d/`.*

## 12. The browser

- [ ] **`Super+B`** opens Kavat Browser on its start page: the clock, the search, and the sites you go to most once there are some, in the theme's colours. Change the mode of the theme to light: a new tab is light.
- [ ] **Ads**: open a news site you know to be full of them: they are gone. The blocker's button on the toolbar turns it off for that site.
- [ ] **The console**: `Ctrl+Shift+K`, type `zen`, Enter: the page takes the whole screen; again to come back. Try "close other tabs", "sleep other tabs", "copy link", "downloads" (Yazi opens on the downloads).
- [ ] **Your own command**: add one to `~/.config/kavat/browser/commands.toml` (the file has examples), open the console: it is there.
- [ ] **A clipping**: select a paragraph on a page, `Ctrl+Shift+S`. The button on the toolbar shows a tick; the note is in `Super+Shift+N`, tagged `clip`, with the address and the paragraph as a quote. The right button's "Save to the notes" does the same.
- [ ] **Files**: on a page that uploads a file, the window that opens is Yazi; the same for "Save as". After a download, "Show in folder" opens Yazi at the file. *The window to pick a file was never seen from the browser: the test opens the browser but uploads nothing.*
- [ ] **Protected video**: play something on Netflix, Prime Video or Spotify's web player. *Never tried: it needs your account. If it says the browser is not supported, note the exact words: Google retires old versions of the module.*
- [ ] **The modes**: with the browser open, enter Incognito: its window goes out of sight, and `Super+B` opens a private window. Leave the mode: the first window is back. Enter Gamer with several tabs open: the tabs out of view have gone to sleep (they reload when you go to them).
- [ ] **Look for what is missing from the browser you use now.** It is Chromium with the system's policies: sync with a Google account is off, and extensions install from the Chrome Web Store as usual. Say what you miss.

## 13. Projects, the palette and the rest

- [ ] **A project**: in a terminal, inside a folder of yours, `kavat project new <name>`; then `kavat project open <name>`: an empty workspace gets a terminal and the editor in that folder. Go to another workspace and open it again: it only takes you there. On the Projects page of the settings it is listed.
- [ ] **The palette**: `Super+D`, then `/s dis` (Displays), `/f` and part of a file's name, `/pr`, `/m gamer`, `/k htop`, `/n` and a few words. Enter on each does what it says. *The typing in the launcher was not tried by the tests: note any of the six that shows nothing.*
- [ ] **`kavat`** and Tab, in a terminal: it offers its words; `kavat mode` and Tab offers the modes.
- [ ] **The screen's text**: `Shift+Print`, drag over some text on the screen, paste it somewhere: it is the text. Try it on Spanish text too.
- [ ] **On the bar**: start a call or a recording: a mark shows that the microphone is in use. `Alt+Shift` changes the keyboard, and the bar says which.
- [ ] **Neovim's marketplace**: `Ctrl+Shift+X` in the editor, with the network: a list of plugins to look through; install one and restart the editor: it is there. *Never tried with a network.*
- [ ] **The lights**, if your keyboard or tower has them: `kav install openrgb`, then change the theme's accent: they take it. *Never tried: it depends on OpenRGB knowing your devices.*
- [ ] **Privacy & network page**: it says the firewall is closed. "Turn it on" installs OpenSnitch; open a few apps and answer its questions. Say whether you want it on from the start.

## 14. What has not changed, once more

- [ ] A snapshot boots from the menu and `kavat-rollback` goes back to it.
- [ ] Recovery in the boot menu ends in its own menu.
- [ ] Windows and CachyOS still boot from "Other systems".

## What to send back

For each line that failed: its number, what you did, what you saw. A photo of the screen is enough for anything before the login. After the login, `journalctl -b > ~/log.txt` right after it happens.

## Found on the first pass

The user went through this list on the PC on 14 November 2026. Everything worked except what follows, and all of it was then done. Each line is something to try again on the next pass.

| What was seen | What was done |
| --- | --- |
| The installer showed one still picture, did not look like the mockup and had no step for the theme | It is drawn after the mockup, has the Appearance step, and shows eight pictures of the system while it installs |
| The desktop's background was the one inherited from Noctalia | It is the picture of the boot and the login, in the theme's colours; a background you choose yourself is kept |
| Icon style and Cursor size could not be changed | Tinted, Line and the sizes 32 and 48 are built |
| In `kav`, after `i`, it was not clear that it waited for the password | A window says what it will run and where the password is asked for, and sudo's line says what it is for |
| A note with a time made the note and no reminder | The notes on the PC's disk showed what was typed: "stand up in two minutes", with the number in words, and then a time inside quotes. Neither was understood. Numbers in words, quotes and the usual short forms are now, and the notification on saving says whether a reminder was set |
| `Ctrl+D` in the notes flashed an error and listed nothing | With nothing due, the preview tried to show a file that was not there. It says "Nothing is due" now |
| The login shows on one screen only | Left as it is: it looks good. The design says one screen |

## Known and not built

These are not failures: they are listed in the design document as left for later.

- The settings panel has twenty-three pages of twenty-six; the rest wait for the agent, or are the terminal's shell.
- The theme following the sun instead of the clock, and rules that change the profile.
- In the modes: all that waits for the agent and the browser, and the VPN's state on the bar.
- Displays: placing by dragging, the main display, and a background for each display.
- In the browser: the palette's colours on its own bars, a profile for each mode, albums, and the agent's panel.
- A profile for each game, encrypted DNS, the sandbox profiles, and placing the displays by dragging.
- In the installer: the programs that are only in the AUR, and repeating an installation from its answer file.
- In Yazi: the wallpaper action, and the indicator of a copy that works out of sight.
- `kavat-fetch` and `kavat-top` are fastfetch and btop with the theme, not the forks of the design yet.
- In the notes: the Albums tab, the index, images inside notes, and the bar's count.
- Neovim's marketplace, the side panels with Claude, and the Editor page.
- In `kav`: most of the Settings tab, and Arch's news.
