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
  { id = "gpu", name = "GPU" },
  { id = "sound", name = "Sound" },
  { id = "network", name = "Network", about = "& Bluetooth" },
  { id = "terminal", name = "Terminal", about = "Kitty" },
  { id = "editor", name = "Editor", about = "Neovim" },
  { id = "files", name = "Files", about = "Yazi" },
  { id = "notes", name = "Notes", about = "nb" },
  { id = "shell", name = "Shell", about = "zsh" },
  { id = "browser", name = "Browser" },
  { id = "bar", name = "Bar & widgets" },
  { id = "modes", name = "Modes" },
  { id = "projects", name = "Projects" },
  { id = "agent", name = "Agent & voice", waits = "the local agent" },
  { id = "skills", name = "Skills", waits = "the agent's skills" },
  { id = "packages", name = "Packages", about = "kav" },
  { id = "backup", name = "Backup & sync" },
  { id = "privacy", name = "Privacy & network" },
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
local projects = {}  -- each: name, folder, workspace, open
local project_name = ""
local privacy = { firewall = "", doors = {}, opensnitch = "", vpn = "", dns = "off", sandbox = {} }
local games = {}  -- the game profiles, as kavat-game lists them
local browser = { default = "", widevine = "", commands = "0" }  -- what the browser page shows
local panel_open = false  -- what reads again and again stops when the panel closes
local gpu = {}  -- what the graphics card says of itself, by name
local profiles = {}      -- the looks saved under a name
local profile_name = ""  -- the name being typed for a new one
local news = ""  -- something done that is worth saying, as `said` is for what was refused
local backup = { snapshots = "0", place = "", every = "never", last = "", history = {}, remote = "" }  -- as kavat-backup and kavat-history say it
local display_chosen = ""  -- the display whose settings are in view
local display_tried = 0    -- above zero while a change of the displays waits to be kept
local walls = { list = {}, kind = "theme", file = "", sound = "off", fit = "fill", gpu = "on" }  -- the background, as kavat-wallpaper says it
local wall_filter = "all"
local modes = { list = {}, vpn = "", vpns = {}, auto = true }  -- the system's modes, as kavat-mode says them
local boot_said = ""
local conf = {}        -- what kavat-conf says each page's settings are, by page and name
local shortcuts = {}   -- the desktop's shortcuts: keys, what they do, their group
local displays = {}    -- the screens, as Hyprland lists them

local render

-- A change made here is kept in the history of the settings, with what it was
local function keep(what)
  noctalia.runAsync({ "kavat-history", "save", what })
end

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
  noctalia.runAsync({ "kavat-theme", "profile" }, function(result)
    profiles = {}
    for mark, name in result.stdout:gmatch("([* ]) ([^\n]+)") do
      profiles[#profiles + 1] = { name = name, on = mark == "*" }
    end
    render()
  end)
end

-- Change settings of the theme; kavat-theme checks them and tells every app
local function set(...)
  busy = true
  said = ""
  render()
  local what = table.concat({ ... }, " ")
  noctalia.runAsync({ "kavat-theme", "set", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    else
      keep("Theme: " .. what)
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

-- The system's modes: what there is, what each does, and what Incognito and Gamer need
local function read_modes()
  noctalia.runAsync("kavat-mode list; for mode in $(kavat-mode list | cut -c3- | cut -f1); do echo \"== $mode\"; kavat-mode show $mode; done; "
    .. "echo '== vpn'; kavat-mode vpn; echo '== vpns'; kavat-mode vpns; echo '== auto'; kavat-mode auto", function(result)
    local found, part = { list = {}, vpn = "", vpns = {}, auto = true }, nil
    for line in result.stdout:gmatch("[^\n]+") do
      local mark, word, name, about = line:match("^([* ]) ([^\t]+)\t([^\t]*)\t(.*)$")
      local heading = line:match("^== (.+)$")
      if heading then
        part = heading
      elseif part == nil and word then
        found.list[#found.list + 1] = { word = word, name = name, about = about, on = mark == "*", does = {} }
      elseif part == "vpn" then
        found.vpn = line
      elseif part == "vpns" then
        found.vpns[#found.vpns + 1] = line
      elseif part == "auto" then
        found.auto = line == "on"
      else
        for _, mode in ipairs(found.list) do
          if mode.word == part then
            mode.does[#mode.does + 1] = line
          end
        end
      end
    end
    modes = found
    render()
  end)
end

-- The background: the library, what is in force and the video's settings, as kavat-wallpaper says them
local function read_walls()
  noctalia.runAsync("kavat-wallpaper; echo '== list'; kavat-wallpaper list; echo '== option'; kavat-wallpaper option", function(result)
    local found, part = { list = {}, kind = "theme", file = "", sound = "off", fit = "fill", gpu = "on" }, nil
    for line in result.stdout:gmatch("[^\n]+") do
      local heading = line:match("^== (.+)$")
      if heading then
        part = heading
      elseif part == nil then
        found.kind, found.file = line:match("^(%S+) ?(.*)$")
      elseif part == "list" then
        local kind, file, thumb = line:match("^([^\t]+)\t([^\t]+)\t(.+)$")
        if kind then
          found.list[#found.list + 1] = { kind = kind, file = file, thumb = thumb,
            name = kind == "theme" and "Kavat" or (file:match("([^/]+)%.[^.]+$") or file) }
        end
      else
        local name, value = line:match("^(%S+) (%S+)$")
        if name then
          found[name] = value
        end
      end
    end
    walls = found
    render()
  end)
end

local function wall_does(...)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-wallpaper", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    end
    read_walls()
    read_theme()
  end)
end

-- Backups, the history of the settings and where it is sent
local function read_backup()
  noctalia.runAsync("echo '== snapshots'; ls /home/.snapshots 2>/dev/null | wc -l; echo '== backup'; kavat-backup; "
    .. "echo '== history'; kavat-history list 7; echo '== remote'; kavat-history remote", function(result)
    local found, part = { snapshots = "0", place = "", every = "never", last = "", history = {}, remote = "" }, nil
    for line in result.stdout:gmatch("[^\n]+") do
      local heading = line:match("^== (.+)$")
      if heading then
        part = heading
      elseif part == "snapshots" then
        found.snapshots = line
      elseif part == "backup" then
        local name, value = line:match("^(%a+): ?(.*)$")
        if name then
          found[name] = value
        end
      elseif part == "history" then
        local mark, when, what = line:match("^(%S+)\t([^\t]+)\t(.*)$")
        if mark then
          found.history[#found.history + 1] = { mark = mark, when = when, what = what }
        end
      elseif part == "remote" then
        found.remote = line
      end
    end
    if found.place == "none" then
      found.place = ""
    end
    backup = found
    render()
  end)
end

local function backup_does(command)
  busy = true
  said = ""
  render()
  noctalia.runAsync(command, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    end
    read_backup()
  end)
end

-- The graphics card: its sensors, read again every two seconds while its page is in view
local function read_gpu()
  noctalia.runAsync("/usr/lib/kavat/gpu-state; echo \"lact $(command -v lact > /dev/null && echo yes || echo no)\"", function(result)
    local found = {}
    for line in result.stdout:gmatch("[^\n]+") do
      local name, value = line:match("^(%S+) (.*)$")
      if name then
        found[name] = value
      end
    end
    gpu = found
    render()
    if page == "gpu" and panel_open then
      noctalia.runAsync("sleep 2", function() if page == "gpu" and panel_open then read_gpu() end end)
    end
  end)
end

local function read_browser()
  noctalia.runAsync("echo \"default $(xdg-settings get default-web-browser 2>/dev/null)\"; "
    .. "echo \"widevine $(sed -n 's/.*\"version\": *\"\\([^\"]*\\)\".*/\\1/p' /usr/lib/chromium/WidevineCdm/manifest.json 2>/dev/null | head -n 1)\"; "
    .. "echo \"commands $(grep -c '^\\[\\[command\\]\\]' ~/.config/kavat/browser/commands.toml 2>/dev/null)\"", function(result)
    local found = { default = "", widevine = "", commands = "0" }
    for line in result.stdout:gmatch("[^\n]+") do
      local name, value = line:match("^(%S+) ?(.*)$")
      if name then
        found[name] = value
      end
    end
    browser = found
    render()
  end)
end

local function read_projects()
  noctalia.runAsync({ "kavat-project" }, function(result)
    projects = {}
    for mark, name, folder, workspace in result.stdout:gmatch("([* ]) ([^\t\n]+)\t([^\t\n]*)\t([^\n]*)") do
      projects[#projects + 1] = { name = name, folder = folder, workspace = workspace, open = mark == "*" }
    end
    render()
  end)
end

local function read_privacy()
  noctalia.runAsync("echo \"firewall $(systemctl is-active kavat-firewall 2>/dev/null)\"; echo \"opensnitch $(systemctl is-active opensnitchd 2>/dev/null)\"; "
    .. "echo \"vpn $(kavat-mode vpn 2>/dev/null)\"; echo \"dns $(kavat-dns 2>/dev/null | tr '\\t' ' ')\"; kavat-sandbox 2>/dev/null | sed 's/^/sandbox /'; cat /etc/kavat/firewall.d/*.nft 2>/dev/null | grep '^add rule' | sed 's/^add rule inet kavat_firewall in /door /'", function(result)
    local found = { firewall = "", doors = {}, opensnitch = "", vpn = "", dns = "off", sandbox = {} }
    for line in result.stdout:gmatch("[^\n]+") do
      local name, value = line:match("^(%S+) ?(.*)$")
      if name == "door" then
        found.doors[#found.doors + 1] = value
      elseif name == "sandbox" then
        local state, profile, about = value:match("^(%S+)\t([^\t]+)\t?(.*)$")
        if state then
          found.sandbox[#found.sandbox + 1] = { state = state, name = profile, about = about }
        end
      elseif name then
        found[name] = value
      end
    end
    privacy = found
    render()
  end)
end

local function mode_does(...)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-mode", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    end
    read_modes()
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
    else
      keep(name:sub(1, 1):upper() .. name:sub(2) .. ": " .. key .. " " .. value)
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

local function profile_does(...)
  busy = true
  said = ""
  render()
  local what = table.concat({ ... }, " ")
  noctalia.runAsync({ "kavat-theme", "profile", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    else
      profile_name = ""
      keep("Theme: profile " .. what)
    end
    read_theme()
  end)
end

local function appearance()
  local saved = {}
  for _, profile in ipairs(profiles) do
    saved[#saved + 1] = ui.button({ key = "profile-" .. profile.name, text = profile.name, controlSize = "sm",
      variant = profile.on and "secondary" or "outline", enabled = not busy, onClick = function() profile_does(profile.name) end })
  end
  if #saved == 0 then
    saved[1] = ui.label({ text = "None saved yet.", color = "on_surface_variant", fontSize = 12 })
  end
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
    ui.row({ gap = 6, align = "center" }, saved),
    ui.row({ gap = 6, align = "center" }, {
      ui.input({ key = "profile-name", placeholder = "a name: work, night…", controlSize = "sm", width = 190, value = profile_name,
        onChange = function(text) profile_name = text end,
        onSubmit = function(text) if trim(text) ~= "" then profile_does("save", trim(text)) end end }),
      ui.button({ text = "Save current", controlSize = "sm", variant = "outline", enabled = not busy,
        onClick = function() if trim(profile_name) ~= "" then profile_does("save", trim(profile_name)) end end }),
    }),
    note("A profile is the whole look as it is now, under a name: press one to wear it. `kavat-theme profile delete <name>` forgets one."),
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

local function pages_modes()
  local cards = {}
  for _, mode in ipairs(modes.list) do
    local lines = { ui.row({ align = "center", gap = 8 }, {
      ui.label({ text = mode.name, fontWeight = "bold", fontSize = 15 }),
      ui.spacer({ flexGrow = 1 }),
      ui.button({ key = "enter-" .. mode.word, text = mode.on and "Active" or "Activate", controlSize = "sm",
        variant = mode.on and "primary" or "outline", enabled = not busy and not mode.on,
        onClick = function() mode_does(mode.word) end }),
    }), ui.label({ text = mode.about, color = "on_surface_variant", fontSize = 12 }) }
    for _, does in ipairs(mode.does) do
      lines[#lines + 1] = ui.label({ text = "·  " .. does, fontSize = 12, maxWidth = 250 })
    end
    cards[#cards + 1] = ui.column({ key = "mode-" .. mode.word, gap = 6, padding = 14, radius = 10, flexGrow = 1, width = 280,
      fill = "surface_container", border = mode.on and "primary" or "outline_variant", borderWidth = mode.on and 2 or 1 }, lines)
  end
  local names, chosen = { "None" }, 1
  for index, name in ipairs(modes.vpns) do
    names[#names + 1] = name
    if name == modes.vpn then
      chosen = index + 1
    end
  end
  return { title = "Modes", sub = "switch from the bar or with Super+Shift+M", file = "~/.config/kavat/modes/", top = ui.row({ gap = 12 }, cards), left = {
    section("Automation"),
    row("Gamer by itself", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = modes.auto, enabled = not busy, onChange = function(now) mode_does("auto", now == "true" and "on" or "off") end }),
    })),
    note("When Steam starts a game, Gamer is entered; when the game ends, the mode from before comes back."),
  }, right = {
    section("VPN for Incognito"),
    row("Connection", select(names, chosen, function(index) mode_does("vpn", index == 1 and "none" or names[index]) end)),
    note(#modes.vpns == 0 and "NetworkManager has no VPN yet. Add a WireGuard or OpenVPN connection in the network controls, or import your provider's file with `nmcli connection import`."
      or "Incognito brings this connection up and lets nothing out but through it; if it drops, nothing goes out until it is back."),
    section("Your own"),
    ui.row({ gap = 8 }, { ui.button({ text = "New mode", variant = "outline", controlSize = "sm", onClick = function()
      noctalia.runInTerminal("mkdir -p ~/.config/kavat/modes && cd ~/.config/kavat/modes && { test -e mine.toml || cp /usr/share/kavat/modes/gamer.toml mine.toml; } && ${EDITOR:-nvim} mine.toml")
      panel.close()
    end }) }),
    note("A mode is a file. One of yours can change the processor's profile, the notifications and your own units; the system's units are only stopped by the modes of the system."),
  } }
end

local function read_games()
  noctalia.runAsync({ "kavat-game" }, function(result)
    local found = {}
    for line in result.stdout:gmatch("[^\n]+") do
      local name, game, sets = line:match("^([^\t]+)\t([^\t]*)\t(.*)$")
      if name then
        found[#found + 1] = { name = name, game = game, sets = sets }
      end
    end
    games = found
    render()
  end)
end

-- The profiles of the games: each with what it sets, opened in the editor by its button
local game_name = ""
local function games_part()
  local part = { section("A profile for each game") }
  for _, game in ipairs(games) do
    part[#part + 1] = row(game.name, ui.row({ gap = 8, align = "center" }, {
      ui.label({ text = (game.game ~= "" and (game.game .. " · ") or "") .. (game.sets ~= "" and game.sets or "sets nothing"), fontFamily = "JetBrains Mono", fontSize = 12, flexGrow = 1 }),
      ui.button({ text = "Edit", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("kavat-game edit " .. game.name)
        panel.close()
      end }),
    }))
  end
  part[#part + 1] = ui.row({ gap = 8, align = "center" }, {
    ui.input({ key = "game-name", placeholder = "a name, and the game's number in Steam", flexGrow = 1, controlSize = "sm",
      onChange = function(text) game_name = text end,
      onSubmit = function(text) game_name = text end }),
    ui.button({ text = "New profile", variant = "outline", controlSize = "sm", onClick = function()
      local name, game = game_name:match("^%s*([%w._-]+)%s*([%w ._-]*)$")
      if not name then
        said = "A profile's name is letters, digits, dots and dashes."
        render()
        return
      end
      noctalia.runAsync(game ~= "" and { "kavat-game", "new", name, trim(game) } or { "kavat-game", "new", name }, function(result)
        said = result.exitCode ~= 0 and trim(result.stderr) or ""
        read_games()
      end)
    end }),
  })
  part[#part + 1] = note("A profile sets the performance monitor and the limit of frames (MangoHud), the game's own resolution and scaling (gamescope), and a profile of the card (LACT). In Steam, the game's launch options are `kavat-game run %command%`; the profile called default is for the games that have none of their own.")
  return part
end

local function pages_gpu()
  if not gpu.name then
    return { title = "GPU", sub = "the graphics card", file = "~/.config/kavat/games/", left = { note("No graphics card answers here.") }, right = games_part() }
  end
  local function meter(name, text, part)
    return row(name, ui.column({ gap = 4, flexGrow = 1 }, {
      ui.label({ text = text, fontFamily = "JetBrains Mono", fontSize = 12 }),
      ui.progress({ progress = math.max(0, math.min(1, part)), height = 6, radius = 3, fill = "primary", track = "surface_container_highest" }),
    }))
  end
  local left = { section("Right now") }
  if gpu.temperature then
    left[#left + 1] = meter("Temperature", gpu.temperature .. " °C", tonumber(gpu.temperature) / 100)
  end
  if gpu.usage then
    left[#left + 1] = meter("Use", gpu.usage .. " %", tonumber(gpu.usage) / 100)
  end
  if gpu.memory then
    local used, total = gpu.memory:match("^(%d+) (%d+)$")
    left[#left + 1] = meter("Memory", string.format("%.1f of %.0f GB", used / 1024, total / 1024), used / math.max(1, total))
  end
  if gpu.fan then
    local turns, most = gpu.fan:match("^(%d+) (%d+)$")
    left[#left + 1] = meter("Fan", turns .. " rpm", tonumber(most) > 0 and turns / most or 0)
  end
  if gpu.power then
    local watts, cap = gpu.power:match("^(%d+) (%d+)$")
    left[#left + 1] = meter("Power", watts .. " W" .. (tonumber(cap) > 0 and (" of " .. cap .. " W allowed") or ""), tonumber(cap) > 0 and watts / cap or 0)
  end
  if #left == 1 then
    left[2] = note("This card gives no readings: it has no sensors the kernel shows, as in a virtual machine.")
  end
  return { title = "GPU", sub = gpu.name, left = left, right = {
    section("Fan curve and power limit"),
    note("They are set with LACT, a program for exactly this, which keeps them after a restart."),
    ui.row({ gap = 8 }, {
      gpu.lact == "yes" and ui.button({ text = "Open LACT", variant = "primary", controlSize = "sm", onClick = function()
        noctalia.runAsync("setsid -f lact gui > /dev/null 2>&1")
        panel.close()
      end }) or ui.button({ text = "Install LACT", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("kav install lact && sudo systemctl enable --now lactd; echo; read -r -n 1 -s -p 'Any key closes.'")
        panel.close()
      end }),
    }),
    ui.column({ gap = 10 }, games_part()),
    section("Not built yet"),
    note("The card's sensors on the bar."),
  } }
end

local function pages_browser()
  local ours = browser.default == "kavat-browser.desktop"
  local open = function(address)
    return function()
      noctalia.runAsync({ "kavat-browser", address })
      panel.close()
    end
  end
  return { title = "Browser", sub = "Kavat Browser · Chromium, made the system's own", file = "~/.config/kavat/browser/commands.toml", left = {
    section("The browser"),
    row("Opens links", ui.row({ gap = 8, align = "center", flexGrow = 1 }, {
      ui.label({ text = ours and "Kavat Browser" or (browser.default ~= "" and browser.default or "nothing chosen"), fontFamily = "JetBrains Mono", fontSize = 12, flexGrow = 1 }),
      ui.button({ text = "Make it the default", variant = "outline", controlSize = "sm", enabled = not ours and not busy, onClick = function()
        noctalia.runAsync({ "xdg-settings", "set", "default-web-browser", "kavat-browser.desktop" }, function() read_browser() end)
      end }),
    })),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Open it", variant = "primary", controlSize = "sm", onClick = function() noctalia.runAsync("kavat-browser"); panel.close() end }),
      ui.button({ text = "Its own settings", variant = "outline", controlSize = "sm", onClick = open("chrome://settings") }),
    }),
    note("Super+B opens it. In Incognito mode every window it opens is a private one, and the ones that were open are hidden until the mode is left; in Gamer the tabs out of view go to sleep."),
    section("Command console"),
    row("Yours", ui.label({ text = browser.commands .. " in the file below", fontFamily = "JetBrains Mono", fontSize = 12 })),
    note("Ctrl+Shift+K opens the console in the browser: type to find a command, Enter runs it. Your own are in the file at the foot of this page, each a name and an address to open or a command to run. `kavat-browser command \"zen mode\"` asks the open browser for one from outside."),
  }, right = {
    section("Shield"),
    note("Ads and trackers are blocked from the first start by uBlock Origin Lite, with its lists inside: it needs no network to work. Its switch for each site and its lists are in its own page."),
    ui.row({ gap = 8 }, {
      ui.button({ text = "The blocker's settings", variant = "outline", controlSize = "sm", onClick = open("chrome-extension://ofdabgbaeilkjjdadjindcdlfonikhmo/dashboard.html") }),
    }),
    section("Notes"),
    note("Ctrl+Shift+S, or the right button's \"Save to the notes\", keeps the page as a note with what is selected as a quote, tagged #clip. In Incognito nothing is saved."),
    section("Protected video"),
    row("Widevine", ui.label({ text = browser.widevine ~= "" and browser.widevine or "not installed", fontFamily = "JetBrains Mono", fontSize = 12 })),
    note("Netflix, Prime Video, Disney+ and Spotify play through it, at 720p at most, as in any browser on Linux."),
  } }
end

local function pages_projects()
  local list = {}
  for _, project in ipairs(projects) do
    list[#list + 1] = ui.row({ key = "project-" .. project.name, gap = 10, align = "center", padding = 10, radius = 8, fill = "surface_container" }, {
      ui.column({ gap = 2, flexGrow = 1 }, {
        ui.label({ text = project.name, fontWeight = "bold" }),
        ui.label({ text = project.folder .. (project.workspace ~= "" and ("  ·  workspace " .. project.workspace) or ""), fontSize = 12, color = "on_surface_variant", maxWidth = 380 }),
      }),
      ui.button({ text = project.open and "Go to it" or "Open", controlSize = "sm", variant = "primary", onClick = function()
        noctalia.runAsync({ "kavat-project", "open", project.name })
        panel.close()
      end }),
      ui.button({ text = "Edit", controlSize = "sm", variant = "outline", onClick = function()
        noctalia.runInTerminal("kavat-project edit " .. project.name)
        panel.close()
      end }),
    })
  end
  if #list == 0 then
    list[1] = note("No project yet.")
  end
  return { title = "Projects", sub = "a whole workspace, saved and reopened", file = "~/.config/kavat/projects/", left = {
    section("Your projects"),
    ui.column({ gap = 6 }, list),
  }, right = {
    section("A new one"),
    row("Name", ui.input({ key = "project-name", placeholder = "letters, digits, - and _", controlSize = "sm", flexGrow = 1, value = project_name,
      onChange = function(text) project_name = text end })),
    ui.row({ gap = 8 }, { ui.button({ text = "Make it, for a folder", variant = "outline", controlSize = "sm", onClick = function()
      if trim(project_name):match("^[%w][%w._-]*$") then
        -- the folder is chosen in Yazi; the project is made for the one it is left in
        noctalia.runInTerminal("cd ~ && folder=$(mktemp) && yazi --cwd-file \"$folder\" && kavat-project new " .. trim(project_name) .. " \"$(cat \"$folder\")\"; echo; read -r -n 1 -s -p 'Any key closes.'")
        panel.close()
      else
        said = "A project's name is letters, digits, - and _."
        render()
      end
    end }) }),
    note("Yazi opens to choose the folder: go into it and leave with q. A project opens a terminal and the editor in its folder, on a workspace of its own; its file says what else: files, the browser with a profile of its own, or any command."),
    note("In the launcher, /pr lists the projects. `kavat project open <name>` does the same from a terminal."),
  } }
end

local function pages_privacy()
  local doors = {}
  for _, door in ipairs(privacy.doors) do
    doors[#doors + 1] = ui.label({ text = door, fontFamily = "JetBrains Mono", fontSize = 12 })
  end
  if #doors == 0 then
    doors[1] = note("None: nothing from outside reaches this machine but the answers to what it asked for.")
  end
  local asking = privacy.opensnitch == "active"
  local dns_on = privacy.dns:match("^on") ~= nil
  local dns_who = privacy.dns:match("^on (.+)$") or ""
  -- a change here is another program's: it is run, what it says when it fails is shown, and the page is read again
  local function privacy_does(command)
    busy = true
    said = ""
    render()
    noctalia.runAsync(command, function(result)
      busy = false
      if result.exitCode ~= 0 then
        said = trim(result.stderr ~= "" and result.stderr or result.stdout)
      end
      read_privacy()
    end)
  end
  local boxes = {}
  for _, box in ipairs(privacy.sandbox) do
    if box.state == "app" then
      boxes[#boxes + 1] = row(box.name, ui.label({ text = "kept in a sandbox", fontFamily = "JetBrains Mono", fontSize = 12 }))
    elseif box.name == "browser" then
      boxes[#boxes + 1] = row("The browser", ui.toggle({ checked = box.state == "on", enabled = not busy, onChange = function(now)
        privacy_does({ "kavat-sandbox", now and "on" or "off", "browser" })
      end }))
    end
  end
  return { title = "Privacy & network", sub = "what comes in, what goes out", file = "/etc/kavat/firewall.d/", left = {
    section("Firewall"),
    row("Everything incoming", ui.label({ text = privacy.firewall == "active" and "closed" or "NOT closed: the firewall is off", fontFamily = "JetBrains Mono", fontSize = 12,
      color = privacy.firewall == "active" and "on_surface" or "error" })),
    section("Names asked for over TLS"),
    row("Encrypted DNS", ui.row({ gap = 8, align = "center" }, {
      ui.label({ text = dns_on and ("on · " .. dns_who) or "off", fontFamily = "JetBrains Mono", fontSize = 12 }),
      ui.button({ text = dns_on and "Turn it off" or "Turn it on", variant = "outline", controlSize = "sm", enabled = not busy, onClick = function()
        privacy_does(dns_on and { "kavat-dns", "off" } or { "kavat-dns", "on" })
      end }),
    })),
    dns_on and row("Who answers", segmented({
      { text = "Quad9", on = dns_who == "quad9", pick = function() privacy_does({ "kavat-dns", "on", "quad9" }) end },
      { text = "Cloudflare", on = dns_who == "cloudflare", pick = function() privacy_does({ "kavat-dns", "on", "cloudflare" }) end },
      { text = "Mullvad", on = dns_who == "mullvad", pick = function() privacy_does({ "kavat-dns", "on", "mullvad" }) end },
    })) or note("Off, the names of the sites you go to are asked in the clear of the server the network gives. On, they go over TLS to one server and nothing is asked in the clear; a network that makes you sign in on a page of its own first will not let it through."),
    section("Doors you opened"),
    ui.column({ gap = 4 }, doors),
    note("A door is a line in a file of /etc/kavat/firewall.d/, as `add rule inet kavat_firewall in tcp dport 22 accept`, and then `sudo systemctl restart kavat-firewall`."),
  }, right = {
    section("Programs that reach the network"),
    row("Ask the first time", ui.label({ text = asking and "on · OpenSnitch" or "off", fontFamily = "JetBrains Mono", fontSize = 12 })),
    ui.row({ gap = 8 }, {
      asking and ui.button({ text = "Open its rules", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runAsync("setsid -f opensnitch-ui > /dev/null 2>&1")
        panel.close()
      end }) or ui.button({ text = "Turn it on", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("kav install opensnitch && sudo systemctl enable --now opensnitchd && (setsid -f opensnitch-ui > /dev/null 2>&1); echo; read -r -n 1 -s -p 'Any key closes.'")
        panel.close()
      end }),
    }),
    note("OpenSnitch asks the first time a program wants to reach the network, and keeps your answer as a rule. It is not on as the system comes: it asks often at first."),
    section("Sandbox"),
    ui.column({ gap = 6 }, boxes),
    note("In its sandbox the browser sees its own folders, the downloads and the notes, and nothing else of your home folder: a file can only be uploaded from there. It is from the browser's next start. `kavat-sandbox app <name>` keeps an Electron app in one, with a home folder of its own."),
    section("Incognito"),
    row("VPN", ui.label({ text = privacy.vpn ~= "" and privacy.vpn or "none chosen", fontFamily = "JetBrains Mono", fontSize = 12 })),
    note("In Incognito mode everything goes out through this VPN and nothing goes out when it drops. It is chosen on the Modes page."),
    section("Not built yet"),
    note("Who used the microphone, the camera and the screen."),
  } }
end

local function pages_backup()
  local in_terminal = function(command)
    return function()
      noctalia.runInTerminal(command .. "; echo; read -r -n 1 -s -p 'Any key closes.'")
      panel.close()
    end
  end
  local changes = {}
  for _, change in ipairs(backup.history) do
    changes[#changes + 1] = ui.row({ key = "change-" .. change.mark, gap = 10 }, {
      ui.label({ text = change.when, fontFamily = "JetBrains Mono", fontSize = 12, color = "on_surface_variant" }),
      ui.label({ text = change.what, fontSize = 12, maxWidth = 300 }),
    })
  end
  if #changes == 0 then
    changes[1] = note("Nothing was changed yet.")
  end
  return { title = "Backup & sync", sub = "snapshots · backups · the history of your settings", file = "~/.config/kavat/backup.place", left = {
    section("Snapshots of your home folder"),
    row("Kept now", ui.label({ text = backup.snapshots .. " · one every hour", fontFamily = "JetBrains Mono", fontSize = 12 })),
    note("The last day's, a week of daily ones and a month of weekly ones. In Yazi, V shows the previous versions of what is under the cursor, and brings one back. The Steam library is not in them."),
    section("Backups off the computer"),
    row("Place", ui.input({ key = "place", placeholder = "a folder, or sftp:you@host:/folder", controlSize = "sm", flexGrow = 1,
      value = backup.place, enabled = not busy, onSubmit = function(text)
        if trim(text) ~= "" then backup_does({ "kavat-backup", "to", trim(text) }) end
      end })),
    row("Made", segmented({
      { text = "Daily", on = backup.every == "daily", pick = function() backup_does({ "kavat-backup", "every", "daily" }) end },
      { text = "Weekly", on = backup.every == "weekly", pick = function() backup_does({ "kavat-backup", "every", "weekly" }) end },
      { text = "By hand", on = backup.every == "never", pick = function() backup_does({ "kavat-backup", "every", "never" }) end },
    })),
    row("Last one", ui.label({ text = backup.last ~= "" and backup.last or "none yet", fontFamily = "JetBrains Mono", fontSize = 12 })),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Back up now", variant = "primary", controlSize = "sm", enabled = backup.place ~= "", onClick = in_terminal("kavat-backup run") }),
      ui.button({ text = "The copies", variant = "outline", controlSize = "sm", enabled = backup.place ~= "", onClick = in_terminal("kavat-backup list") }),
      ui.button({ text = "Show the key", variant = "outline", controlSize = "sm", enabled = backup.place ~= "", onClick = in_terminal("kavat-backup key") }),
    }),
    note("Type the place and press Enter. The copies are encrypted: without the key they cannot be opened, so keep the key somewhere off this machine."),
  }, right = {
    section("History of your settings"),
    ui.column({ gap = 4 }, changes),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Open the history", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("kavat-history browse")
        panel.close()
      end }),
      ui.button({ text = "Undo the last change", variant = "outline", controlSize = "sm", enabled = not busy and #backup.history > 1,
        onClick = function() backup_does({ "kavat-history", "undo" }) end }),
    }),
    note("Every change, from this panel or by hand, is kept. Going back to one puts the files as they were and tells the apps."),
    section("Sync"),
    row("Sent to", ui.input({ key = "remote", placeholder = "the address of a private git repository", controlSize = "sm", flexGrow = 1,
      value = backup.remote, enabled = not busy, onSubmit = function(text)
        if trim(text) ~= "" then backup_does({ "kavat-history", "remote", trim(text) }) end
      end })),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Send now", variant = "outline", controlSize = "sm", enabled = backup.remote ~= "", onClick = in_terminal("kavat-history push") }),
    }),
    note("On another machine, `kavat-history clone <address>` brings these settings and wears them."),
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

-- what opens each kind of thing, as the desktop's own list says
local shell_apps = {}
local function read_shell_apps()
  noctalia.runAsync("for pair in 'A link=x-scheme-handler/https' 'A folder=inode/directory' 'A text file=text/plain' 'An image=image/png' 'A video=video/mp4' 'A PDF=application/pdf'; do "
    .. "printf '%s\\t%s\\n' \"${pair%%=*}\" \"$(xdg-mime query default \"${pair#*=}\" 2>/dev/null)\"; done", function(result)
    local found = {}
    for line in result.stdout:gmatch("[^\n]+") do
      local what, name = line:match("^([^\t]+)\t(.*)$")
      if what then
        found[#found + 1] = { what = what, name = name ~= "" and name:gsub("%.desktop$", "") or "nothing set" }
      end
    end
    shell_apps = found
    render()
  end)
end

local function shell()
  local apps = {}
  for _, app in ipairs(shell_apps) do
    apps[#apps + 1] = row(app.what, ui.label({ text = app.name, fontFamily = "JetBrains Mono", fontSize = 12 }))
  end
  return { title = "Shell", sub = "zsh · from the next terminal opened", file = "~/.config/kavat/shell.conf", left = {
    section("Prompt"),
    row("Shows", conf_choice("shell", "prompt", { { "short", "Short" }, { "full", "Full" } })),
    note("Short is who, where and the branch, and starts no program to draw itself. Full adds the state of the repository and how long the last command took; a large repository makes it slower. A prompt of your own in ~/.config/starship.toml takes the place of both."),
    section("As you type"),
    row("Suggestions from the history", conf_toggle("shell", "suggestions", "yes", "no")),
    row("Colours for what is typed", conf_toggle("shell", "highlighting", "yes", "no")),
    row("Tab opens the picker", conf_toggle("shell", "completion_picker", "yes", "no")),
    section("History"),
    row("Commands kept", conf_select("shell", "history_lines", { { "10000", "10,000" }, { "50000", "50,000" }, { "200000", "200,000" } })),
    row("Shared between terminals", conf_toggle("shell", "shared_history", "yes", "no")),
  }, right = {
    section("Listing"),
    row("ls is", conf_choice("shell", "listing", { { "lsd", "lsd" }, { "ls", "ls" } })),
    section("Aliases and functions"),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Open them in Neovim", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("mkdir -p ~/.config/kavat && nvim ~/.config/kavat/aliases.zsh")
        panel.close()
      end }),
    }),
    note("Yours live in ~/.config/kavat/aliases.zsh, which every terminal reads; ~/.zshrc is read after everything and has the last word."),
    section("Default apps"),
    ui.column({ gap = 2 }, apps),
    note("What opens a link, a folder, a text file, an image, a video and a PDF. `xdg-mime default <app>.desktop <type>` changes one."),
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
  local tiles, rows = {}, {}
  for _, item in ipairs(walls.list) do
    if wall_filter == "all" or item.kind == wall_filter or item.kind == "theme" then
      local on = (item.kind == "theme" and walls.kind == "theme") or item.file == walls.file
      tiles[#tiles + 1] = ui.column({ key = "wall-" .. item.file, gap = 4, width = 178 }, {
        ui.image({ path = item.thumb, width = 178, height = 100, radius = 8, fit = "cover",
          border = on and "primary" or "outline_variant", borderWidth = on and 3 or 1,
          onClick = function()
            if item.kind == "theme" then wall_does("theme") else wall_does("set", item.file) end
          end }),
        ui.row({ gap = 6, align = "center" }, {
          ui.label({ text = item.name, fontSize = 12, maxWidth = 120 }),
          ui.spacer({ flexGrow = 1 }),
          ui.label({ text = item.kind == "video" and "▶ Live" or item.kind == "theme" and "Theme" or "Image", fontSize = 11, color = "on_surface_variant" }),
        }),
      })
    end
  end
  for at = 1, #tiles, 5 do
    rows[#rows + 1] = ui.row({ gap = 12 }, { tiles[at], tiles[at + 1], tiles[at + 2], tiles[at + 3], tiles[at + 4] })
  end
  local top = ui.column({ gap = 12 }, {
    ui.row({ gap = 12, align = "center" }, {
      section("Library"),
      ui.spacer({ flexGrow = 1 }),
      ui.row({ width = 260 }, { segmented({
        { text = "All", on = wall_filter == "all", pick = function() wall_filter = "all"; render() end },
        { text = "Images", on = wall_filter == "image", pick = function() wall_filter = "image"; render() end },
        { text = "Live", on = wall_filter == "video", pick = function() wall_filter = "video"; render() end },
      }) }),
    }),
    ui.column({ gap = 12 }, rows),
  })
  local live = walls.kind == "video"
  return { title = "Wallpaper", sub = "image or live video · " .. (live and "a video is playing" or walls.kind == "image" and "an image of yours" or "the theme's picture"),
    file = "~/.config/kavat/wallpaper.json", top = top, left = {
    section("Folders"),
    row("Images", ui.label({ text = "~/Pictures/Wallpapers/images", fontFamily = "JetBrains Mono", fontSize = 12 })),
    row("Live", ui.label({ text = "~/Pictures/Wallpapers/live", fontFamily = "JetBrains Mono", fontSize = 12 })),
    ui.row({ gap = 8 }, {
      ui.button({ text = "Open the folders", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("yazi ~/Pictures/Wallpapers")
        panel.close()
      end }),
    }),
    note("A file put in one of the two folders shows here. In Yazi, W sets the file under the cursor as the background."),
    section("Also"),
    row("Colours from it", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = theme.palette == "wallpaper", enabled = not busy, onChange = function(now)
        if now == "true" then wall_does("palette") else set("palette", "kavat") end
      end }),
    })),
    note("The theme takes its palette from the background, and again when the background changes. The login and the lock show a still frame of a video."),
  }, right = {
    section("Live wallpaper"),
    row("Sound", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = walls.sound == "on", enabled = not busy, onChange = function(now) wall_does("option", "sound", now == "true" and "on" or "off") end }),
    })),
    row("Fit", segmented({
      { text = "Fill", on = walls.fit == "fill", pick = function() wall_does("option", "fit", "fill") end },
      { text = "Fit", on = walls.fit == "fit", pick = function() wall_does("option", "fit", "fit") end },
      { text = "Stretch", on = walls.fit == "stretch", pick = function() wall_does("option", "fit", "stretch") end },
    })),
    row("GPU decoding", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = walls.gpu == "on", enabled = not busy, onChange = function(now) wall_does("option", "gpu", now == "true" and "on" or "off") end }),
    })),
    note("The video is played by mpvpaper. It stops while a window covers the whole screen, and in Gamer mode."),
    note("A different background for each display and a slideshow are not built yet."),
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
  return { title = "Sound", sub = "PipeWire", file = "~/.config/pipewire/pipewire.conf.d/", left = {
    section("Volume and devices"),
    note("The output, the input and the volume of each app are in the shell's control center."),
    ui.row({ gap = 8 }, { shell_panel("Open the sound controls", "panel-toggle control-center audio") }),
  }, right = {
    section("Microphone"),
    row("Noise removed", conf_toggle("sound", "noise_suppression", "on", "off")),
    note("Adds a microphone to the list, \"Microphone, noise removed\", that lets the voice through and leaves out keys, fans and the room. Calls use it unless another is chosen. Changing this restarts the sound for a moment."),
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

local displays_read = 0

local function read_displays()
  noctalia.runAsync("kavat-displays; for name in $(kavat-displays | cut -f1); do echo \"== $name\"; kavat-displays modes $name; done", function(result)
    local found, part = {}, nil
    for line in result.stdout:gmatch("[^\n]+") do
      local heading = line:match("^== (.+)$")
      local name, what, mode, position, scale, turn = line:match("^([^\t]+)\t([^\t]*)\t([^\t]+)\t([^\t]+)\t([^\t]+)\t([^\t]+)$")
      if heading then
        part = heading
      elseif part == nil and name then
        local width, height, rate = mode:match("^(%d+)x(%d+)@([%d.]+)")
        local x, y = position:match("^(-?%d+)x(-?%d+)")
        found[#found + 1] = { name = name, what = what, mode = mode, size = width .. "x" .. height, rate = rate, width = tonumber(width), height = tonumber(height),
          x = tonumber(x), y = tonumber(y), scale = scale, turn = turn, modes = {} }
      elseif part then
        for _, display in ipairs(found) do
          if display.name == part then
            display.modes[#display.modes + 1] = line
          end
        end
      end
    end
    table.sort(found, function(one, other) return one.x < other.x end)
    displays = found
    -- what is read is what is drawn: a display dropped where it could not stay goes back
    displays_read = displays_read + 1
    render()
  end)
end

-- A change of a display is tried: it stays only when it is kept before ten seconds go by
local function display_does(...)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-displays", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    else
      display_tried = display_tried + 1
      local this = display_tried
      noctalia.runAsync("sleep 11", function()
        if display_tried == this then
          display_tried = 0
          read_displays()
        end
      end)
    end
    read_displays()
  end)
end

local function pages_displays()
  if #displays == 0 then
    return { title = "Displays", sub = "Hyprland", left = { note("Hyprland lists no display.") }, right = {} }
  end
  local chosen = displays[1]
  local boxes, names = {}, {}
  for _, display in ipairs(displays) do
    if display.name == display_chosen then
      chosen = display
    end
  end
  -- each display to scale, where it is: the room it takes is its size, turned and scaled
  for _, display in ipairs(displays) do
    local sideways = display.turn == "90" or display.turn == "270"
    local scale = tonumber(display.scale) or 1
    boxes[#boxes + 1] = { key = display.name, x = display.x, y = display.y,
      width = math.floor((sideways and display.height or display.width) / scale + 0.5),
      height = math.floor((sideways and display.width or display.height) / scale + 0.5),
      label = display.name, sublabel = display.size .. " · " .. string.format("%.0f", tonumber(display.rate) or 0) .. " Hz",
      selected = display == chosen }
    if display ~= chosen then
      names[#names + 1] = display.name
    end
  end
  local top = ui.column({ gap = 10, align = "center" }, {
    -- dragged to where it goes: it ends against the nearest edge of another display
    ui.arrange({ key = "arrangement", width = 560, height = 230, items = boxes, revision = displays_read,
      onSelect = function(name) display_chosen = name; render() end,
      onMove = function(name, x, y) display_does("place", name, tostring(x), tostring(y)) end }),
    display_tried > 0 and ui.row({ gap = 10, align = "center", justify = "center" }, {
      ui.label({ text = "Keep this arrangement? Unless it is kept, it goes back in ten seconds.", color = "primary" }),
      ui.button({ text = "Keep", variant = "primary", controlSize = "sm", onClick = function()
        display_tried = 0
        noctalia.runAsync({ "kavat-displays", "keep" }, function() read_displays() end)
      end }),
      ui.button({ text = "Undo", variant = "outline", controlSize = "sm", onClick = function()
        display_tried = 0
        noctalia.runAsync({ "kavat-displays", "undo" }, function() read_displays() end)
      end }),
    }) or ui.label({ text = #displays > 1 and (chosen.what .. " · drag a display to where it stands") or chosen.what, color = "on_surface_variant", fontSize = 12 }),
  })

  -- the sizes the display can show, and the rates of the size in use
  local sizes, size_at, rates, rate_at, seen = {}, 1, {}, 1, {}
  for _, mode in ipairs(chosen.modes) do
    local size, rate = mode:match("^(%d+x%d+)@([%d.]+)")
    if size and not seen[size] then
      seen[size] = true
      sizes[#sizes + 1] = size
      if size == chosen.size then
        size_at = #sizes
      end
    end
    if size == chosen.size then
      rates[#rates + 1] = rate
      if math.abs((tonumber(rate) or 0) - (tonumber(chosen.rate) or 0)) < 0.6 then
        rate_at = #rates
      end
    end
  end
  local rate_names = {}
  for index, rate in ipairs(rates) do
    rate_names[index] = string.format("%.0f Hz", tonumber(rate) or 0)
  end
  local function choices(values, now, name, texts)
    local list = {}
    for index, value in ipairs(values) do
      list[index] = { text = texts and texts[index] or value, on = tonumber(value) == tonumber(now), pick = function() display_does("set", chosen.name, name .. "=" .. value) end }
    end
    return segmented(list)
  end
  local places, place_values = { "Where it is" }, { "" }
  for _, name in ipairs(names) do
    for _, side in ipairs({ { "right-of", "Right of " }, { "left-of", "Left of " }, { "above", "Above " }, { "below", "Below " } }) do
      places[#places + 1] = side[2] .. name
      place_values[#place_values + 1] = side[1] .. ":" .. name
    end
  end
  return { title = "Displays", sub = #displays == 1 and "one display" or (#displays .. " displays · drag them to where they stand, press one to change it"),
    file = "~/.config/hypr/monitors.lua", top = top, left = {
    section(chosen.name),
    row("Resolution", #sizes > 0 and select(sizes, size_at, function(index) display_does("set", chosen.name, "mode=" .. sizes[index]) end) or note(chosen.size)),
    row("Refresh rate", #rates > 0 and select(rate_names, rate_at, function(index) display_does("set", chosen.name, "mode=" .. chosen.size .. "@" .. rates[index]) end) or note(chosen.rate .. " Hz")),
    row("Scale", choices({ "1", "1.25", "1.5", "2" }, chosen.scale, "scale", { "100%", "125%", "150%", "200%" })),
    row("Rotation", choices({ "0", "90", "180", "270" }, chosen.turn, "rotate", { "0°", "90°", "180°", "270°" })),
  }, right = {
    section("Place"),
    #names > 0 and row("Put it", select(places, 1, function(index)
      if index > 1 then display_does("set", chosen.name, "at=" .. place_values[index]) end
    end)) or note("With one display there is nothing to arrange. A second one is put to the right of this one when it is plugged in."),
    note("Each set of displays keeps its own arrangement, known by what the displays are and not by where they are plugged in."),
    section("Not built yet"),
    note("The main display, the workspaces of each display and night light."),
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
  displays = pages_displays, sound = sound, network = network, terminal = terminal, shell = shell, editor = editor, files = files, notes = notes,
  bar = bar, modes = pages_modes, backup = pages_backup, gpu = pages_gpu, browser = pages_browser, projects = pages_projects, privacy = pages_privacy, packages = pages_packages, boot = boot, login = login, language = language }

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
        ui.label({ text = entry.name, flexGrow = 1, fontWeight = entry.id == page and "medium" or "normal",
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
  elseif news ~= "" then
    body[#body + 1] = ui.label({ text = news, color = "primary", maxWidth = 900 })
  end
  -- a page may have something across its whole width above its two columns
  local columns = ui.row({ gap = 36, align = "start" }, {
    ui.column({ gap = 12, flexGrow = 1 }, content.left),
    ui.column({ gap = 12, flexGrow = 1 }, content.right),
  })
  body[#body + 1] = ui.scroll({ key = "page-" .. page, flexGrow = 1, gap = 22 }, content.top and { content.top, columns } or { columns })
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
      ui.button({ text = "Undo", variant = "outline", controlSize = "sm", enabled = not busy, onClick = function()
        busy = true
        render()
        noctalia.runAsync({ "kavat-history", "undo" }, function(result)
          busy = false
          open_page(page)
          local words = trim(result.exitCode == 0 and result.stdout or result.stderr):gsub("^kavat%-history: ", "")
          if result.exitCode == 0 then news = words else said = words end
          render()
        end)
      end }),
      ui.button({ text = "History", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runInTerminal("kavat-history browse")
        panel.close()
      end }),
      ui.button({ text = "Export", variant = "outline", controlSize = "sm", onClick = function()
        noctalia.runAsync({ "kavat-history", "export" }, function(result)
          if result.exitCode == 0 then
            news = "Your settings are in " .. trim(result.stdout) .. ": `kavat-history import` wears them on another machine."
          else
            said = trim(result.stderr)
          end
          render()
        end)
      end }),
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
  news = ""
  render()
  if id == "appearance" or id == "windows" or id == "accessibility" then
    read_theme()
  elseif id == "language" then
    read_conf("language")
  elseif id == "displays" then
    read_displays()
  elseif id == "backup" then
    read_backup()
  elseif id == "gpu" then
    read_games()
    read_gpu()
  elseif id == "browser" then
    read_browser()
  elseif id == "projects" then
    read_projects()
  elseif id == "privacy" then
    read_privacy()
  elseif id == "modes" then
    read_modes()
  elseif id == "sound" then
    read_conf("sound")
  elseif id == "wallpaper" then
    read_walls()
    read_theme()
  elseif id == "packages" then
    read_packages()
  elseif id == "notes" then
    read_notes()
  elseif id == "terminal" or id == "editor" or id == "files" then
    read_conf(id)
  elseif id == "shell" then
    read_conf("shell")
    read_shell_apps()
  elseif id == "login" then
    read_conf("lock")
  elseif id == "keys" then
    read_shortcuts()
    render()
  end
end

-- The page can be named when the panel is opened: panel-open kavat/settings:panel notes
function onOpen(context)
  panel_open = true
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

function onClose()
  panel_open = false
end
