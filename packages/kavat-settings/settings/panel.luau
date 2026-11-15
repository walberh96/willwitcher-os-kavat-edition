-- The system's settings, drawn as the design's mockup draws them: the pages on the left,
-- with a search over them, and each page in two columns with its file at the bottom.
--
-- Each page shows what its own program says and changes it by running that program:
-- kavat-theme for the look, kav for the packages, kavat-notes for the notes. Nothing is
-- kept here: the files those programs write are the truth, so a value changed by hand
-- shows the next time the panel opens. A page whose piece of the system is not built yet
-- says so, and a control with nothing behind it yet is there but cannot be pressed.

local PALETTES = "/usr/share/kavat/themes/palettes"

-- The pages, in the mockup's order: its name, what it is about when that is a program,
-- and, for the ones still to come, what they wait for
local PAGES = {
  { id = "appearance", name = "Appearance" },
  { id = "wallpaper", name = "Wallpaper" },
  { id = "accessibility", name = "Accessibility" },
  { id = "windows", name = "Windows", about = "Hyprland" },
  { id = "keys", name = "Keybindings" },
  { id = "displays", name = "Displays" },
  { id = "gpu", name = "GPU", waits = "the graphics card's controls" },
  { id = "sound", name = "Sound" },
  { id = "network", name = "Network", about = "& Bluetooth" },
  { id = "terminal", name = "Terminal", about = "Kitty" },
  { id = "editor", name = "Editor", about = "Neovim" },
  { id = "files", name = "Files", about = "Yazi" },
  { id = "notes", name = "Notes", about = "nb" },
  { id = "shell", name = "Shell", about = "zsh", waits = "the terminal shell's settings page" },
  { id = "browser", name = "Browser", waits = "the browser" },
  { id = "bar", name = "Bar & widgets" },
  { id = "modes", name = "Modes", waits = "the system modes" },
  { id = "projects", name = "Projects", waits = "the projects" },
  { id = "agent", name = "Agent & voice", waits = "the local agent" },
  { id = "skills", name = "Skills", waits = "the agent's skills" },
  { id = "packages", name = "Packages", about = "kav" },
  { id = "backup", name = "Backup & sync", waits = "backups and sync" },
  { id = "privacy", name = "Privacy & network", waits = "the privacy page" },
  { id = "boot", name = "Boot", about = "GRUB" },
  { id = "login", name = "Login & lock" },
  { id = "language", name = "Language & region" },
}

local page = "appearance"
local search = ""
local theme = {}       -- the theme's settings in force, by name
local palettes = {}    -- each palette: id, name, its accents in order, and a few of its colours
local said = ""        -- why the last change was refused
local busy = false
local packages = { installed = "…", updates = "…" }
local due = "…"
local boot_said = ""
local conf = {}        -- what kavat-conf says each page's settings are, by page and name
local shortcuts = {}   -- the desktop's shortcuts: keys, what they do, their group
local displays = {}    -- the screens, as Hyprland lists them

local render

local function trim(text)
  return (text:gsub("^%s+", ""):gsub("%s+$", ""))
end

local function index_of(list, value)
  for index, item in ipairs(list) do
    if item == value then
      return index
    end
  end
  return 1
end

-- A palette file: its name, its accents in the order written, and the colours of each mode
local function read_palette(id, text)
  local palette = { id = id, name = id, accents = {}, dark = {}, light = {} }
  local section = ""
  for line in text:gmatch("[^\n]+") do
    local header = line:match("^%[(%w+)%]")
    local key, value = line:match('^(%w+)%s*=%s*"([^"]*)"')
    if header then
      section = header
    elseif key and section == "" and key == "name" then
      palette.name = value
    elseif key and section == "accents" then
      palette.accents[#palette.accents + 1] = { name = key, colour = value }
    elseif key and (section == "dark" or section == "light") then
      palette[section][key] = value
    end
  end
  return palette
end

local function read_palettes()
  palettes = {}
  local files = noctalia.listDir(PALETTES) or {}
  table.sort(files)
  for _, file in ipairs(files) do
    local id = file:match("^(.*)%.toml$")
    local text = id and noctalia.readFile(PALETTES .. "/" .. file)
    if text then
      local palette = read_palette(id, text)
      -- the system's own goes first, as in the design
      table.insert(palettes, id == "kavat" and 1 or #palettes + 1, palette)
    end
  end
end

local function read_theme()
  noctalia.runAsync({ "kavat-theme", "get" }, function(result)
    theme = {}
    for line in result.stdout:gmatch("[^\n]+") do
      local key, value = line:match("^(%S+) = (.*)$")
      if key then
        theme[key] = (trim(value):gsub('^"(.*)"$', "%1"))
      end
    end
    render()
  end)
end

-- Change settings of the theme; kavat-theme checks them and tells every app
local function set(...)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-theme", "set", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    end
    read_theme()
  end)
end

-- The mode: light, dark, or by the hour, which a small program of the theme's looks after
local function set_mode(mode)
  if mode == "auto" then
    set("auto.follow", "true")
  else
    set("auto.follow", "false", "mode", mode)
  end
  noctalia.runAsync("sleep 1; /usr/lib/kavat/theme-auto", function() read_theme() end)
end

local function read_packages()
  noctalia.runAsync("pacman -Qq | wc -l; pacman -Qu 2>/dev/null | wc -l", function(result)
    local installed, updates = result.stdout:match("(%d+)%s+(%d+)")
    packages = { installed = installed or "?", updates = updates or "?" }
    render()
  end)
end

local function read_notes()
  noctalia.runAsync("kavat-notes due | sed 's/\\x1b\\[[0-9;]*m//g'", function(result)
    due = trim(result.stdout) ~= "" and result.stdout or "Nothing is due."
    render()
  end)
end

-- The settings an app keeps in its own file are read and changed by kavat-conf, which
-- knows the file, how a line of it is written and what each setting may be.
local function read_conf(name)
  noctalia.runAsync({ "kavat-conf", "show", name }, function(result)
    conf[name] = {}
    for line in result.stdout:gmatch("[^\n]+") do
      local key, value = line:match("^(%S+) (.*)$")
      if key then
        conf[name][key] = value
      end
    end
    render()
  end)
end

local function set_conf(name, key, value)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-conf", "set", name, key, value }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    end
    read_conf(name)
  end)
end

local function read_shortcuts()
  shortcuts = {}
  local text = noctalia.readFile("/usr/share/kavat/hypr/shortcuts.tsv") or ""
  for line in text:gmatch("[^\n]+") do
    local keys, what, group = line:match("^([^#\t][^\t]*)\t([^\t]*)\t(.*)$")
    if keys then
      shortcuts[#shortcuts + 1] = { keys = keys, what = what, group = group }
    end
  end
end

-- The pieces every page is made of ------------------------------------------------------

local function section(text)
  return ui.label({ text = text:upper(), fontSize = 11, fontWeight = "medium", color = "on_surface_variant" })
end

-- One setting: its name on the left, its control on the right
local function row(name, control)
  return ui.row({ key = "row-" .. name, align = "center", gap = 12, minHeight = 34 }, {
    ui.column({ width = 150, minWidth = 150 }, { ui.label({ text = name, color = "on_surface" }) }),
    control,
  })
end

-- A choice of a few, side by side. Each is { text, on, pick }; one with no pick cannot be pressed.
local function segmented(choices)
  local buttons = {}
  for _, choice in ipairs(choices) do
    buttons[#buttons + 1] = ui.button({ key = choice.text, text = choice.text, controlSize = "sm", flexGrow = 1,
      variant = choice.on and "secondary" or "ghost", selected = choice.on,
      enabled = choice.pick ~= nil and not busy, onClick = choice.pick })
  end
  return ui.row({ gap = 2, padding = 2, radius = 8, fill = "surface_container", flexGrow = 1 }, buttons)
end

local function slider(key, min, max, step)
  return ui.slider({ min = min, max = max, step = step, value = tonumber(theme[key]) or min, flexGrow = 1,
    controlSize = "sm", enabled = not busy, onDragEnd = function(now) set(key, tostring(now)) end })
end

local function toggle(key)
  return ui.row({ justify = "end", flexGrow = 1 }, {
    ui.toggle({ checked = theme[key] == "true", enabled = not busy, onChange = function(now) set(key, now) end }),
  })
end

local function select(options, selected, pick)
  return ui.select({ options = options, selectedIndex = selected - 1, flexGrow = 1, controlSize = "sm",
    enabled = pick ~= nil and not busy, onChange = pick and function(index) pick(tonumber(index) + 1) end })
end

local function note(text)
  return ui.label({ text = text, color = "on_surface_variant", fontSize = 12, maxWidth = 440 })
end

-- A setting kept by kavat-conf, as a switch: on is one value in the file and off another
local function conf_toggle(name, key, on, off)
  local now = (conf[name] or {})[key]
  return ui.row({ justify = "end", flexGrow = 1 }, {
    ui.toggle({ checked = now == on, enabled = not busy and now ~= nil,
      onChange = function(checked) set_conf(name, key, checked == "true" and on or off) end }),
  })
end

-- The same, as a choice of a few side by side; each choice is { value, text }
local function conf_choice(name, key, choices)
  local now, list = (conf[name] or {})[key], {}
  for _, choice in ipairs(choices) do
    list[#list + 1] = { text = choice[2], on = now == choice[1], pick = function() set_conf(name, key, choice[1]) end }
  end
  return segmented(list)
end

-- And as a list to pick from
local function conf_select(name, key, choices)
  local now, values, texts = (conf[name] or {})[key], {}, {}
  for _, choice in ipairs(choices) do
    values[#values + 1] = choice[1]
    texts[#texts + 1] = choice[2]
  end
  return select(texts, index_of(values, now), function(index) set_conf(name, key, values[index]) end)
end

-- The pages ---------------------------------------------------------------------------

-- A palette as a card: its background, a second surface and its accent, then its name
local function palette_card(palette)
  local colours = palette[theme.mode == "light" and "light" or "dark"]
  local accent = palette.accents[1] and palette.accents[1].colour or "#888888"
  local on = palette.id == theme.palette
  return ui.column({ key = "palette-" .. palette.id, gap = 6, padding = 8, radius = 10, width = 112,
    fill = on and "surface_container_high" or "surface_container", border = on and "primary" or "outline_variant",
    borderWidth = on and 2 or 1, tooltip = palette.name,
    -- a palette has its own accents: the one in force may not be among them
    onClick = function() if not busy then set("palette", palette.id, "accent", "default") end end }, {
    ui.row({ gap = 6, justify = "center" }, {
      ui.box({ width = 50, height = 40, radius = 6, fill = colours.surface or "#151821", border = accent, borderWidth = 1 }),
      ui.column({ gap = 4 }, {
        ui.box({ width = 28, height = 18, radius = 4, fill = colours.surface2 or "#1e2230" }),
        ui.box({ width = 28, height = 18, radius = 4, fill = accent }),
      }),
    }),
    ui.label({ text = palette.name, fontSize = 12, fontWeight = "medium" }),
  })
end

local function appearance()
  local cards, current = {}, nil
  local line = {}
  for _, palette in ipairs(palettes) do
    line[#line + 1] = palette_card(palette)
    if #line == 4 then
      cards[#cards + 1] = ui.row({ gap = 10 }, line)
      line = {}
    end
    if palette.id == theme.palette then
      current = palette
    end
  end
  if #line > 0 then
    cards[#cards + 1] = ui.row({ gap = 10 }, line)
  end

  -- the accents of the palette in force, as dots; "default" is its first
  local dots = {}
  for index, accent in ipairs(current and current.accents or {}) do
    local on = theme.accent == accent.name or (theme.accent == "default" and index == 1)
    dots[#dots + 1] = ui.box({ key = "accent-" .. accent.name, width = 34, height = 34, radius = 17, fill = accent.colour,
      border = on and "on_surface" or accent.colour, borderWidth = 3, tooltip = accent.name,
      onClick = function() if not busy then set("accent", accent.name) end end })
  end

  local faces = { "Inter", "Noto Sans" }
  local mono = { "JetBrains Mono", "JetBrains Mono Nerd Font" }
  local scales = { "0.9", "1.0", "1.1", "1.25", "1.5" }
  local scale_names = { "90%", "100%", "110%", "125%", "150%" }
  local scale = string.format("%.2f", tonumber(theme["fonts.scale"]) or 1):gsub("0$", "")
  local dark, tone = theme.mode == "dark", theme.tone
  local auto = theme["auto.follow"] == "true"
  local style, size = theme["icons.style"] or "colour", theme["cursor.size"] or "24"

  local left = {
    section("Palette"),
    ui.column({ gap = 10 }, cards),
    section("Mode"),
    ui.row({}, { segmented({
      { text = "Light", on = not dark and not auto, pick = function() set_mode("light") end },
      { text = "Dark", on = dark and not auto, pick = function() set_mode("dark") end },
      { text = "Auto", on = auto, pick = function() set_mode("auto") end },
    }) }),
    note(auto and ("Light from " .. (theme["auto.light_from"] or "7") .. ":00, dark from " .. (theme["auto.dark_from"] or "19") .. ":00. The hours are in the theme's file.") or ""),
    section("Accent"),
    ui.row({ gap = 8 }, dots),
    section("Icons and cursor"),
    row("Icon style", segmented({
      { text = "Colour", on = style == "colour", pick = function() set("icons.style", "colour") end },
      { text = "Tinted", on = style == "tinted", pick = function() set("icons.style", "tinted") end },
      { text = "Line", on = style == "line", pick = function() set("icons.style", "line") end },
    })),
    row("Cursor size", segmented({
      { text = "24", on = size == "24", pick = function() set("cursor.size", "24") end },
      { text = "32", on = size == "32", pick = function() set("cursor.size", "32") end },
      { text = "48", on = size == "48", pick = function() set("cursor.size", "48") end },
    })),
    note("Colour leaves each app its own colours and puts the accent on the folders. Tinted paints every app's icon with the palette. Line draws the folders, and the apps that have one, as outlines."),
  }
  local right = {
    section("Tone & contrast"),
    row("Tone", segmented({
      { text = "Neutral", on = tone == "neutral", pick = function() set("tone", "neutral") end },
      { text = "Warm", on = tone == "warm", pick = function() set("tone", "warm") end },
      { text = "Cool", on = tone == "cool", pick = function() set("tone", "cool") end },
    })),
    row("Contrast", segmented({
      { text = "Normal", on = theme.contrast ~= "high", pick = function() set("contrast", "normal") end },
      { text = "High", on = theme.contrast == "high", pick = function() set("contrast", "high") end },
    })),
    row("OLED black", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = tone == "oled", enabled = not busy and dark,
        onChange = function(now) set("tone", now == "true" and "oled" or "neutral") end }),
    })),
    section("Typography"),
    row("Interface", select(faces, index_of(faces, theme["fonts.ui"]), function(index) set("fonts.ui", faces[index]) end)),
    row("Monospace", select(mono, index_of(mono, theme["fonts.mono"]), function(index) set("fonts.mono", mono[index]) end)),
    row("Scale", select(scale_names, index_of(scales, scale), function(index) set("fonts.scale", scales[index]) end)),
    section("Profiles"),
    ui.row({ gap = 6 }, {
      ui.button({ text = "work", controlSize = "sm", variant = "outline", enabled = false }),
      ui.button({ text = "night", controlSize = "sm", variant = "outline", enabled = false }),
      ui.button({ text = "+ Save current", controlSize = "sm", variant = "outline", enabled = false }),
    }),
    note("Profiles, and switching to one at sunset, are not built yet."),
  }
  return { title = "Appearance", sub = "one theme, every app", left = left, right = right, file = "~/.config/kavat/theme.toml" }
end

local function windows()
  local borders = { "0 px", "1 px", "2 px", "3 px", "4 px" }
  local speeds, speed_names = { "0.5", "1.0", "1.5" }, { "Slow", "Normal", "Fast" }
  local speed = tonumber(theme["effects.animations"]) or 1
  local moving = speed > 0
  local left = {
    section("Layout"),
    row("Gaps", slider("shape.gap", 0, 24, 1)),
    row("Gap at the edge", slider("shape.edge", 0, 48, 1)),
    row("Corner radius", slider("shape.radius", 0, 24, 1)),
    row("Border width", select(borders, (tonumber(theme["shape.border"]) or 2) + 1, function(index) set("shape.border", tostring(index - 1)) end)),
    note("Tiling, the keyboard and the mouse, and the window rules are not on this page yet: they are in Hyprland's own file."),
  }
  local right = {
    section("Effects"),
    row("Blur", toggle("effects.blur")),
    row("Transparency", toggle("effects.transparency")),
    row("Animations", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = moving, enabled = not busy, onChange = function(now) set("effects.animations", now == "true" and "1.0" or "0") end }),
    })),
    row("Animation speed", select(speed_names, moving and index_of(speeds, string.format("%.1f", speed)) or 2,
      moving and function(index) set("effects.animations", speeds[index]) end or nil)),
    row("Shadows", toggle("effects.shadows")),
  }
  return { title = "Windows", sub = "Hyprland · changes apply live", left = left, right = right, file = "~/.config/kavat/theme.toml" }
end

local function boot_button()
  return ui.row({ gap = 8 }, {
    ui.button({ text = "Give them the look in force", variant = "primary", enabled = not busy,
      onClick = function()
        busy = true
        boot_said = "Working…"
        render()
        noctalia.runAsync({ "kavat-theme", "boot" }, function(result)
          busy = false
          boot_said = result.exitCode == 0 and "Done: the next boot wears it." or trim(result.stderr ~= "" and result.stderr or result.stdout)
          render()
        end)
      end }),
  })
end

local function boot()
  return { title = "Boot", sub = "GRUB and the loading screen", file = "/etc/kavat/theme.toml", left = {
    section("Look"),
    note("The boot menu and the loading screen are shown before anybody logs in, so they have a look of their own, kept with the system. This gives them, and the login screen, the look in force now. It asks for your password."),
    boot_button(),
    ui.label({ text = boot_said, color = "on_surface_variant", maxWidth = 440 }),
  }, right = {
    section("Menu"),
    note("The timeout, the default entry and which entries show are not on this page yet."),
  } }
end

local MINUTES = { { "0", "Never" }, { "1", "1 minute" }, { "5", "5 minutes" }, { "10", "10 minutes" }, { "15", "15 minutes" },
  { "30", "30 minutes" }, { "60", "1 hour" } }

local function login()
  return { title = "Login & lock", sub = "kavat-greeter · hyprlock", file = "/etc/kavat/theme.toml", left = {
    section("Look"),
    note("The login screen takes its look with the boot menu and the loading screen. It asks for your password."),
    boot_button(),
    ui.label({ text = boot_said, color = "on_surface_variant", maxWidth = 440 }),
  }, right = {
    section("Lock"),
    row("Lock after", conf_select("lock", "lock_after", MINUTES)),
    row("Screen off after", conf_select("lock", "screen_off_after", MINUTES)),
    note("Super+Shift+L locks at once. The lock screen wears the theme, with the same picture as the login."),
  } }
end

local function pages_packages()
  local terminal = function(command)
    return function()
      noctalia.runInTerminal(command)
      panel.close()
    end
  end
  return { title = "Packages", sub = "kav · the AUR only through the reviewed list", file = "/etc/kav/allowlist.toml", left = {
    section("Now"),
    row("Installed", ui.label({ text = packages.installed, fontWeight = "bold" })),
    row("Updates waiting", ui.label({ text = packages.updates, fontWeight = "bold" })),
    note("By the lists last fetched: updating fetches them again."),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Open kav", variant = "primary", onClick = terminal("kav") }),
      ui.button({ text = "Update the system", variant = "outline", onClick = terminal("kav update; echo; echo 'Enter closes this window.'; read _") }),
    }),
  }, right = {
    section("Health"),
    note("kav doctor looks at failed services, packages nothing needs, settings files left to merge, the disk and the snapshots."),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Check it", variant = "outline", onClick = terminal("kav doctor; echo; echo 'Enter closes this window.'; read _") }),
    }),
  } }
end

local function notes()
  local terminal = function(command)
    return function()
      noctalia.runInTerminal(command)
      panel.close()
    end
  end
  return { title = "Notes", sub = "nb · Markdown files in ~/Notes", file = "~/Notes", left = {
    section("Due"),
    ui.label({ text = due, fontFamily = "JetBrains Mono", fontSize = 12, maxWidth = 440 }),
  }, right = {
    section("Open"),
    ui.row({ gap = 8 }, {
      ui.button({ text = "The notes", variant = "primary", onClick = terminal("kavat-notes") }),
      ui.button({ text = "A quick note", variant = "outline", onClick = terminal("kavat-notes quick") }),
    }),
    note("Super+N saves a note from a single line; a day and a time at its end set a reminder. The folder, the default notebook and how long a reminder is put off are not on this page yet."),
  } }
end

local function terminal()
  local padding = tonumber((conf.terminal or {}).window_padding_width) or 0
  return { title = "Terminal", sub = "Kitty · changes apply live", file = "~/.config/kitty/kitty.conf", left = {
    section("Cursor"),
    row("Shape", conf_choice("terminal", "cursor_shape", { { "block", "Block" }, { "beam", "Beam" }, { "underline", "Underline" } })),
    row("Blinks", conf_toggle("terminal", "cursor_blink_interval", "-1", "0")),
    section("Window"),
    row("Margin", ui.slider({ min = 0, max = 24, step = 1, value = padding, flexGrow = 1, controlSize = "sm", enabled = not busy,
      onDragEnd = function(now) set_conf("terminal", "window_padding_width", tostring(math.floor(tonumber(now) + 0.5))) end })),
    row("Tabs", conf_select("terminal", "tab_bar_style", { { "fade", "Fade" }, { "separator", "Separator" }, { "powerline", "Powerline" }, { "hidden", "Hidden" } })),
    note("The font, its size and how much the background lets through come from Appearance."),
  }, right = {
    section("History"),
    row("Lines kept", conf_select("terminal", "scrollback_lines", { { "2000", "2,000" }, { "10000", "10,000" }, { "50000", "50,000" }, { "200000", "200,000" } })),
    section("Behaviour"),
    row("Bell", conf_toggle("terminal", "enable_audio_bell", "yes", "no")),
    row("Ask before closing", conf_toggle("terminal", "confirm_os_window_close", "-1", "0")),
  } }
end

local function editor()
  return { title = "Editor", sub = "Neovim · from the next time it opens", file = "~/.config/kavat/editor.json", left = {
    section("Typing"),
    row("Opens ready to type", conf_toggle("editor", "start_typing", "true", "false")),
    row("Format on saving", conf_toggle("editor", "format_on_save", "true", "false")),
    row("Indent", conf_choice("editor", "indent", { { "2", "2" }, { "4", "4" }, { "8", "8" } })),
  }, right = {
    section("View"),
    row("Wrap long lines", conf_toggle("editor", "wrap", "true", "false")),
    row("Relative line numbers", conf_toggle("editor", "relative_numbers", "true", "false")),
    note("The shortcut presets, the language packs, the plugins and the side panels are not on this page yet. Your own init.lua has the last word."),
  } }
end

local function files()
  return { title = "Files", sub = "Yazi · from the next time it opens", file = "~/.config/yazi/yazi.toml", left = {
    section("List"),
    row("Hidden files", conf_toggle("files", "mgr.show_hidden", "true", "false")),
    row("Folders first", conf_toggle("files", "mgr.sort_dir_first", "true", "false")),
    row("Sort by", conf_select("files", "mgr.sort_by", { { "natural", "Name" }, { "alphabetical", "Name, letter by letter" }, { "mtime", "Date changed" }, { "size", "Size" }, { "extension", "Kind" } })),
    row("Beside each name", conf_select("files", "mgr.linemode", { { "none", "Nothing" }, { "size", "Size" }, { "mtime", "Date changed" }, { "permissions", "Permissions" } })),
  }, right = {
    section("The window apps pick files with"),
    note("Every app opens and saves files with Yazi, in a floating window. Dragging is Yazi's own, with the mouse."),
    note("The copy window, what a download does and the apps each kind of file opens with are not on this page yet."),
  } }
end

-- One of the shell's own panels, opened in the place of this one
local function shell_panel(text, what)
  return ui.button({ text = text, variant = "outline", onClick = function()
    panel.close()
    noctalia.runAsync("kavat-shell msg " .. what)
  end })
end

local function wallpaper()
  return { title = "Wallpaper", sub = "the desktop's background", file = "~/.config/kavat/theme.toml", left = {
    section("The theme's picture"),
    note("The background is the same picture as the boot and the login, in the colours of the theme, and changes with it."),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Use the theme's picture", variant = "primary", onClick = function()
        -- with no picture of your own in place, the theme's is put back
        noctalia.runAsync("kavat-shell msg wallpaper-set \"$(ls ${XDG_STATE_HOME:-$HOME/.local/state}/kavat/theme/current/wallpaper/*.svg | head -n 1)\"")
      end }),
    }),
  }, right = {
    section("A picture of your own"),
    note("A picture you choose stays when the theme changes."),
    ui.row({ gap = 8 }, { shell_panel("Choose a picture", "panel-toggle wallpaper") }),
    note("Video backgrounds and a picture for each screen are not built yet."),
  } }
end

local function accessibility()
  local scales, scale_names = { "1.0", "1.1", "1.25", "1.5" }, { "100%", "110%", "125%", "150%" }
  local scale = string.format("%.2f", tonumber(theme["fonts.scale"]) or 1):gsub("0$", "")
  local still = (tonumber(theme["effects.animations"]) or 1) == 0
  return { title = "Accessibility", sub = "easier to look at", file = "~/.config/kavat/theme.toml", left = {
    section("Motion"),
    row("Reduce motion", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = still, enabled = not busy, onChange = function(now) set("effects.animations", now == "true" and "0" or "1.0") end }),
    })),
    section("Text"),
    row("Text scale", select(scale_names, index_of(scales, scale), function(index) set("fonts.scale", scales[index]) end)),
  }, right = {
    section("Contrast"),
    row("High contrast", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = theme.contrast == "high", enabled = not busy, onChange = function(now) set("contrast", now == "true" and "high" or "normal") end }),
    })),
    row("Bigger cursor", segmented({
      { text = "24", on = (theme["cursor.size"] or "24") == "24", pick = function() set("cursor.size", "24") end },
      { text = "32", on = theme["cursor.size"] == "32", pick = function() set("cursor.size", "32") end },
      { text = "48", on = theme["cursor.size"] == "48", pick = function() set("cursor.size", "48") end },
    })),
    note("These are the theme's own settings, also on Appearance: here they are together."),
  } }
end

local function sound()
  return { title = "Sound", sub = "PipeWire", left = {
    section("Volume and devices"),
    note("The output, the input and the volume of each app are in the shell's control center."),
    ui.row({ gap = 8 }, { shell_panel("Open the sound controls", "panel-toggle control-center audio") }),
  }, right = {
    section("Not built yet"),
    note("Noise suppression for the microphone."),
  } }
end

local function network()
  return { title = "Network", sub = "NetworkManager · BlueZ", left = {
    section("Connections"),
    note("Cable, Wi-Fi and VPNs are in the shell's control center."),
    ui.row({ gap = 8 }, { shell_panel("Open the network", "panel-toggle control-center network") }),
  }, right = {
    section("Bluetooth"),
    note("Devices and their battery."),
    ui.row({ gap = 8 }, { shell_panel("Open Bluetooth", "panel-toggle control-center bluetooth") }),
  } }
end

local function bar()
  return { title = "Bar & widgets", sub = "Kavat Shell", file = "~/.config/kavat-shell/config.toml", left = {
    section("The bar, the launcher and the notifications"),
    note("Where the bar is, its widgets and their order, the launcher and the notifications are in the shell's own settings window."),
    ui.row({ gap = 8 }, { shell_panel("Open the shell's settings", "settings-open") }),
  }, right = {
    section("Mind the theme"),
    note("The shell's settings have a page for colours too. The system's colours are set on Appearance: what is changed there is put back at the next change of theme."),
  } }
end

local function language()
  local first = (conf.language or {}).keyboard_first
  return { title = "Language & region", sub = "the keyboard", file = "~/.config/kavat/desktop.json", left = {
    section("Keyboard"),
    row("The one in use first", segmented({
      { text = "English (US)", on = first == "us", pick = function() set_conf("language", "keyboard_first", "us") end },
      { text = "Spanish", on = first == "es", pick = function() set_conf("language", "keyboard_first", "es") end },
    })),
    note("Alt+Shift goes to the other one and back. The system has these two keyboards."),
  }, right = {
    section("Language"),
    note("The system's own texts are in English. The language of the other apps, the formats of dates and numbers and the time zone are set when the system is installed; changing them here is not built yet."),
  } }
end

local function read_displays()
  noctalia.runAsync("hyprctl monitors 2>/dev/null || hyprctl --instance 0 monitors", function(result)
    displays = {}
    local current
    for line in result.stdout:gmatch("[^\n]+") do
      local name = line:match("^Monitor (%S+)")
      if name then
        current = { name = name, mode = "", scale = "", what = "" }
        displays[#displays + 1] = current
      elseif current then
        current.mode = line:match("^%s+(%d+x%d+@[%d.]+)") or current.mode
        current.scale = line:match("^%s+scale: ([%d.]+)") or current.scale
        current.what = line:match("^%s+description: (.*)") or current.what
      end
    end
    render()
  end)
end

local function pages_displays()
  local list = {}
  for _, display in ipairs(displays) do
    local size, rate = display.mode:match("^(%d+x%d+)@([%d.]+)")
    list[#list + 1] = ui.column({ key = "display-" .. display.name, gap = 2, padding = 12, radius = 10, fill = "surface_container" }, {
      ui.label({ text = display.name, fontWeight = "bold" }),
      ui.label({ text = display.what, color = "on_surface_variant", fontSize = 12, maxWidth = 420 }),
      ui.label({ text = (size or "?") .. " · " .. string.format("%.0f", tonumber(rate) or 0) .. " Hz · scale " .. display.scale, fontFamily = "JetBrains Mono", fontSize = 12 }),
    })
  end
  if #list == 0 then
    list[1] = note("Hyprland lists no screen.")
  end
  return { title = "Displays", sub = "Hyprland", file = "/usr/share/kavat/hypr/hyprland.lua", left = { section("Connected now"), ui.column({ gap = 8 }, list) }, right = {
    section("Not built yet"),
    note("Placing the screens by dragging them, the resolution, the refresh rate, the scale and the rotation of each, and night light. Today every screen is used at its own best mode, side by side."),
  } }
end

local function keys()
  -- the shortcuts in two columns, group by group; each group is a tight list of its own
  local columns, rows, last = { {}, {} }, nil, nil
  local half = 0
  for index, shortcut in ipairs(shortcuts) do
    if shortcut.group ~= last then
      -- a group goes whole to the second column once the first has half of them
      local column = columns[half * 2 < #shortcuts and 1 or 2]
      rows = {}
      column[#column + 1] = section(shortcut.group)
      column[#column + 1] = ui.column({ key = "group-" .. shortcut.group, gap = 2 }, rows)
      last = shortcut.group
    end
    rows[#rows + 1] = ui.row({ key = "key-" .. shortcut.keys, align = "center", gap = 12, minHeight = 24 }, {
      ui.column({ width = 180, minWidth = 180 }, { ui.label({ text = shortcut.keys, fontFamily = "JetBrains Mono", fontSize = 12, color = "primary" }) }),
      ui.label({ text = shortcut.what, fontSize = 13 }),
    })
    half = index
  end
  return { title = "Keybindings", sub = "the desktop's shortcuts · Super+/ opens this list", file = "/usr/share/kavat/hypr/hyprland.lua",
    left = columns[1], right = columns[2] }
end

local BUILT = { appearance = appearance, wallpaper = wallpaper, accessibility = accessibility, windows = windows, keys = keys,
  displays = pages_displays, sound = sound, network = network, terminal = terminal, editor = editor, files = files, notes = notes,
  bar = bar, packages = pages_packages, boot = boot, login = login, language = language }

local function waiting(entry)
  return { title = entry.name, sub = entry.about or "", left = {
    note("This page is not built yet. It comes with " .. entry.waits .. "."),
  }, right = {} }
end

-- The panel ---------------------------------------------------------------------------

function render()
  local nav = {
    ui.input({ key = "search", placeholder = "Search settings…", controlSize = "sm", value = search,
      onChange = function(text) search = text:lower() render() end }),
  }
  local shown
  for _, entry in ipairs(PAGES) do
    if entry.id == page then
      shown = entry
    end
    if search == "" or (entry.name .. " " .. (entry.about or "")):lower():find(search, 1, true) then
      nav[#nav + 1] = ui.row({ key = "nav-" .. entry.id, align = "center", paddingH = 12, minHeight = 26, radius = 6,
        fill = entry.id == page and "surface_container_highest" or "surface_container_low", onClick = function() open_page(entry.id) end }, {
        ui.label({ text = entry.name, flexGrow = 1, fontWeight = entry.id == page and "medium" or "regular",
          color = BUILT[entry.id] and "on_surface" or "on_surface_variant" }),
        ui.label({ text = entry.about or "", fontSize = 11, color = "on_surface_variant" }),
      })
    end
  end

  local content = BUILT[page] and BUILT[page]() or waiting(shown or PAGES[1])
  local body = {
    ui.row({ align = "end", gap = 14 }, {
      ui.label({ text = content.title, fontSize = 24, fontWeight = "bold" }),
      ui.label({ text = content.sub or "", color = "on_surface_variant", fontSize = 13 }),
    }),
  }
  if said ~= "" then
    body[#body + 1] = ui.label({ text = said, color = "error", maxWidth = 900 })
  end
  body[#body + 1] = ui.scroll({ key = "page-" .. page, flexGrow = 1 }, {
    ui.row({ gap = 36, align = "start" }, {
      ui.column({ gap = 12, flexGrow = 1 }, content.left),
      ui.column({ gap = 12, flexGrow = 1 }, content.right),
    }),
  })
  if content.file then
    body[#body + 1] = ui.row({ align = "center", gap = 12 }, {
      ui.button({ text = "Open config in Neovim", variant = "primary", controlSize = "sm",
        onClick = function()
          noctalia.runInTerminal("${EDITOR:-nvim} " .. content.file)
          panel.close()
        end }),
      ui.label({ text = content.file, fontFamily = "JetBrains Mono" }),
      ui.label({ text = "two-way with this file", color = "on_surface_variant" }),
      ui.spacer({ flexGrow = 1 }),
      ui.button({ text = "Undo", variant = "outline", controlSize = "sm", enabled = false }),
      ui.button({ text = "History", variant = "outline", controlSize = "sm", enabled = false }),
      ui.button({ text = "Export", variant = "outline", controlSize = "sm", enabled = false }),
    })
  end

  panel.render(ui.row({ gap = 0 }, {
    ui.scroll({ gap = 1, padding = 14, width = 228, fill = "surface_container_low" }, nav),
    ui.column({ gap = 18, padding = 30, flexGrow = 1 }, body),
  }))
end

function open_page(id)
  page = id
  said = ""
  render()
  if id == "appearance" or id == "windows" or id == "accessibility" then
    read_theme()
  elseif id == "language" then
    read_conf("language")
  elseif id == "displays" then
    read_displays()
  elseif id == "packages" then
    read_packages()
  elseif id == "notes" then
    read_notes()
  elseif id == "terminal" or id == "editor" or id == "files" then
    read_conf(id)
  elseif id == "login" then
    read_conf("lock")
  elseif id == "keys" then
    read_shortcuts()
    render()
  end
end

-- The page can be named when the panel is opened: panel-open kavat/settings:panel notes
function onOpen(context)
  boot_said = ""
  search = ""
  read_palettes()
  local wanted = page
  for _, entry in ipairs(PAGES) do
    if entry.id == context then
      wanted = context
    end
  end
  open_page(wanted)
end
