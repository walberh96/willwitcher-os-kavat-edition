// The programs the Software page of the installer offers, by category. Each is the name
// of its package in Arch's repositories, what to call it, and what it is. They are all
// from Arch's own repositories: what is only in the AUR comes in later, through kav's
// reviewed list.
.pragma library

var categories = [
    { name: "Browsers", programs: [
        { pkg: "firefox", name: "Firefox", what: "a second browser, beside Kavat Browser, which comes with the system" },
    ] },
    { name: "Development", programs: [
        { pkg: "git github-cli", name: "git · github-cli", what: "version control, gh" },
        { pkg: "lazygit", name: "lazygit", what: "git on one screen; Neovim opens it too" },
        { pkg: "mise", name: "mise", what: "Node and other tools, a version per project" },
        { pkg: "pnpm", name: "pnpm", what: "package manager" },
        { pkg: "docker docker-compose docker-buildx", name: "docker · compose", what: "containers" },
        { pkg: "lazydocker", name: "lazydocker", what: "the containers on one screen" },
        { pkg: "postgresql", name: "postgresql", what: "the psql client" },
        { pkg: "rustup", name: "rustup", what: "rustc, cargo, clippy, rust-analyzer" },
        { pkg: "lldb gcc pkgconf", name: "lldb · gcc · pkg-config", what: "debugger and build tools" },
        { pkg: "jdk17-openjdk", name: "jdk17-openjdk", what: "Android builds" },
        { pkg: "android-tools", name: "android-tools", what: "adb" },
    ] },
    { name: "Creation", programs: [
        { pkg: "obs-studio", name: "OBS Studio", what: "recording and streaming" },
        { pkg: "gimp", name: "GIMP", what: "images" },
        { pkg: "inkscape", name: "Inkscape", what: "drawings" },
        { pkg: "kdenlive", name: "Kdenlive", what: "video editing" },
        { pkg: "blender", name: "Blender", what: "3D" },
    ] },
    { name: "Games", programs: [
        { pkg: "mangohud", name: "MangoHud", what: "the performance monitor over a game" },
        { pkg: "gamescope", name: "gamescope", what: "a game's own resolution and scaling" },
        { pkg: "gamemode", name: "gamemode", what: "the processor tuned while a game runs" },
        { pkg: "lutris", name: "Lutris", what: "games from other stores" },
        { pkg: "lact", name: "LACT", what: "the graphics card's fan curve and power limit" },
        { pkg: "openrgb", name: "OpenRGB", what: "the lights of the keyboard and the tower, in the theme's accent" },
    ] },
    { name: "Communication", programs: [
        { pkg: "discord", name: "Discord", what: "chat and voice" },
        { pkg: "telegram-desktop", name: "Telegram", what: "messages" },
        { pkg: "thunderbird", name: "Thunderbird", what: "mail" },
    ] },
    { name: "Media", programs: [
        { pkg: "mpv", name: "mpv", what: "video and audio" },
        { pkg: "zathura zathura-pdf-mupdf", name: "zathura", what: "PDF" },
        { pkg: "imv", name: "imv", what: "images" },
    ] },
    { name: "System", programs: [
        { pkg: "flatpak", name: "Flatpak", what: "programs kept apart from the base system" },
        { pkg: "opensnitch", name: "OpenSnitch", what: "asks the first time a program reaches the network" },
        { pkg: "proton-vpn-gtk-app proton-vpn-cli", name: "Proton VPN", what: "Proton's own app, with its login; Incognito mode can use it" },
    ] },
    { name: "Office", programs: [
        { pkg: "libreoffice-fresh", name: "LibreOffice", what: "documents, sheets and slides" },
    ] },
]
