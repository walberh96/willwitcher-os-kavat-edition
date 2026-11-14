-- Part of kavat-greeter: the small Hyprland session that the login screen is drawn on.
-- It has no desktop: the page fills every screen, and when the page's program ends, so
-- does the session.

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })

hl.config({
    general    = { gaps_in = 0, gaps_out = 0, border_size = 0 },
    decoration = { rounding = 0 },
    animations = { enabled = false },
    -- the keyboards the login screen offers, in the order kavat-greeter names them
    input      = { kb_layout = "us,es" },
    misc       = { disable_hyprland_logo = true, disable_splash_rendering = true },
    -- the pointer stays out of the way until it is moved
    cursor     = { inactive_timeout = 1, hide_on_key_press = true },
    -- nothing here is an X program
    xwayland   = { enabled = false },
    ecosystem  = { no_update_news = true, no_donation_nag = true },
})

-- What shows for the instant before the page is drawn: the colour of the theme, when
-- kavat-theme has made the login screen's look.
local found, look = pcall(dofile, "/var/lib/kavat/theme/current/greeter/look.lua")
if found and type(look) == "table" then
    hl.config(look)
end

hl.on("hyprland.start", function()
    hl.exec_cmd("kavat-greeter; hyprctl dispatch exit")
end)
