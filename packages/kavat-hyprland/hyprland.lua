-- Willwitcher-OS Kavat Edition: the system's Hyprland configuration.
--
-- It is installed in /usr/share/kavat/hypr/ and is what a session uses until the user writes
-- ~/.config/hypr/hyprland.lua, which then takes its place.
--
-- The shortcuts are the ones of Willwitcher-OS Endgame, plus the new ones whose target
-- exists today. The full table is in docs/DESIGN.md, under "Desktop".

local mod      = "SUPER"
local terminal = "kitty"
local files    = "kitty yazi"

-- A display nothing is said of shows its best mode, to the right of the others.
hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })

-- The arrangement kept for the displays connected, which kavat-displays writes: one
-- arrangement for each set of displays, and the one named in `active` is worn.
do
    local home = os.getenv("XDG_CONFIG_HOME") or (os.getenv("HOME") .. "/.config")
    local read, kept = pcall(dofile, home .. "/hypr/monitors.lua")
    if read and type(kept) == "table" and type(kept.sets) == "table" then
        for _, display in ipairs(kept.sets[kept.active] or {}) do
            hl.monitor({ output = display.output, mode = display.mode, position = display.position,
                         scale = display.scale, transform = display.transform })
            -- the workspaces that open on this display; the first of them is the one it starts on
            local first = true
            for number in tostring(display.workspaces or ""):gmatch("%d+") do
                hl.workspace_rule({ workspace = number, monitor = display.output, default = first })
                first = false
            end
        end
    end
end

-- The shell: bar, launcher, notifications and wallpaper
hl.on("hyprland.start", function()
    hl.exec_cmd("kavat-shell --daemon")
    -- The cursor and the icons are the theme's: the cursor is packed from what kavat-theme
    -- wrote, and the icons of the style in force are made and the apps told
    hl.exec_cmd("kavat-cursor-apply")
    -- the background is the theme's, unless you chose one of your own
    hl.exec_cmd("/usr/lib/kavat/wallpaper")
    -- a video you chose as the background plays again
    hl.exec_cmd("kavat-wallpaper restore")
    -- the arrangement kept for the displays that are connected now
    hl.exec_cmd("kavat-displays restore")
    -- Wi-Fi and Bluetooth are turned off for who set them not to be on at startup
    hl.exec_cmd("kavat-network startup")
    -- light by day and dark by night, for who asked for it
    hl.exec_cmd("/usr/lib/kavat/theme-auto")
    hl.exec_cmd("kavat-icons-apply")
    -- GTK 3 apps, the browser among them, wear the theme's own theme of GTK, Kavat: a
    -- session that has another is given it; kavat-theme goes between its two names after that
    hl.exec_cmd([[sh -c 'case $(gsettings get org.gnome.desktop.interface gtk-theme) in *Kavat*) ;; *) gsettings set org.gnome.desktop.interface gtk-theme Kavat ;; esac']])
    -- the lights of the keyboard and the tower take the accent, when OpenRGB is installed
    hl.exec_cmd("/usr/lib/kavat/lights")
    -- The reminders written in the notes become timers of this session; those whose time
    -- passed while the computer was off are shown now
    hl.exec_cmd("kavat-notes sync")
    -- A disk that is plugged in is mounted, and a notice offers to open it or take it out
    hl.exec_cmd("udiskie --config /usr/share/kavat/udiskie.yml")
    -- Every session starts in Coder: what another mode left changed is undone. Then the
    -- watch that enters Gamer when Steam starts a game.
    hl.exec_cmd("kavat-mode restore; exec kavat-mode watch")
    -- The screen locks when nobody is there. Not on the live medium, whose user has no
    -- password to come back with.
    hl.exec_cmd("test -e /run/archiso || hypridle")
end)

-- the agent that holds the SSH keys, for every app of the session
hl.env("SSH_AUTH_SOCK", (os.getenv("XDG_RUNTIME_DIR") or ("/run/user/" .. (os.getenv("UID") or "1000"))) .. "/ssh-agent.socket")
hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")

-- Look: gaps, borders, corners, colours and effects come from the theme, as a table of
-- values that kavat-theme writes. It is asked to write them first, so that they are there
-- on a first login and still match the settings after an update; when they already do,
-- that changes nothing.
local function look()
    os.execute("kavat-theme apply --no-reload")
    local state = os.getenv("XDG_STATE_HOME")
    if not state or state == "" then
        state = os.getenv("HOME") .. "/.local/state"
    end
    local found, values = pcall(dofile, state .. "/kavat/theme/current/hyprland/look.lua")
    if found and type(values) == "table" then
        return values
    end
    -- no theme to be had: the values of Endgame, with Hyprland's own colours
    return {
        general    = { gaps_in = 5, gaps_out = 20, border_size = 2 },
        decoration = { rounding = 10 },
    }
end
hl.config(look())

-- The keyboard: the system's two, US and Spanish, with the one in use first; Alt+Shift goes
-- to the other. Which is first is what the Language & region page of the settings wrote
-- or, until then, the keyboard chosen when the system was installed.
local function keyboards()
    local first
    local config = os.getenv("XDG_CONFIG_HOME")
    if not config or config == "" then
        config = os.getenv("HOME") .. "/.config"
    end
    local places = { config .. "/kavat/desktop.json", "/etc/X11/xorg.conf.d/00-keyboard.conf", "/etc/vconsole.conf" }
    for _, place in ipairs(places) do
        local file = io.open(place)
        if file then
            local text = file:read("a")
            file:close()
            first = text:match('"keyboard_first"%s*:%s*"(%a+)"') or text:match('XkbLayout"%s+"(%a+)') or text:match("KEYMAP=(%a+)")
            if first then
                break
            end
        end
    end
    return first == "es" and "es,us" or "us,es"
end
hl.config({ input = { kb_layout = keyboards(), kb_options = "grp:alt_shift_toggle" } })

hl.config({
    general = {
        layout = "dwindle",
    },
    decoration = {
        -- Hyprland darkens a window while one of its dialogs is open, and leaves it dark
        -- when the dialog closes without the window being touched: the installer stayed
        -- dimmed after a partition was picked.
        dim_modal = false,
    },
    dwindle = {
        preserve_split = true,
    },
    misc = {
        -- the wallpaper belongs to the shell
        force_default_wallpaper = 0,
        disable_hyprland_logo   = true,
    },
    ecosystem = {
        -- Hyprland greets each new version with a window of news, and asks for donations
        -- now and then; updates here come through the system, not through Hyprland
        no_update_news  = true,
        no_donation_nag = true,
    },
})

-- Shortcuts inherited from Endgame

hl.bind(mod .. " + Return", hl.dsp.exec_cmd(terminal))
hl.bind(mod .. " + Q", hl.dsp.window.close())
hl.bind(mod .. " + D", hl.dsp.exec_cmd("kavat-shell msg panel-toggle launcher"))
hl.bind(mod .. " + E", hl.dsp.exec_cmd(files))
hl.bind(mod .. " + B", hl.dsp.exec_cmd("kavat-browser"))
hl.bind(mod .. " + V", hl.dsp.window.float({ action = "toggle" }))
hl.bind(mod .. " + P", hl.dsp.window.pseudo())
hl.bind(mod .. " + J", hl.dsp.layout("togglesplit"))

for _, direction in ipairs({ "left", "right", "up", "down" }) do
    hl.bind(mod .. " + " .. direction, hl.dsp.focus({ direction = direction }))
end

for i = 1, 10 do
    local key = i % 10 -- workspace 10 is on the 0 key
    hl.bind(mod .. " + " .. key, hl.dsp.focus({ workspace = i }))
    hl.bind(mod .. " + SHIFT + " .. key, hl.dsp.window.move({ workspace = i }))
end

hl.bind(mod .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }))
hl.bind(mod .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }))

hl.bind(mod .. " + S", hl.dsp.workspace.toggle_special("scratchpad"))
hl.bind(mod .. " + SHIFT + S", hl.dsp.window.move({ workspace = "special:scratchpad" }))

hl.bind("ALT + Tab", hl.dsp.window.cycle_next())
hl.bind(mod .. " + SHIFT + M", hl.dsp.exec_cmd("kavat-mode next"))
hl.bind(mod .. " + SHIFT + V", hl.dsp.exec_cmd("kavat-shell msg panel-toggle clipboard"))
hl.bind(mod .. " + SHIFT + P", hl.dsp.exec_cmd("hyprpicker -a"))
-- a region of the screen, read: its text goes to the clipboard
hl.bind("SHIFT + Print", hl.dsp.exec_cmd("/usr/lib/kavat/screen-text"))
hl.bind("Print", hl.dsp.exec_cmd('grim -g "$(slurp)" - | wl-copy'))
hl.bind(mod .. " + SHIFT + L", hl.dsp.exec_cmd("hyprlock"))
hl.bind(mod .. " + M", hl.dsp.exit())

hl.bind(mod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(mod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- New shortcuts

hl.bind(mod .. " + F", hl.dsp.window.fullscreen({ action = "toggle", mode = "fullscreen" }))

hl.bind(mod .. " + Escape", hl.dsp.exec_cmd("kavat-shell msg panel-toggle session"))

-- The file search, in a window of its own over whatever is open. Kitty asks for the size
-- its last window had, which here was a tiled one: it is told not to.
hl.bind(mod .. " + SHIFT + F",
    hl.dsp.exec_cmd("kitty --class kavat-find -o remember_window_size=no kavat-find --window"))
hl.window_rule({
    name   = "file-search",
    match  = { class = "^kavat-find$" },
    float  = true,
    size   = "(monitor_w*0.7) (monitor_h*0.7)",
    center = true,
})

-- The system's settings, on the page of the look; and kav, on its full screen
hl.bind(mod .. " + T", hl.dsp.exec_cmd("kavat-shell msg panel-toggle kavat/settings:panel"))
hl.bind(mod .. " + U", hl.dsp.exec_cmd("kitty --class kavat-kav -o remember_window_size=no kav"))
-- The cheat sheet: every shortcut of this file, on the Keybindings page of the settings
hl.bind(mod .. " + slash", hl.dsp.exec_cmd("kavat-shell msg panel-open kavat/settings:panel keys"))
hl.window_rule({
    name   = "kav",
    match  = { class = "^kavat-kav$" },
    float  = true,
    size   = "(monitor_w*0.7) (monitor_h*0.7)",
    center = true,
})

-- Notes: a line to type one in, over whatever is open, and the notes themselves
hl.bind(mod .. " + N",
    hl.dsp.exec_cmd("kitty --class kavat-note -o remember_window_size=no kavat-notes quick"))
hl.bind(mod .. " + SHIFT + N",
    hl.dsp.exec_cmd("kitty --class kavat-notes -o remember_window_size=no kavat-notes"))
hl.window_rule({
    name   = "quick-note",
    match  = { class = "^kavat-note$" },
    float  = true,
    size   = "900 150",
    center = true,
})
hl.window_rule({
    name   = "notes",
    match  = { class = "^kavat-notes$" },
    float  = true,
    size   = "(monitor_w*0.75) (monitor_h*0.75)",
    center = true,
})

-- The installer, on the live medium: a window in the middle, the size it is drawn for
hl.window_rule({
    name   = "installer",
    match  = { class = "^io\\.calamares\\.calamares$" },
    float  = true,
    size   = "1280 800",
    center = true,
})

-- Yazi when it is the window an app picks a file with: over the app, in the middle
hl.window_rule({
    name   = "file-picker",
    match  = { class = "^kavat-picker$" },
    float  = true,
    size   = "(monitor_w*0.7) (monitor_h*0.7)",
    center = true,
})

local step = 40 -- pixels a window grows or shrinks with each press
for direction, by in pairs({ left = { -step, 0 }, right = { step, 0 }, up = { 0, -step }, down = { 0, step } }) do
    hl.bind(mod .. " + SHIFT + " .. direction, hl.dsp.window.move({ direction = direction }))
    hl.bind(mod .. " + CTRL + " .. direction, hl.dsp.window.resize({ x = by[1], y = by[2], relative = true }),
        { repeating = true })
end
