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
  { id = "wallpaper", name = "Wallpaper", waits = "the wallpapers, images and video" },
  { id = "accessibility", name = "Accessibility", waits = "the accessibility settings" },
  { id = "windows", name = "Windows", about = "Hyprland" },
  { id = "keys", name = "Keybindings", waits = "the shortcut editor" },
  { id = "displays", name = "Displays", waits = "the displays page" },
  { id = "gpu", name = "GPU", waits = "the graphics card's controls" },
  { id = "sound", name = "Sound", waits = "the sound page; the shell's own control center has the volume today" },
  { id = "network", name = "Network", about = "& Bluetooth", waits = "the network page; the shell's own control center has it today" },
  { id = "terminal", name = "Terminal", about = "Kitty", waits = "Kitty's settings page" },
  { id = "editor", name = "Editor", about = "Neovim", waits = "the editor's settings page" },
  { id = "files", name = "Files", about = "Yazi", waits = "the files' settings page" },
  { id = "notes", name = "Notes", about = "nb" },
  { id = "shell", name = "Shell", about = "zsh", waits = "the terminal shell's settings page" },
  { id = "browser", name = "Browser", waits = "the browser" },
  { id = "bar", name = "Bar & widgets", waits = "the bar's page; the shell's own settings have it today" },
  { id = "modes", name = "Modes", waits = "the system modes" },
  { id = "projects", name = "Projects", waits = "the projects" },
  { id = "agent", name = "Agent & voice", waits = "the local agent" },
  { id = "skills", name = "Skills", waits = "the agent's skills" },
  { id = "packages", name = "Packages", about = "kav" },
  { id = "backup", name = "Backup & sync", waits = "backups and sync" },
  { id = "privacy", name = "Privacy & network", waits = "the privacy page" },
  { id = "boot", name = "Boot", about = "GRUB" },
  { id = "login", name = "Login & lock" },
  { id = "language", name = "Language & region", waits = "the language page; the installer sets it today" },
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

  local left = {
    section("Palette"),
    ui.column({ gap = 10 }, cards),
    section("Mode"),
    ui.row({}, { segmented({
      { text = "Light", on = not dark, pick = function() set("mode", "light") end },
      { text = "Dark", on = dark, pick = function() set("mode", "dark") end },
      { text = "Auto", on = false },
    }) }),
    section("Accent"),
    ui.row({ gap = 8 }, dots),
    section("Icons and cursor"),
    row("Icon style", segmented({ { text = "Colour", on = true, pick = function() end }, { text = "Tinted", on = false }, { text = "Line", on = false } })),
    row("Cursor size", segmented({ { text = "24", on = true, pick = function() end }, { text = "32", on = false }, { text = "48", on = false } })),
    note("The folders and the cursor follow the palette, the mode and the accent. The other icon styles and cursor sizes are not built yet."),
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

local function login()
  return { title = "Login & lock", sub = "kavat-greeter · hyprlock", file = "/etc/kavat/theme.toml", left = {
    section("Look"),
    note("The login screen takes its look with the boot menu and the loading screen. It asks for your password."),
    boot_button(),
    ui.label({ text = boot_said, color = "on_surface_variant", maxWidth = 440 }),
  }, right = {
    section("Lock"),
    note("The lock screen and its timings are not on this page yet."),
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

local BUILT = { appearance = appearance, windows = windows, boot = boot, login = login, packages = pages_packages, notes = notes }

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
        fill = entry.id == page and "surface_container_highest" or nil, onClick = function() open_page(entry.id) end }, {
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
  if id == "appearance" or id == "windows" then
    read_theme()
  elseif id == "packages" then
    read_packages()
  elseif id == "notes" then
    read_notes()
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
