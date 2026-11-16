# Willwitcher-OS Kavat Edition — Pending decisions

Updated on 16 November 2026 · [Design document](DESIGN.md) · [Interactive mockup](mockup/index.html)

## How it works

Only what waits for your decision is here. Everything else lives in the [design document](DESIGN.md) and in the mockup.

- **If you approve something**, it goes to the design document and to the mockup, and leaves this file.
- **If you reject it**, it is deleted from here and is not asked again.
- **To answer**, the number in the chat is enough ("D1 yes, D5 no"), or write the answer next to the decision in this file.

## Summary

| # | Decision | Recommendation |
| --- | --- | --- |
| D50 | Remote access: reaching this computer from your other devices, off as the system comes | Yes, as the mockup draws it: Tailscale, SSH and Sunshine behind one switch |

## D50 · Remote access

You asked for a way to reach the PC at home from somewhere else, free and in your hands, as a piece of the system that does nothing until it is turned on. The setup is the one your colleague recommended and we went over: Tailscale to reach the machine, Sunshine and Moonlight to see its desktop, and SSH beside them.

**It is drawn in the mockup**: Settings, the page "Remote access", marked as proposed. Its switch, "Enabled", shows both states. [Off](mockup/index.html#scene=settings&spage=remote) · [On](mockup/index.html#scene=settings&spage=remote&remote=true)

### What it is

A page of the settings panel and a command, `kavat-remote`, in a package of its own, `packages/kavat-remote/`.

**Off as the system comes.** Three programs are installed and stopped: Tailscale, the SSH server and Sunshine. No service of theirs is enabled, the firewall has no door for them, and the installer does not ask about it.

**Turning it on does four things**, and turning it off undoes the four:

1. **Joins your Tailscale network.** It is a private network between your own devices, WireGuard underneath. The first time, the page shows an address and a QR code to sign in with. Tailscale does not get to set the system's DNS, so encrypted DNS stays as it is.
2. **Opens the terminal and the desktop to that network only.** The SSH server takes keys only, no passwords and no root. Sunshine starts with your session. The firewall opens both on Tailscale's interface and nowhere else: not to the local network, not to the internet. The router needs no open port.
3. **Keeps the computer awake.** A suspended machine cannot be reached, and nothing wakes it from outside. The screen still locks and turns off.
4. **Marks the bar**, for as long as it is on, with how many devices are connected. A notification says when one comes in.

**The page shows** the computer's name and address in the network, who is connected now, with a button to disconnect them, and which devices may come in: the SSH keys, added by pasting them, and the devices paired with Moonlight, paired by typing its PIN there. Each of the two, terminal and desktop, has its own switch, and so does staying awake.

**What stands around it:**

- Turning it on or off asks for your password.
- **Incognito pauses it** and gives it back when you leave the mode: Incognito lets nothing out but through its VPN.
- **Kavat, the agent,** can say whether it is on and who is connected, and can turn it off, with confirmation. It has no tool to turn it on.
- After a restart, the terminal works from the start; the desktop can be seen only once someone has logged in, unless logging in by itself is on. The page says so.
- The file behind the page is `/etc/kavat/remote.conf`.

### Three choices inside it, with my pick

| Choice | My pick | The other way |
| --- | --- | --- |
| Who coordinates the private network | **Tailscale's own servers**, free for personal use, with a field on the page for a server of yours | A server of yours from the start (Headscale): no third party, but it needs a machine with a public address that is always on. The field leaves that door open for later |
| How the terminal comes in | **The system's SSH server with your keys.** Who comes in is decided on this machine | Tailscale's own SSH: no keys to keep, but Tailscale's account decides who comes in |
| Where it lives in the panel | **A page of its own**, the twenty-seventh, after Network: it has its own lists of devices and keys | A block on the Network page: one page less, and a crowded one |

### What it costs and what is not known yet

- **Sunshine is not in Arch's repositories.** It is built in `packages/sunshine/` from its project's own build file, in the clean chroot, as mpvpaper and fzf-tab are. Tailscale and the SSH server are in Arch's repositories.
- **To check when it is built**, on the virtual machine or on the PC: which of Sunshine's ways of capturing the screen works on Hyprland; that the lock screen is seen and opened through it; that the Super key arrives from Moonlight on a Mac; that Tailscale's own firewall rules sit well beside the system's; and what is seen with the monitor turned off.
- **The tests**: the virtual machine can check that nothing listens while it is off and that the doors open only to the private network's interface. Signing in to Tailscale needs an account and a browser, and the picture needs a second device: both go to the manual tests.

### Left out

- **RustDesk and VNC**: other ways to the same picture. Sunshine is the one you chose.
- **A port open on the router**: what this is here to avoid.
- **Wake-on-LAN**: it needs another device awake at home.
- **Claude Code's Remote Control**: it is Claude Code's own and works by itself; the system adds nothing to it.

**Recommendation: yes, with the three picks above.** The package, the page and the lines of the manual tests are built on the PC; the design document, the mockup and the README take it in when you approve.

**If you say no**, the page leaves the mockup and none of the three programs is installed. They can still be set up by hand, with a door of your own in `/etc/kavat/firewall.d/`.

Settled on 16 November 2026:

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
