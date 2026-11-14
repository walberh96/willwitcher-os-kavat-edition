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

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })

-- The shell: bar, launcher, notifications and wallpaper
hl.on("hyprland.start", function()
    hl.exec_cmd("noctalia --daemon")
end)

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
hl.bind(mod .. " + D", hl.dsp.exec_cmd("noctalia msg panel-toggle launcher"))
hl.bind(mod .. " + E", hl.dsp.exec_cmd(files))
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
hl.bind(mod .. " + SHIFT + V", hl.dsp.exec_cmd("noctalia msg panel-toggle clipboard"))
hl.bind(mod .. " + SHIFT + P", hl.dsp.exec_cmd("hyprpicker -a"))
hl.bind("Print", hl.dsp.exec_cmd('grim -g "$(slurp)" - | wl-copy'))
hl.bind(mod .. " + SHIFT + L", hl.dsp.exec_cmd("hyprlock"))
hl.bind(mod .. " + M", hl.dsp.exit())

hl.bind(mod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(mod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- New shortcuts

hl.bind(mod .. " + F", hl.dsp.window.fullscreen({ action = "toggle", mode = "fullscreen" }))

hl.bind(mod .. " + Escape", hl.dsp.exec_cmd("noctalia msg panel-toggle session"))

local step = 40 -- pixels a window grows or shrinks with each press
for direction, by in pairs({ left = { -step, 0 }, right = { step, 0 }, up = { 0, -step }, down = { 0, step } }) do
    hl.bind(mod .. " + SHIFT + " .. direction, hl.dsp.window.move({ direction = direction }))
    hl.bind(mod .. " + CTRL + " .. direction, hl.dsp.window.resize({ x = by[1], y = by[2], relative = true }),
        { repeating = true })
end
