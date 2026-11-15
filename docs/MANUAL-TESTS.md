# Willwitcher-OS Kavat Edition — What to try by hand

Written on 14 November 2026, for phase 3 · [Design document](DESIGN.md) · [Pending decisions](PENDING.md)

The automatic tests run in a virtual machine with one screen, no network and a keyboard that types through a remote console. This is the list of what they cannot see: the real graphics card, two monitors, the mouse, the network, and how things feel. It is meant to be gone through once, from top to bottom, after one installation.

Mark each line as you go. For anything that fails, the line says what to note down.

## Before starting

- Write the ISO in `out/` to the USB stick, or ask for it to be written.
- Install as before, on the 230.8 GiB Btrfs partition of the 465.8 GiB disk. **Do not touch the Windows disk** (476.9 GiB, NTFS).
- The first part needs no network. The last part does.

## 1. From the boot menu to the desktop

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
- [ ] **Pages not built** say so in one line. The greyed controls (Auto, Tinted, Line, 32, 48, profiles, Undo, History, Export) cannot be pressed.

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
- [ ] **Dragging**: drag a file from Yazi to another window with the mouse, and drop a file from elsewhere onto Yazi. *Never tried: it is Yazi's own, through Kitty. If it does not work, say so: ripdrag was dropped on its account.*
- [ ] **`C`** compresses the selection (type `name.zip`), Enter on an archive offers to extract.
- [ ] **`M`**? It is not set: the disks are not mounted from Yazi yet. Plug a USB stick and note what you expected.
- [ ] **Previews**: an image, a PDF, a video, an archive.
- [ ] **`e`** opens in the editor, **`T`** a terminal there, **`Ctrl+P`** the file search, **`g` `t`** the trash.

The window apps use to open and save files cannot be tried until there is an app that asks: the browser comes later.

## 7. Notes

- [ ] **`Super+N`**: a line appears. Type `try the notes #kavat` and Enter: a notification says it was saved.
- [ ] **A reminder**: `Super+N`, type `stand up in 2 minutes`, Enter. Two minutes later a notification comes up with Open, In 10 minutes, In 1 hour, Tomorrow and Done. **Press each button on different reminders.** *The buttons were never pressed: the test only sees the notification come up.*
- [ ] **`Super+Shift+N`** opens the notes; `Ctrl+D` shows what is due; Enter opens a note in Neovim.
- [ ] **A reminder across a restart**: set one for five minutes ahead, switch off, come back after its time: it shows at login.
- [ ] **In a terminal**: `nb`, `nb due`, `nb remind 1 tomorrow 9:00`.

## 8. kav

- [ ] **`Super+U`** opens `kav` in a floating window. It should look like `docs/screenshots/kav-dashboard.png`: four boxes with numbers, Health, Recent activity, the keys at the bottom.
- [ ] **`1` to `7`** change tab; **`/`** then a name and Enter searches; Enter shows the record.
- [ ] **`5`**, Enter: asks for your password and lists the snapshots.

With the network:

- [ ] **`kav update`** fetches the lists and updates. *Never run: the test machine has no network.*
- [ ] **`kav install htop`**, then **`kav remove htop`**, then **`kav undo`** and restart.
- [ ] **An AUR package**: `kav request <name>`, `kav review <name>`, approve it, `kav install <name>`. *The build itself was never run: it needs the network. This is the most likely place for something to fail.*
- [ ] **`kav doctor`**.

## 9. What has not changed, once more

- [ ] A snapshot boots from the menu and `kavat-rollback` goes back to it.
- [ ] Recovery in the boot menu ends in its own menu.
- [ ] Windows and CachyOS still boot from "Other systems".

## What to send back

For each line that failed: its number, what you did, what you saw. A photo of the screen is enough for anything before the login. After the login, `journalctl -b > ~/log.txt` right after it happens.

## Known and not built

These are not failures: they are listed in the design document as left for later.

- The settings panel has six pages of twenty-six; the rest say they are not built.
- Icon styles Tinted and Line, cursor sizes 32 and 48, profiles, and the theme following the time of day.
- The copy window of Yazi, mounting disks from it, and the wallpaper action.
- The notes' own screen with task views, the index, images inside notes, and the bar's count.
- Neovim's marketplace, the side panels with Claude, and the Editor page.
- In `kav`: changing settings, Arch's news, and fixing things from Doctor.
