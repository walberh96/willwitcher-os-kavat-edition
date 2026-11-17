# Willwitcher-OS Kavat Edition — What to try by hand

Written on 14 November 2026 and brought up to date on 15 November, three times, for phase 3 · [Design document](DESIGN.md) · [Pending decisions](PENDING.md)

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
- [ ] **Boot and login look.** On the Boot page, press "Make them look like my desktop" with a light theme: it asks for your password in a dialog of the shell. Restart: the boot menu, the loading screen and the login are light. Put it back to dark the same way.
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

## 14. The browser's colours, its profiles and its albums

- [ ] **Its bars wear the palette.** Open the browser, then change the palette on Appearance: the bar of tabs and the address bar change with the browser open, in a second or so. Try Light too. *Seen in the virtual machine; on the PC, look at whether any text on the bars is hard to read with some palette, and say which.* The settings and history pages of the browser keep their own colours: that is known.
- [ ] **Other GTK 3 apps** now wear the palette through a theme called Kavat. If an app you install looks wrong (unreadable buttons or menus), note which.
- [ ] **A profile for a mode**: in `~/.config/kavat/browser/browser.toml`, take the `#` off `gamer = "Gaming"`. Enter Gamer and press `Super+B`: the window is another profile's, with no history and none of your sessions. Leave the mode and press `Super+B`: yours again.
- [ ] **An album**: on a site, `Ctrl+Shift+K`, type `add to album recipes`, Enter. Do it on two more sites. Then `albums` in the console: the page shows them as thumbnails. "Open all" opens the three; the ✕ on a card takes it out; "Show on the start page" puts the album under the search of every new tab.
- [ ] **Read later**: `Ctrl+Shift+L` on a page; on the albums' page, in `read-later`, the ✓ puts it away. *If `Ctrl+Shift+L` does nothing, look in `chrome://extensions/shortcuts`: Chromium gives a key to an extension only when nothing else has it.*
- [ ] **Tabs to album**: with several tabs open, `tabs to album trip` in the console; `open album trip` in a new window brings them back.
- [ ] **The right button**: on a page, "Add to the album of favourites" and "Read later"; on an image, "Save the image to the album of images".
- [ ] **Notes about this site**: after clipping a page of a site, go to another page of it, and `notes about this site` in the console, or on the right button: a panel at the side lists the clipping; a press opens it in the editor. Go to another tab: the panel follows.
- [ ] **A clipping has its picture**: clip a page, then open the note from `Super+Shift+N`: in Kitty, the picture of the page is drawn inside the note. Clip another selection of the same page: it is added to the same note. *The picture inside the editor was never seen with the real ImageMagick and Kitty.*
- [ ] **The sandbox**: on Privacy & network, turn on "The browser" under Sandbox, close the browser and open it again. Everything should work as before: the theme, the console, a clipping, a download to Downloads, a video. Then try to upload a file from `~/Documents`: Yazi lets you pick it, and the browser cannot read it; that is the sandbox, and a file in Downloads can be uploaded. *Tried in the virtual machine without sound, a camera or a real graphics card: note anything that stops working, above all video, the microphone in a call and Widevine.* Turn it off if it gets in the way.
- [ ] **An Electron app in a sandbox**, if you install one (Vesktop, say): `kavat-sandbox app vesktop`, then open it from the launcher: it starts with a home folder of its own, signed out. `kavat-sandbox app vesktop off` lets it out. *Never tried with a real app.*

## 15. Dragging the displays, games, names and the shell

- [ ] **Drag a display**: with the two monitors, on the Displays page drag one rectangle to the other side of the other, above it, and half way down its side: its edges catch the other's as it moves, and when dropped the real screens follow; "Keep" keeps it, and without it everything goes back in ten seconds. Turn one to 90° and drag it again. *The control was tried in the virtual machine with made-up rectangles; with two real displays it has never run. Note whether the rectangle jumps back for a moment when dropped.*
- [ ] **A game's profile**: on the GPU page, type `test 1245620` (a name, and the game's number in Steam: it is in the address of its page in the shop) and press "New profile"; "Edit" opens its file: set `fps_limit = 60`. Install MangoHud (`kav install mangohud gamescope`). In Steam, in the game's properties, set the launch options to `kavat-game run %command%`, and start it: the monitor is over the game and it does not go above 60. Take the `#` off the `[gamescope]` lines and try again. *Never tried with a real game. `gpu_profile` needs LACT and a profile made in it; the two commands it uses, `lact cli profile get` and `set`, were not tried: if the card's profile does not change, say so.*
- [ ] **Encrypted DNS**: on Privacy & network, "Turn it on": it asks for your password. Sites still open; `resolvectl status` says `+DNSOverTLS` and names Quad9. Change to Cloudflare. Enter Incognito with your VPN: sites still open there too. "Turn it off": back to your router's server (`cat /etc/resolv.conf`). *Never tried with a network.*
- [ ] **The Shell page**: set the prompt to Full and open a terminal inside a git repository with changes: the prompt shows them. Turn the suggestions off: a new terminal offers none. Press "Open them in Neovim" under Aliases, write `alias hi='echo hello'`, save, open a terminal: `hi` works.
- [ ] **Default apps**: in Yazi, Enter on an image opens imv, on a PDF zathura, on a video mpv.
- [ ] **A palette for the terminal alone**: `kavat theme set terminal.palette nord`: Kitty turns to Nord and the rest stays. `kavat theme set terminal.palette system` puts it back. The same with `editor.palette`.
- [ ] **The mode by the sun**: `kavat theme set auto.sun true auto.latitude <yours> auto.longitude <yours> auto.follow true` (west is below zero). `systemctl --user list-timers` shows the next change at today's sunset or tomorrow's sunrise, to within a few minutes.
- [ ] **A rule**: save two profiles on Appearance, `day` and `night`. Write `~/.config/kavat/theme-rules.toml` with a rule `profile = "night"`, `from = <this hour>`, `to = <the next>`, and a last rule with only `profile = "day"`. Within five minutes, or at once with `/usr/lib/kavat/theme-rules`, the look is `night`; `/usr/lib/kavat/theme-rules show` marks the rule that holds.

## 16. The notes and `kav`, what is new

- [ ] **The bar's count**: with a note due today or overdue, a number shows on the bar beside the notifications; a press opens the notes; with nothing due it is not there. *Never seen: it was not run in a shell before this build.*
- [ ] **Albums in the notes**: `Super+Shift+N`, then `6`: the albums the browser made. `Enter` opens a site, `O` the whole album, `R` marks read, `x` removes. On the other tabs, `b` and `B` choose a notebook; `a` adds to it.
- [ ] **Templates**: `n` on the notes' screen offers meeting, bug, idea and daily; `D` opens today's note. In a terminal, `kavat-notes new meeting "Monday"` opens the editor on it.
- [ ] **Chips**: open a note that has a reminder, a day it is due and tags: they are drawn as chips under its title, and again after saving.
- [ ] **A reminder waits in Gamer**: enter Gamer, set a reminder for two minutes from now: nothing shows; leave the mode: it shows. A note with `urgent: true` in its front matter shows at once.
- [ ] **`kav` with the mouse**: in `kav`, click a tab, a card of the Dashboard, a row; the wheel moves the list.
- [ ] **Browse**: `f` goes through installed, explicit, orphans and AUR.
- [ ] **Updates**: `kav check`, then the Updates tab: what waits, with its sizes and a mark on what asks for a restart. `systemctl start kav-check.service` with updates waiting leaves a notice.
- [ ] **Snapshots tab**: Enter on one says what its change did; `c` compares it with now; `p` pins it; `d` deletes one you do not need, after asking.
- [ ] **`kav update` inside Hyprland**: the desktop does not flicker or reload in the middle, and reloads once at the end. *The call that holds Hyprland back was tried by hand in the virtual machine; a whole update with a network never ran with it.*
- [ ] **A news item that asks for hands**, when Arch publishes one: `kav update` shows it, shows the commands, and asks before each. *Tried with a made-up item only.*
- [ ] **Firmware**: `kav install fwupd`, then `kav update`: it lists firmware when there is some. *The answer of fwupd was read from memory of its shape, not from a real one: if `kav update` says something odd at that step, send what `fwupdmgr get-updates --json` prints.*
- [ ] **`kav set cache 2`** leaves two versions of each package in `/var/cache/pacman/pkg`.

## 17. What has not changed, once more

- [ ] A snapshot boots from the menu and `kavat-rollback` goes back to it.
- [ ] Recovery in the boot menu ends in its own menu.
- [ ] Windows and CachyOS still boot from "Other systems".

## 18. The second pass: what was found, to try again

Each of these is something found on the PC on 15 November 2026 and corrected. One installation from the USB stick covers them all.

- [ ] **The live session asks for no keyring.** From the USB stick to the installer's last page, nothing asks for a password to make a keyring.
- [ ] **The login on two displays.** The main one asks who logs in; the other shows the clock over the same picture, not black. Turn the second display off and on at the login: it gets the clock again.
- [ ] **No picture of Noctalia at the start.** After the login the background is the theme's picture from the first moment; at most the theme's plain colour shows for an instant before it.
- [ ] **The bar** has no background picker beside the workspaces and no owl beside the power button, and it shows which keyboard is in use, `us` or `es`: `Alt+Shift` changes it.
- [ ] **Displays, keeping a change.** Set a display to 144 Hz: an amber band asks "Keep these display settings?" and counts down from ten. Without pressing anything, change the rotation too: the count starts again. Press **Keep**: both stay. Change the scale and let the count end: it goes back, and the page says the change was not kept.
- [ ] **Workspaces on each display.** On Displays, press the second display, then 6, 7, 8, 9 and 10 under "Workspaces on": `Super+6` opens on that display from then on, and a window sent with `Super+Shift+6` goes there. Log out and in: it is kept.
- [ ] **Network, Wi-Fi.** The Network page lists the cable with its speed and, on a machine with Wi-Fi, the networks in reach: Connect on a locked one asks for its password in the page and connects.
- [ ] **Network, Bluetooth.** Enabled is on. Put a controller or headphones in pairing mode, press **Pair a device**, wait ten seconds, press **Pair** on its line: it ends "connected", with its battery when the device says it. Disconnect, Connect and Forget do what they say.
- [ ] **A VPN for Incognito.** With no VPN yet, press Incognito on the bar: a notification says it needs a VPN and how one is added, and the Network page opens. Download a WireGuard file from your provider (Proton VPN: account.protonvpn.com, Downloads, WireGuard configuration), press **+ Add a VPN**, pick it in Yazi: it shows on the page as "used by Incognito". Press Incognito again: the mode is entered, and `curl ifconfig.me` gives the VPN's address. This is also the test of the network cut that waited from before.
- [ ] **Sound.** The Sound page lists the outputs and the microphones: choosing another output moves what is playing to it, the two sliders change the volumes, and a video playing in the browser shows under "Playing now" with a slider of its own.
- [ ] **Bar & widgets.** Position Bottom moves the bar at once; Top brings it back. The arrows move a widget along the bar, ✕ takes it off, and "Add a widget" puts one on: add "Processor and memory" to the right.
- [ ] **Boot.** The page says which look the boot menu, the loading screen and the login wear, beside the desktop's. Set "Show menu for" to 3 seconds and turn Recovery off: it asks for your password once; at the next start the menu waits three seconds and has no Recovery. Turn it back on.
- [ ] **The look of the boot, said plainly.** Change the desktop to Light on Appearance, open Boot: "They wear" and "Your desktop" differ, and the button "Make them look like my desktop" is there. Press it, restart: the boot menu, the loading screen and the login are light.
- [ ] **A new tab's background.** In a new tab, the **Background** button at its foot: "The desktop's background" puts the wallpaper behind the clock, "A picture of yours…" opens the file window and puts that one, "The theme's colour" takes it away. The Browser page of the settings shows the same choice.

## 19. What was built on the night of 16 November

None of this was seen on the PC yet. The virtual machine tried what it could; what it could not is said in each line.

- [ ] **Night light.** Displays, "Night light": On warms the colours of every display at once, Warmer more so, Off takes it away. Auto with From and Until set around the hour it is now turns it on and off by itself. After logging out and in, it is as it was left.
- [ ] **A background for each display.** With the two displays on, the Wallpaper page has "Background of": choose the second display and press an image: only that display changes, and its name gets a dot. Pressing the image the others show takes it back. Change the theme to Light: the display with its own image keeps it. *Tried with a made-up second display only.*
- [ ] **The slideshow.** With three or more images in `~/Pictures/Wallpapers/images`, set "Next image" to 5 min on the Wallpaper page: the background changes every five minutes, in the order of the names. Off stops it.
- [ ] **Proton VPN's own app.** Check Proton VPN on the installer's Software page, or press "Install Proton VPN" on the Network page. Open it from the Network page and sign in. Press "Use for Incognito", then Incognito on the bar: Proton connects by itself and the mode is entered; leaving the mode disconnects it. *Never tried with a real account: say what it says if the mode is not entered.*
- [ ] **A VPN with a name and a password.** Add an OpenVPN file that asks for them (Proton's OpenVPN files do): its line says "asks for your name and password" and has **Sign in**; type them, Keep, and Connect brings it up. *The virtual machine has no network: it was never really connected.*
- [ ] **A Bluetooth device with a code.** Pair a keyboard: a notice says the code to type on it. Pair something old that asks for a code, with **With a code** and `0000`. *Never tried: the virtual machine has no Bluetooth. If Pair now fails where it worked before, say so: the pairing was rewritten.*
- [ ] **The VPN on the bar.** With a VPN connected, "VPN · its name" stands beside the modes; a press opens the Network page. It goes within five seconds of disconnecting.
- [ ] **What starts by itself.** Boot, "Starts by itself": The other system. Restart and touch nothing: Windows starts. Choose Kavat on the menu, set it back to This system. *The virtual machine has no other system: only the menu's file was read.*
- [ ] **The boot follows the desktop.** Boot, "Follow my desktop" on. Change the palette or the accent, turn the machine off: it takes some seconds longer, and at the next start the menu, the loading screen and the login wear the new look. Turn off again without changing anything: it is as quick as always.
- [ ] **The login's clock and names.** Login & lock: 12 hours, and "Show who can log in" off. Log out: the clock says AM or PM, nobody is named, and typing your name and your password logs in; a wrong name says the password is not right. Set them back.
- [ ] **Logging in by itself.** In the installer, check "Log in automatically" on the Users page: the installed system opens the desktop with no password asked, and nothing asks for one afterwards: save a password in the browser and restart. Log out: the login asks as always. On Login & lock, "Logs in by itself" changes who, or sets it back to Nobody. *The passwords saved by who logs in by itself are not encrypted, unless the disk is.*
- [ ] **Qt apps in the theme.** Install a Qt app, `kav install qbittorrent` or `vlc`: it wears the palette, and changes with it when the theme changes while it is open.
- [ ] **Repeating the installation.** After installing, `cat /etc/kavat/kavat-install.toml`: it has your answers and no password. Copy it to the top folder of a USB stick, boot the installer with that stick plugged in and open it: the side bar says it was filled in from the file, and every page comes up as you answered it, in your language, with the passwords empty. The disk is still chosen and confirmed by you. *Next to the ISO on a Ventoy stick the file may not be found: say whether it is. `kavat-installer --answers <file>` in a terminal always works.*
- [ ] **A clipping with a comment, tags and its picture.** On a long page, select a paragraph, `Ctrl+Shift+K`, `save to notes`, Enter: the console asks for a comment, tags and the picture. Write both and press Enter: the note has your tags beside `#clip`, the comment and the quote. Again with **Whole page**: the page scrolls by itself and comes back, and the note gains a tall picture with the site's top bar in it once. Again with **A region**: the page dims, drag a rectangle, and the toolbar's button shows a tick; the picture is that rectangle. Esc while it is dimmed saves nothing. *Note any real site whose whole page comes out wrong: bars repeated, blank screens where pictures load late.*
- [ ] **A copy out of sight.** In Yazi, copy something large to another disk, and press `Esc` while the window is up. The right of the status bar says `Copying 42% · 1 of 3` and moves; `Ctrl+V` meanwhile says a copy is still working; a notice says when it is done.
- [ ] **The calendar's marks.** `Super+N`, "dentist friday 10:00". Open the control center, Calendar: Friday has a dot, and pressing it lists "dentist" at 10:00. Mark the note done in the notes: the dot goes. *If nothing shows the first time, log out and in. Say also if the shell itself notifies about a reminder: only the notes should.*
- [ ] **A picture in Albums.** Keep a page in an album from the browser, then `Super+Shift+N`, `6`, down to the album: the picture of the site under the cursor shows on the right, changes with the arrows, and goes away under Help (`?`) and when leaving the tab.
- [ ] **The mirrors.** `kav set mirror es`, then `kav set`: the mirror in use is a Spanish one, and `kav update` downloads from it. `kav set mirror worldwide` puts the first ones back. In `kav`, Settings tab, Enter on the mirror asks for the country in the terminal.
- [ ] **The room a snapshot takes.** `kav snapshots size`: every snapshot has a size; note how long it takes with the PC's real snapshots. In `kav`, Snapshots tab, `s` on one shows it in its row.
- [ ] **The signature of the reviewed list**, when the first package goes into it. Make the key once, on a USB stick: `export GNUPGHOME=/run/media/$USER/<stick>/kavat-key; mkdir -m 700 -p "$GNUPGHOME"; gpg --quick-generate-key "Willwitcher-OS Kavat Edition" ed25519 sign never`. After each change of `kav/allowlist.toml`, `tools/sign-allowlist.sh` with the stick in. A list changed after it was signed is refused by `kav`.

- [ ] **Booting a snapshot from `kav`.** In `kav`, Snapshots tab, `b` on a snapshot from before a change, Enter, the password: it says the next boot starts from it, once, and nothing restarts. Restart: the system is the snapshot (`cat /proc/cmdline` has `@snapshots/<n>/snapshot`). Restart again: it is the system as it was.
- [ ] **Ready to paste.** In Yazi, `Ctrl+C` on a file: the right of the status bar says `1 item ready to paste · as a copy`. Select three with Space and press `Ctrl+X`: `3 items ready to paste · as a move`. Paste them: the line goes. Copy again and press `Y`: it goes too.
- [ ] **Tabs in the launcher.** With the browser open on two or three tabs, `Super+D`, `/t` and a word of a tab's title: Enter brings the browser to the front on that tab, from another workspace too. `/win` does the same for windows. In Incognito, `/t` lists no tab.
- [ ] **Appearance, Auto and From wallpaper**, in the installer. Choose "From wallpaper" and Auto: the small desktop is light by day and dark by night. After installing, the boot menu and the login wear Kavat's palette in the side of that hour; at the first login the colours are taken from the background, and the mode follows the hour. Set another image as the background: the colours change with it. *The palette is made as the first session starts: say if the desktop comes up in Kavat's colours and stays so.*
- [ ] **The network, in the installer.** The bar of the live session says "No network · offline install" with the cable out and no Wi-Fi, and "Online" within five seconds of connecting. A press on it opens the Network page: join a Wi-Fi network there. On the installed system the same widget is the network's icon on the bar, and a press opens the Network page. *Never seen with a real network: the test machine has none.*
- [ ] **No empty password.** On the installer's Users page, Next stays off until a password is typed twice.
- [ ] **A display plugged in.** With the arrangement of the two displays kept, unplug the second one and plug it back in during a session: it comes back where it was, with its mode and its workspaces.
- [ ] **Searching for a setting.** `Super+T`, type `warmth` in the search: Displays is listed, with "Warmth" beside it. `password` or `clock` find theirs.
- [ ] **Incognito and recent files.** In Incognito, open a file from a GTK app's file window: after leaving the mode, it is not among that app's recent files.

## 20. What was built on 16 November, from the list of what was not

None of this was seen on the PC. What the virtual machine could not try is said in each line.

- [ ] **Changing a shortcut.** Keybindings: press "Files", type `Super+Shift+E`, Enter: `Super+E` opens nothing and `Super+Shift+E` opens Yazi. "As it comes" gives it back. Keys another shortcut has are refused, with its name.
- [ ] **`port-kill` and `y`.** Start a dev server: `port-kill` lists its port with its program, `port-kill <port>` stops it and names it; `port-kill 53` says it is not yours and asks for the password. `y`, go to another folder in Yazi, `q`: the terminal is in that folder. *On the PC already installed: `cp /etc/skel/.config/kavat/aliases.zsh ~/.config/kavat/` first.*
- [ ] **`sudo` on Esc twice, and more completions.** Type a command, press `Esc` twice: `sudo` is in front; twice more takes it away. `Tab` after a command only zsh-completions knows offers its options; with its switch off on the Shell page, a new terminal does not.
- [ ] **`kavat-fetch`.** In Kitty the cat is a picture, in the accent, beside the lines and not over them; after changing the accent it has the new colour; inside tmux it is the cat in characters. The Health line says `✓ ok`, with the updates waiting after `kav check` finds some.
- [ ] **No YouTube Shorts.** With a network: the home page has no Shorts shelf, the side bar no Shorts entry, a channel no Shorts tab, and the subscriptions and a search show none. Browser page, "Hide Shorts" off, back to the tab: they are there. *Never tried on the real site.*
- [ ] **A page as an app.** `kavat webapp install`, a name, `https://music.youtube.com` and nothing for the icon: the launcher has the app with the site's icon; it opens in a window with no tabs and no address bar, logged in where the browser is. In the browser, `Ctrl+Shift+K`, "install app", Enter does it for the page in view. The Browser page lists them, with Remove.
- [ ] **Wallpapers from the network.** Wallpaper, Online: search "mountains": pictures appear within seconds; press one: it becomes the background and is in `~/Pictures/Wallpapers/images`. With no network, the same search says so in a line.
- [ ] **A video on one display.** With two displays: a video on one and an image on the other; both show, the video pauses under a full-screen window, and after logging out and in both are back. *Tried with stand-ins only.*
- [ ] **The lock shows the background.** With an image or a video of yours as the background, `Super+Shift+L` shows it, or the video's still frame, behind the clock; with "The lock shows it" off on the Wallpaper page, the theme's picture.
- [ ] **The browser in Incognito.** In the mode, `Super+B` opens a browser with nothing in it: no history, not logged in anywhere, and with the blocker's icon there and ads blocked. Log in somewhere, leave the mode: that browser closes; enter the mode again: it is logged in nowhere.
- [ ] **Incognito hides what notifications say.** With the mode on, a message from a chat app shows the app's name and "Notification", in the toast and in the history; its button still opens the app; after leaving the mode, new ones show in full.
- [ ] **The installer finds the disk again.** Started with the answers of the earlier installation, the disk page comes up with the same disk chosen although its nvme name changed, and the side bar says which; with a file from another machine, none is chosen and the side bar says so. *Check the disk it names before pressing Install: this was never tried with two real disks.*
- [ ] **Claude beside the editor.** `kav install claude-code` once it is on the reviewed list, `claude` once in a terminal to log in. In Neovim, `Ctrl+Alt+B` opens Claude in a Kitty window to the right, and `/ide` there says it is connected; select lines, `Ctrl+Alt+K`: Claude has them; ask for a change: it opens as a diff, `<leader>aa` applies it and `<leader>ad` drops it; `Ctrl+Alt+B` again closes it. *Never tried with a real Kitty or a real Claude Code.*
- [ ] **Remote access.** It needs a Tailscale account and a second device with Tailscale and Moonlight; a phone will do.
  - Before turning it on: the page says off, the bar has no mark, and `ssh` to this computer from the local network is refused.
  - Turn it on: the password is asked for, the page shows an address and a picture to sign in with, and after signing in it shows this computer's name on the private network. The bar carries the mark.
  - Paste the phone's public key on the page, `ssh` from the phone to that name: it comes in, a notification says so, and the mark counts it. With a password, or as root, it does not.
  - From a machine of the local network that is not in the Tailscale network, `ssh` and Moonlight still do not reach it.
  - In Moonlight, add this computer by its Tailscale name: it shows a PIN; typed on the page, the phone is paired and sees the desktop, with keyboard and mouse.
  - Left alone, the computer stays awake and reachable, and the screen still locks. After a restart, `ssh` works before anyone logs in and the desktop only after.
  - Enter Incognito: the phone cannot reach the computer; leave it: it can again.
  - Turn it off: the mark goes, the phone reaches nothing, and `systemctl is-enabled tailscaled sshd` says disabled.
  - *None of the lines above was ever tried: only that it is off as the system comes, and what the firewall is told.*

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
| The login shows on one screen only | Left as it was then; on the second pass the other screen was black, and that was fixed: see below |

## Found on the second pass

The user went through the list on the PC on 15 November 2026 and left what was found in `~/Documents/testing.md`. All of it was done; section 18 tries each again.

| What was seen | What was done |
| --- | --- |
| In the installer, a window asked for a password to make a keyring | It was the shell, keeping the key of its clipboard history: nobody types a password to enter the live session, so no keyring was open. In the live session that key is a file now |
| At the login, the second display was black | The display woke up after the login had started, and only the screens there at that moment got a page. A screen gets its page when it appears |
| At the start, Noctalia's background showed before Kavat's | The shell showed its own picture until the system set the theme's. That picture is out of the shell; the theme's colour shows until its picture is read |
| A change of a display was undone at the next change | It was the ten seconds to keep a change: the question was a line of small text and went unseen. It is a band across the page now, with the seconds counted and the Keep button in it, and the page says when a change was undone |
| Network, Bluetooth and others were a button to the shell's controls | Network, Sound, Bar & widgets, Boot and Login & lock have the controls themselves now. Bluetooth was not installed at all: BlueZ comes with the system and is on |
| Incognito did nothing: no VPN, and nowhere to set one | A notification says why and the Network page opens; a VPN is added there from the provider's file |
| The background picker and the owl on the bar looked out of place | Both are off the bar. The background is chosen on the Wallpaper page; the control center opens from the network, Bluetooth and volume widgets |
| "Give them the look in force" was not understood | The Boot and the Login pages say which look those screens wear, which the desktop wears, and offer to make them the same |
| No way to choose a new tab's background | The theme's colour, the desktop's background or a picture, from the tab itself and from the settings |
| No way to say which workspaces go on which display | Each display has its workspaces on the Displays page |

Seen on the way: the keyboard's indicator was not on the bar, its name misspelt in the shell's settings. It is there now.

## Known and not built

These are not failures: they are listed in the design document as left for later.

- The settings panel has twenty-five pages of twenty-seven; the other two wait for the agent.
- In the modes: what waits for the agent. 
- Wallpaper: the login showing a video's still frame, and the lock showing each display's own background.
- In the browser: the palette on the pages Chromium draws itself and hiding its native panels, which are to be built with a Chromium compiled here (D46); and the agent's panel.
- More of your aliases and functions from Endgame, when you go over them: `port-kill` and `y` are in.
- In the installer: the programs that are only in the AUR, which wait for their review (D44).
- `kavat-top` is btop with the theme, not the fork of the design, which is phase 4; `kavat-fetch` has no line for the agent.
- In the notes: the agent's tools. The agent's tools for containers, git, ports and remote access: phase 4.
- In Neovim: the panels other than Claude's, popping a panel out, and a memory for each project.
