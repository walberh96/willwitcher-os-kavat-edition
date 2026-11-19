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
  { id = "pointer", name = "Mouse & touchpad" },
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
  { id = "remote", name = "Remote access" },
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
local browser = { default = "", widevine = "", commands = "0", shorts = "off", apps = {} }  -- what the browser page shows
local panel_open = false  -- what reads again and again stops when the panel closes
local gpu = {}  -- what the graphics card says of itself, by name
local profiles = {}      -- the looks saved under a name
local profile_name = ""  -- the name being typed for a new one
local news = ""  -- something done that is worth saying, as `said` is for what was refused
local backup = { snapshots = "0", place = "", every = "never", last = "", history = {}, remote = "" }  -- as kavat-backup and kavat-history say it
local display_chosen = ""  -- the display whose settings are in view
local display_tried = 0    -- above zero while a change of the displays waits to be kept
local walls = { list = {}, displays = {}, kind = "theme", file = "", sound = "off", fit = "fill", gpu = "on" }  -- the background, as kavat-wallpaper says it
local wall_display = ""  -- the display a background is being chosen for; none is all of them
local wall_filter = "all"
local wall_asked = "all"   -- the filter the last search was made with
local wall_words = ""      -- what is searched for online
local wall_found = {}      -- what the sources answered: each a small picture, the full one's address and its size
local wall_searching = false
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
  noctalia.runAsync("kavat-wallpaper; echo '== list'; kavat-wallpaper list; echo '== displays'; kavat-wallpaper displays; echo '== option'; kavat-wallpaper option", function(result)
    local found, part = { list = {}, displays = {}, kind = "theme", file = "", sound = "off", fit = "fill", gpu = "on" }, nil
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
      elseif part == "displays" then
        local name, file = line:match("^([^\t]+)\t(.+)$")
        if name then
          found.displays[#found.displays + 1] = { name = name, file = file ~= "-" and file or nil }
        end
      else
        local name, value = line:match("^(%S+) (%S+)$")
        if name then
          found[name] = value
        end
      end
    end
    local still = false
    for _, display in ipairs(found.displays) do
      still = still or display.name == wall_display
    end
    if not still then
      wall_display = ""
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
    .. "echo \"commands $(grep -c '^\\[\\[command\\]\\]' ~/.config/kavat/browser/commands.toml 2>/dev/null)\"; "
    .. "echo \"background $(sed -n 's/^start_background *= *\"\\(.*\\)\"/\\1/p' ~/.config/kavat/browser/browser.toml 2>/dev/null | head -n 1)\"; "
    .. "echo \"shorts $(kavat-browser shorts 2>/dev/null)\"; kavat-webapp list 2>/dev/null | sed 's/^/app\t/'", function(result)
    local found = { default = "", widevine = "", commands = "0", background = "", shorts = "off", apps = {} }
    for line in result.stdout:gmatch("[^\n]+") do
      local app, address = line:match("^app\t([^\t]+)\t([^\t]*)")
      local name, value = line:match("^(%S+) ?(.*)$")
      if app then
        found.apps[#found.apps + 1] = { name = app, address = address }
      elseif name then
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

local key_changing = ""  -- the shortcut being given other keys, by the keys it comes with
local key_typed = ""

-- The shortcuts as kavat-keys lists them: the keys now, what they do, the group, and the
-- keys a changed one comes with. The list itself is read when kavat-keys is not there.
local function read_shortcuts()
  noctalia.runAsync({ "kavat-keys" }, function(result)
    local text, found = result.exitCode == 0 and result.stdout or (noctalia.readFile("/usr/share/kavat/hypr/shortcuts.tsv") or ""), {}
    for line in text:gmatch("[^\n]+") do
      local keys, what, group, first = line:match("^([^#\t][^\t]*)\t([^\t]*)\t([^\t]*)\t?(.*)$")
      if keys then
        local changed = first ~= "" and first ~= "-"
        found[#found + 1] = { keys = keys, what = what, group = group, first = changed and first or keys, changed = changed }
      end
    end
    shortcuts = found
    render()
  end)
end

local function key_does(...)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-keys", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-keys: ", "")
    else
      key_changing = ""
    end
    read_shortcuts()
  end)
end

-- The pieces every page is made of ------------------------------------------------------

-- While this is a list, the names of the sections and of the settings that are drawn are
-- added to it: that is how the search knows what each page has without a list kept by hand.
local gathering = nil

local function section(text)
  if gathering then
    gathering[#gathering + 1] = text
  end
  return ui.label({ text = text:upper(), fontSize = 11, fontWeight = "medium", color = "on_surface_variant" })
end

-- One setting: its name on the left, its control on the right
local function row(name, control)
  if gathering then
    gathering[#gathering + 1] = name
  end
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

-- A slider of the theme's. What it gives is a number with a fraction, wherever it is let
-- go: it is brought to the nearest step, so that a setting counted in whole numbers is
-- given a whole number.
local function slider(key, min, max, step)
  return ui.slider({ min = min, max = max, step = step, value = tonumber(theme[key]) or min, flexGrow = 1,
    controlSize = "sm", enabled = not busy, onDragEnd = function(now)
      local value = math.floor((tonumber(now) or min) / step + 0.5) * step
      value = math.max(min, math.min(max, value))
      set(key, step == math.floor(step) and string.format("%d", value) or string.format("%.2f", value))
    end })
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

-- The boot: what kavat-boot-settings keeps, and the look the system's own screens wear
local boot_now = { timeout = "5", quiet = "yes", splash = "yes", fallback = "yes", snapshots = "yes", recovery = "yes", firmware = "yes", other = "yes" }
local boot_look = {}  -- the palette, the mode and the accent of the boot menu, the loading screen and the login

local login_now = { clock = "24", users = "yes", auto = "none", people = {} }  -- how the login screen asks, as kavat-login-settings says it

local function read_boot()
  noctalia.runAsync("kavat-boot-settings; echo '== look'; cat /etc/kavat/theme.toml /var/lib/kavat/theme/settings.toml 2>/dev/null; echo '== login'; kavat-login-settings 2>/dev/null; "
    .. "echo '== people'; getent passwd | awk -F: '$3 >= 1000 && $3 < 60000 && $7 !~ /(nologin|false)$/ { print $1 }'", function(result)
    local part = "boot"
    local found = { clock = "24", users = "yes", auto = "none", people = {} }
    boot_look = {}
    for line in result.stdout:gmatch("[^\n]+") do
      local heading = line:match("^== (%a+)$")
      if heading then
        part = heading
      elseif part == "look" then
        -- the look the machine was installed with, then what was handed to the boot since: the later one counts
        local key, value = line:match('^(%w+)%s*=%s*"([^"]*)"')
        if key then
          boot_look[key] = value
        end
      elseif part == "people" then
        found.people[#found.people + 1] = line
      else
        local key, value = line:match("^(%w+)\t([%w._-]+)$")
        if key and part == "boot" then
          boot_now[key] = value
        elseif key then
          found[key] = value
        end
      end
    end
    login_now = found
    render()
  end)
end

local function boot_set(name, value)
  busy = true
  said = ""
  boot_said = "Waiting for your password, then writing the boot menu again…"
  render()
  noctalia.runAsync({ "kavat-boot-settings", "set", name .. "=" .. value }, function(result)
    busy = false
    boot_said = ""
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-boot%-settings: ", "")
      if said == "" then said = "The password was not given, so nothing changed." end
    else
      news = "Changed. It shows the next time the machine starts."
    end
    read_boot()
  end)
end

local function boot_toggle(name)
  return ui.row({ justify = "end", flexGrow = 1 }, {
    ui.toggle({ checked = boot_now[name] == "yes", enabled = not busy, onChange = function(now) boot_set(name, now == "true" and "yes" or "no") end }),
  })
end

local function pretty(word)
  return word and word ~= "" and (word:sub(1, 1):upper() .. word:sub(2)) or "?"
end

-- The boot menu, the loading screen and the login screen are shown before anybody logs in,
-- so they cannot follow the desktop's theme by themselves: this says which look they have
-- and, when the desktop's is another, offers to copy it to them.
local function system_look()
  local theirs = pretty(boot_look.palette) .. " · " .. pretty(boot_look.mode) .. " · " .. pretty(boot_look.accent)
  local accent = theme.accent == "default" and boot_look.accent or theme.accent
  local same = boot_look.palette == theme.palette and boot_look.mode == theme.mode and (theme.accent == "default" or boot_look.accent == theme.accent)
  local parts = {
    row("They wear", ui.label({ text = boot_look.palette and theirs or "…", fontWeight = "medium" })),
    row("Your desktop", ui.label({ text = theme.palette and (pretty(theme.palette) .. " · " .. pretty(theme.mode) .. " · " .. pretty(accent)) or "…", fontWeight = "medium" })),
  }
  if boot_look.palette and theme.palette and same then
    parts[#parts + 1] = note("They already look like your desktop.")
  else
    parts[#parts + 1] = ui.row({ gap = 8 }, {
      ui.button({ text = "Make them look like my desktop", variant = "primary", enabled = not busy,
        onClick = function()
          busy = true
          boot_said = "Waiting for your password, then making their pictures…"
          render()
          noctalia.runAsync({ "kavat-theme", "boot" }, function(result)
            busy = false
            boot_said = result.exitCode == 0 and "Done: you will see it the next time the machine starts." or trim(result.stderr ~= "" and result.stderr or result.stdout)
            read_boot()
          end)
        end }),
    })
  end
  parts[#parts + 1] = note("The boot menu, the loading screen and the login screen show before anybody logs in, so they do not change when you change the desktop's theme: they keep their own palette, mode and accent until you ask for it here. It asks for your password, because they are files of the system.")
  if boot_said ~= "" then
    parts[#parts + 1] = ui.label({ text = boot_said, color = "primary", maxWidth = 440 })
  end
  return parts
end

local function boot()
  local waits, wait_names = { "0", "1", "3", "5", "10", "30" },
    { "Not shown · hold Shift or Esc to see it", "1 second", "3 seconds", "5 seconds", "10 seconds", "30 seconds" }
  local right = {
    section("While it starts"),
    row("Loading screen", boot_toggle("splash")),
    row("No kernel messages", boot_toggle("quiet")),
    note("With the loading screen off, the system starts showing its messages. Esc shows them from the loading screen too."),
    section("Look"),
  }
  for _, part in ipairs(system_look()) do
    right[#right + 1] = part
  end
  right[#right + 1] = row("Follow my desktop", boot_toggle("follow"))
  right[#right + 1] = note("With this on they take the desktop's theme by themselves, as the machine is turned off: that turning off takes about half a minute longer when the theme changed.")
  return { title = "Boot", sub = "the boot menu and the loading screen", file = "/etc/kavat/boot.conf", left = {
    section("Boot menu"),
    row("Show menu for", select(wait_names, index_of(waits, boot_now.timeout), function(index) boot_set("timeout", waits[index]) end)),
    row("Starts by itself", segmented({
      { text = "This system", on = boot_now.default ~= "other", pick = function() boot_set("default", "kavat") end },
      { text = "The other system", on = boot_now.default == "other", pick = function() boot_set("default", "other") end },
    })),
    section("Entries on the menu"),
    row("Fallback kernel", boot_toggle("fallback")),
    row("Safe mode", ui.row({ justify = "end", align = "center", gap = 8, flexGrow = 1 }, {
      ui.label({ text = "always shown", fontSize = 12, color = "on_surface_variant" }), ui.toggle({ checked = true, enabled = false }) })),
    row("Snapshots", boot_toggle("snapshots")),
    row("Recovery", boot_toggle("recovery")),
    row("Firmware settings", boot_toggle("firmware")),
    row("Other systems", boot_toggle("other")),
    note("A change asks for your password and writes the menu again; you see it the next time the machine starts. After a start that failed, the menu opens by itself on Safe mode."),
  }, right = right }
end

local MINUTES = { { "0", "Never" }, { "1", "1 minute" }, { "5", "5 minutes" }, { "10", "10 minutes" }, { "15", "15 minutes" },
  { "30", "30 minutes" }, { "60", "1 hour" } }

local function login()
  local left = { section("Login screen") }
  for _, part in ipairs(system_look()) do
    left[#left + 1] = part
  end
  left[#left + 1] = note("With more than one display, the main one asks who logs in and the others show the clock.")
  local login_set = function(name, value)
    busy = true
    said = ""
    boot_said = "Waiting for your password…"
    render()
    noctalia.runAsync({ "kavat-login-settings", "set", name .. "=" .. value }, function(result)
      busy = false
      boot_said = ""
      if result.exitCode ~= 0 then
        said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-login%-settings: ", "")
        if said == "" then said = "The password was not given, so nothing changed." end
      else
        news = "Changed. It shows the next time the login screen does."
      end
      read_boot()
    end)
  end
  local who, who_at = { "Nobody: the password is asked" }, 1
  for index, name in ipairs(login_now.people) do
    who[#who + 1] = name
    if name == login_now.auto then
      who_at = index + 1
    end
  end
  left[#left + 1] = section("Who logs in")
  left[#left + 1] = row("Clock", segmented({
    { text = "24 hours", on = login_now.clock ~= "12", pick = function() login_set("clock", "24") end },
    { text = "12 hours", on = login_now.clock == "12", pick = function() login_set("clock", "12") end },
  }))
  left[#left + 1] = row("Show who can log in", ui.row({ justify = "end", flexGrow = 1 }, {
    ui.toggle({ checked = login_now.users ~= "no", enabled = not busy, onChange = function(now) login_set("users", now == "true" and "yes" or "no") end }) }))
  left[#left + 1] = row("Logs in by itself", select(who, who_at, function(index) login_set("auto", index == 1 and "none" or who[index]) end))
  left[#left + 1] = note("With nobody shown, the name is typed too. Someone who logs in by itself does so once, as the machine starts, and the keyring, which the password typed at the login opens, then asks for it once the desktop is up.")
  return { title = "Login & lock", sub = "kavat-greeter · hyprlock", file = "~/.config/hypr/hypridle.conf", left = left, right = {
    section("Lock"),
    row("Lock after", conf_select("lock", "lock_after", MINUTES)),
    row("Screen off after", conf_select("lock", "screen_off_after", MINUTES)),
    note("Super+Shift+L locks at once, and the screen locks before the machine sleeps. The lock screen follows the desktop's theme by itself."),
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
      end }) or ui.button({ text = "Bring LACT back", variant = "outline", controlSize = "sm", onClick = function()
        -- it comes with the system; this is for a system it was taken out of
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
  local content = { title = "Browser", sub = "Kavat Browser · Chromium, made the system's own", file = "~/.config/kavat/browser/commands.toml", left = {
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
    section("New tab"),
    row("Background", segmented({
      { text = "Theme", on = browser.background == "" or browser.background == "theme", pick = function()
        noctalia.runAsync({ "kavat-browser", "background", "theme" }, function() read_browser() end) end },
      { text = "Wallpaper", on = browser.background == "wallpaper", pick = function()
        noctalia.runAsync({ "kavat-browser", "background", "wallpaper" }, function() read_browser() end) end },
      { text = "A picture…", on = browser.background ~= "" and browser.background ~= "theme" and browser.background ~= "wallpaper", pick = function()
        noctalia.runInTerminal("chosen=$(mktemp); yazi --chooser-file=\"$chosen\" ~/Pictures; [ -s \"$chosen\" ] && kavat-browser background \"$(head -n 1 \"$chosen\")\"; rm -f \"$chosen\"")
        panel.close()
      end },
    })),
    note("What a new tab shows behind its clock: the theme's colour, the desktop's background, or a picture of yours, picked in Yazi. The new tab has a Background button of its own, at its foot, for the same."),
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
  local right = content.right
  right[#right + 1] = section("YouTube")
  right[#right + 1] = row("Hide Shorts", ui.row({ justify = "end", flexGrow = 1 }, {
    ui.toggle({ checked = browser.shorts ~= "on", enabled = not busy, onChange = function(now)
      noctalia.runAsync({ "kavat-browser", "shorts", now == "true" and "off" or "on" }, function() read_browser() end)
    end }) }))
  right[#right + 1] = section("Pages installed as apps")
  for _, app in ipairs(browser.apps) do
    right[#right + 1] = ui.row({ key = "app-" .. app.name, align = "center", gap = 8, paddingH = 10, paddingV = 4, minHeight = 36, radius = 8, fill = "surface_container" }, {
      ui.column({ gap = 1, flexGrow = 1 }, {
        ui.label({ text = app.name, maxWidth = 300 }),
        ui.label({ text = app.address, fontSize = 11, color = "on_surface_variant", maxWidth = 300 }),
      }),
      ui.button({ text = "Remove", variant = "ghost", controlSize = "sm", onClick = function()
        noctalia.runAsync({ "kavat-webapp", "remove", app.name }, function() read_browser() end)
      end }),
    })
  end
  right[#right + 1] = ui.row({ gap = 8 }, { ui.button({ text = "+ Install a page as an app", variant = "outline", controlSize = "sm", onClick = function()
    noctalia.runInTerminal("kavat-webapp install; echo; read -r -p 'Enter closes this window. ' _")
    panel.close()
  end }) })
  right[#right + 1] = note("An app is a page in a window of its own, with no tabs and no address bar, found in the launcher. In the browser, Ctrl+Shift+K and \"install app\" installs the page in view.")
  return content
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
        noctalia.runInTerminal("sudo systemctl enable --now opensnitchd && (setsid -f opensnitch-ui > /dev/null 2>&1); echo; read -r -n 1 -s -p 'Any key closes.'")
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
    row("More completions", conf_toggle("shell", "more_completions", "yes", "no")),
    row("Esc twice puts sudo", conf_toggle("shell", "esc_sudo", "yes", "no")),
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
  local own = nil  -- the image of the display being chosen for, when it has one
  for _, display in ipairs(walls.displays) do
    if display.name == wall_display then
      own = display.file
    end
  end
  for _, item in ipairs(walls.list) do
    if wall_filter == "all" or item.kind == wall_filter or item.kind == "theme" then
      local on = (item.kind == "theme" and walls.kind == "theme") or item.file == walls.file
      if wall_display ~= "" then
        on = own ~= nil and item.file == own or own == nil and on
      end
      tiles[#tiles + 1] = ui.column({ key = "wall-" .. item.file, gap = 4, width = 178 }, {
        ui.image({ path = item.thumb, width = 178, height = 100, radius = 8, fit = "cover",
          border = on and "primary" or "outline_variant", borderWidth = on and 3 or 1,
          onClick = function()
            if wall_display ~= "" then
              if item.file == walls.file or (item.kind == "theme" and walls.kind == "theme") then
                wall_does("same", wall_display)
              else
                wall_does("set", item.file, wall_display)
              end
            elseif item.kind == "theme" then
              wall_does("theme")
            else
              wall_does("set", item.file)
            end
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
  -- Online: the words go to the sources that are on, those of images, those of live
  -- wallpapers or both, as the filter above says, and what they answer is drawn; a press
  -- downloads the image or the video to the library and sets it. Nothing is asked until
  -- Search is pressed.
  local search = function()
    if wall_words == "" or wall_searching then
      return
    end
    wall_searching, wall_found, said = true, {}, ""
    render()
    local command = { "kavat-wallpaper", "search" }
    if wall_filter ~= "all" then
      command[#command + 1] = wall_filter == "video" and "--live" or "--images"
    end
    command[#command + 1] = wall_words
    wall_asked = wall_filter
    noctalia.runAsync(command, function(result)
      wall_searching = false
      for line in result.stdout:gmatch("[^\n]+") do
        local source, id, small, full, size, file, kind = line:match("^([^\t]*)\t([^\t]*)\t([^\t]*)\t([^\t]*)\t([^\t]*)\t([^\t]*)\t([^\t]*)$")
        if file and file ~= "-" and full ~= "-" then
          wall_found[#wall_found + 1] = { source = source, name = id, full = full, size = size, file = file, kind = kind }
        end
      end
      if result.exitCode ~= 0 then
        said = trim(result.stderr):gsub("^kavat%-wallpaper: ", "")
      elseif #wall_found == 0 then
        said = "Nothing was found for those words."
      end
      render()
    end)
  end
  local online = {
    ui.row({ gap = 8, align = "center" }, {
      section("Online"),
      ui.input({ key = "wall-words", placeholder = wall_filter == "video" and "Search live wallpapers on the network: rain, city at night…"
          or wall_filter == "image" and "Search images on the network: mountains, city at night…"
          or "Search wallpapers on the network, images and live: mountains, city at night…", controlSize = "sm", flexGrow = 1, value = wall_words,
        onChange = function(text) wall_words = text end, onSubmit = search }),
      ui.button({ text = wall_searching and "Searching…" or "Search", variant = "outline", controlSize = "sm", enabled = not wall_searching, onClick = search }),
      #wall_found > 0 and ui.button({ text = "✕", variant = "ghost", controlSize = "sm", onClick = function() wall_found = {}; render() end }) or nil,
    }),
  }
  -- what was found is filtered as the library is
  local found = {}
  for _, picture in ipairs(wall_found) do
    if wall_filter == "all" or picture.kind == wall_filter then
      found[#found + 1] = picture
    end
  end
  if #wall_found > 0 and #found == 0 then
    online[#online + 1] = note("The search was for " .. (wall_asked == "video" and "live wallpapers" or "images") .. ": press Search again for " .. (wall_filter == "video" and "live ones." or "images."))
  end
  for at = 1, #found, 5 do
    local line = {}
    for index = at, math.min(at + 4, #found) do
      local picture = found[index]
      line[#line + 1] = ui.column({ key = "online-" .. picture.file, gap = 4, width = 178 }, {
        ui.image({ path = picture.file, width = 178, height = 100, radius = 8, fit = "cover", border = "outline_variant", borderWidth = 1,
          onClick = function() wall_does("fetch", picture.full) end }),
        ui.row({ gap = 6 }, {
          ui.label({ text = picture.kind == "video" and "▶ Live" or picture.size ~= "-" and picture.size or "", fontSize = 11, color = "on_surface_variant" }),
          ui.spacer({ flexGrow = 1 }),
          ui.label({ text = picture.source, fontSize = 11, color = "on_surface_variant" }),
        }),
      })
    end
    online[#online + 1] = ui.row({ gap = 12 }, line)
  end
  top = ui.column({ gap = 12 }, { top, ui.column({ gap = 10 }, online) })
  if #walls.displays > 1 then
    local choices = { { text = "All displays", on = wall_display == "", pick = function() wall_display = ""; render() end } }
    for _, display in ipairs(walls.displays) do
      choices[#choices + 1] = { text = display.name .. (display.file and " •" or ""), on = wall_display == display.name,
        pick = function() wall_display = display.name; render() end }
    end
    top = ui.column({ gap = 12 }, {
      ui.row({ gap = 12, align = "center" }, { ui.label({ text = "Background of", color = "on_surface_variant" }), ui.row({ width = 150 * #choices }, { segmented(choices) }) }),
      top,
    })
  end
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
    row("The lock shows it", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = (walls.lock or "background") ~= "theme", enabled = not busy, onChange = function(now) wall_does("option", "lock", now == "true" and "background" or "theme") end }),
    })),
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
    section("Slideshow"),
    row("Next image", segmented({
      { text = "Off", on = (walls.slideshow or "off") == "off", pick = function() wall_does("option", "slideshow", "off") end },
      { text = "5 min", on = walls.slideshow == "5", pick = function() wall_does("option", "slideshow", "5") end },
      { text = "15", on = walls.slideshow == "15", pick = function() wall_does("option", "slideshow", "15") end },
      { text = "30", on = walls.slideshow == "30", pick = function() wall_does("option", "slideshow", "30") end },
      { text = "1 h", on = walls.slideshow == "60", pick = function() wall_does("option", "slideshow", "60") end },
    })),
    note("The images of the library are shown one after another, in the order of their names, on every display. With more than one display, each can have an image or a video of its own, chosen above."),
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

-- The outputs, the inputs and what is playing, as kavat-sound lists them
local sound_now = { outputs = {}, inputs = {}, apps = {} }

local function read_sound()
  noctalia.runAsync({ "kavat-sound" }, function(result)
    local found = { outputs = {}, inputs = {}, apps = {} }
    for line in result.stdout:gmatch("[^\n]+") do
      local parts = {}
      for part in (line .. "\t"):gmatch("([^\t]*)\t") do
        parts[#parts + 1] = part
      end
      if parts[1] == "output" or parts[1] == "input" then
        local list = parts[1] == "output" and found.outputs or found.inputs
        list[#list + 1] = { name = parts[2], what = parts[3], default = parts[4] == "default", volume = tonumber(parts[5]) or 0, muted = parts[6] == "muted" }
      elseif parts[1] == "app" then
        found.apps[#found.apps + 1] = { number = parts[2], name = parts[3], volume = tonumber(parts[4]) or 0, muted = parts[5] == "muted" }
      end
    end
    sound_now = found
    render()
  end)
end

local function sound_does(...)
  said = ""
  noctalia.runAsync({ "kavat-sound", ... }, function(result)
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-sound: ", "")
    end
    read_sound()
  end)
end

-- The device in use of a kind, picked from a list, with its volume and whether it is muted
local function sound_device(kind, list, volume_name)
  local names, at, chosen = {}, 1, nil
  for index, device in ipairs(list) do
    names[index] = device.what
    if device.default then
      at, chosen = index, device
    end
  end
  if #list == 0 then
    return { note(kind == "output" and "No output: nothing to play through." or "No microphone.") }
  end
  chosen = chosen or list[1]
  return {
    row(kind == "output" and "Device" or "Microphone", select(names, at, function(index) sound_does(kind, list[index].name) end)),
    row(volume_name, ui.row({ gap = 10, align = "center", flexGrow = 1 }, {
      ui.slider({ key = kind .. "-volume", min = 0, max = 100, step = 1, value = math.min(chosen.volume, 100), flexGrow = 1, controlSize = "sm",
        onDragEnd = function(now) sound_does("volume", kind, tostring(math.floor(now + 0.5))) end }),
      ui.label({ text = chosen.volume .. "%", fontSize = 12, color = "on_surface_variant", width = 40 }),
    })),
    row("Muted", ui.row({ justify = "end", flexGrow = 1 }, {
      ui.toggle({ checked = chosen.muted, onChange = function(now) sound_does("mute", kind, now == "true" and "on" or "off") end }),
    })),
  }
end

local function sound()
  local left, right = { section("Output") }, { section("Playing now") }
  for _, part in ipairs(sound_device("output", sound_now.outputs, "Volume")) do
    left[#left + 1] = part
  end
  left[#left + 1] = section("Input")
  for _, part in ipairs(sound_device("input", sound_now.inputs, "Level")) do
    left[#left + 1] = part
  end
  left[#left + 1] = row("Noise removed", conf_toggle("sound", "noise_suppression", "on", "off"))
  left[#left + 1] = note("Adds a microphone to the list, \"Microphone, noise removed\", that lets the voice through and leaves out keys, fans and the room. Changing this restarts the sound for a moment.")
  if #sound_now.apps == 0 then
    right[#right + 1] = note("Nothing is playing. An app that plays something shows here, with a volume of its own.")
  end
  for _, app in ipairs(sound_now.apps) do
    right[#right + 1] = ui.row({ key = "app-" .. app.number, align = "center", gap = 10, paddingH = 10, minHeight = 38, radius = 8, fill = "surface_container" }, {
      ui.label({ text = app.name, width = 170, maxWidth = 170 }),
      ui.slider({ key = "app-volume-" .. app.number, min = 0, max = 100, step = 1, value = math.min(app.volume, 100), flexGrow = 1, controlSize = "sm",
        onDragEnd = function(now) sound_does("app", app.number, tostring(math.floor(now + 0.5))) end }),
      ui.label({ text = app.volume .. "%", fontSize = 12, color = "on_surface_variant", width = 40 }),
    })
  end
  right[#right + 1] = ui.row({ gap = 8 }, { ui.button({ text = "Read again", variant = "outline", controlSize = "sm", onClick = function() read_sound() end }) })
  right[#right + 1] = note("The same controls are a press away on the bar, in the volume's own panel.")
  return { title = "Sound", sub = "PipeWire", file = "~/.config/pipewire/pipewire.conf.d/", left = left, right = right }
end

-- The connections and the Bluetooth devices, as kavat-network lists them
local net = { wired = {}, wifi = "none", wifi_startup = true, networks = {}, vpns = {}, proton = false, proton_incognito = false, bluetooth = "none", bluetooth_startup = true, devices = {} }
local net_asking = ""    -- the Wi-Fi network whose password is being typed
local net_password = ""
local net_user = ""
local net_working = ""   -- what is being waited for: a scan, a pairing

local function read_net()
  noctalia.runAsync({ "kavat-network" }, function(result)
    local found = { wired = {}, wifi = "none", wifi_startup = true, networks = {}, vpns = {}, proton = false, proton_incognito = false, bluetooth = "none", bluetooth_startup = true, devices = {} }
    for line in result.stdout:gmatch("[^\n]+") do
      local parts = {}
      for part in (line .. "\t"):gmatch("([^\t]*)\t") do
        parts[#parts + 1] = part
      end
      if parts[1] == "wired" then
        found.wired[#found.wired + 1] = { device = parts[2], state = parts[3], speed = parts[4] }
      elseif parts[1] == "wifi" then
        found.wifi, found.wifi_startup = parts[2], parts[3] == "on"
      elseif parts[1] == "network" then
        found.networks[#found.networks + 1] = { name = parts[2], signal = tonumber(parts[3]) or 0, locked = parts[4] == "locked",
          saved = parts[5] == "saved", connected = parts[6] == "connected" }
      elseif parts[1] == "vpn" then
        found.vpns[#found.vpns + 1] = { name = parts[2], kind = parts[3], up = parts[4] == "up", incognito = parts[5] == "incognito", login = parts[6] == "login" }
      elseif parts[1] == "proton" then
        found.proton, found.proton_incognito = parts[2] == "installed", parts[3] == "incognito"
      elseif parts[1] == "bluetooth" then
        found.bluetooth, found.bluetooth_startup = parts[2], parts[3] == "on"
      elseif parts[1] == "device" then
        found.devices[#found.devices + 1] = { address = parts[2], name = parts[3], state = parts[4], battery = parts[5] }
      end
    end
    net = found
    render()
  end)
end

-- Ask kavat-network for something; `working` is what the page says while it takes
local function net_does(working, ...)
  busy = true
  said = ""
  news = ""
  net_working = working or ""
  render()
  noctalia.runAsync({ "kavat-network", ... }, function(result)
    busy = false
    net_working = ""
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-network: ", "")
    elseif trim(result.stdout) ~= "" then
      news = trim(result.stdout)
    end
    read_net()
  end)
end

-- One thing of a list: its name, what is said of it, and the buttons that act on it
local function net_line(key, name, state, good, buttons)
  local parts = {
    ui.column({ gap = 1, flexGrow = 1 }, {
      ui.label({ text = name, maxWidth = 300 }),
      ui.label({ text = state, fontSize = 11, color = good and "primary" or "on_surface_variant" }),
    }),
  }
  for _, button in ipairs(buttons or {}) do
    parts[#parts + 1] = ui.button({ key = key .. "-" .. button[1], text = button[1], controlSize = "sm", variant = button[3] or "ghost",
      enabled = not busy, onClick = button[2] })
  end
  return ui.row({ key = key, align = "center", gap = 8, paddingH = 10, paddingV = 5, minHeight = 40, radius = 8, fill = "surface_container" }, parts)
end

local function switch(checked, change)
  return ui.row({ justify = "end", flexGrow = 1 }, { ui.toggle({ checked = checked, enabled = not busy, onChange = function(now) change(now == "true") end }) })
end

local function network()
  local left, right = { section("Wired") }, { section("Bluetooth") }
  if #net.wired == 0 then
    left[#left + 1] = note("No cable connection on this machine.")
  end
  for _, wired in ipairs(net.wired) do
    left[#left + 1] = net_line("wired-" .. wired.device, wired.device .. (wired.speed ~= "-" and " · " .. wired.speed or ""), wired.state, wired.state == "connected")
  end

  left[#left + 1] = section("Wi-Fi")
  if net.wifi == "none" then
    left[#left + 1] = note("This machine has no Wi-Fi.")
  else
    left[#left + 1] = row("Enabled", switch(net.wifi == "on", function(on) net_does(nil, "wifi", on and "on" or "off") end))
    left[#left + 1] = row("On at startup", switch(net.wifi_startup, function(on) net_does(nil, "wifi", "startup", on and "on" or "off") end))
    for _, network in ipairs(net.networks) do
      local state = network.connected and "connected" or (network.signal > 0 and (network.signal .. "%") or "not in reach") .. (network.saved and " · saved" or network.locked and " · locked" or "")
      local buttons = {}
      if network.connected then
        buttons[1] = { "Forget", function() net_does(nil, "wifi", "forget", network.name) end }
      elseif network.signal > 0 then
        buttons[1] = { "Connect", function()
          if network.locked and not network.saved then
            net_asking, net_password = network.name, ""
            render()
          else
            net_does("Connecting to " .. network.name .. "…", "wifi", "connect", network.name)
          end
        end, "outline" }
      end
      if network.saved and not network.connected then
        buttons[#buttons + 1] = { "Forget", function() net_does(nil, "wifi", "forget", network.name) end }
      end
      left[#left + 1] = net_line("wifi-" .. network.name, network.name, state, network.connected, buttons)
      if net_asking == network.name then
        local connect = function()
          local name, password = net_asking, net_password
          net_asking, net_password = "", ""
          net_does("Connecting to " .. name .. "…", "wifi", "connect", name, password)
        end
        left[#left + 1] = ui.row({ key = "wifi-password", gap = 8, align = "center" }, {
          ui.input({ key = "wifi-password-" .. network.name, placeholder = "Password of " .. network.name, password = true, focus = true, controlSize = "sm", flexGrow = 1,
            onChange = function(text) net_password = text end, onSubmit = connect }),
          ui.button({ text = "Connect", variant = "primary", controlSize = "sm", onClick = connect }),
          ui.button({ text = "Cancel", variant = "ghost", controlSize = "sm", onClick = function() net_asking = ""; render() end }),
        })
      end
    end
    if net.wifi == "on" then
      left[#left + 1] = ui.row({ gap = 8 }, { ui.button({ text = "Look again", variant = "outline", controlSize = "sm", enabled = not busy,
        onClick = function() net_does("Looking for networks…", "wifi", "scan") end }) })
    end
  end

  left[#left + 1] = section("VPN")
  for _, vpn in ipairs(net.vpns) do
    local sign_in = function() net_asking, net_user, net_password = "vpn:" .. vpn.name, "", ""; render() end
    local buttons = { vpn.login and { "Sign in", sign_in, "primary" }
      or { vpn.up and "Disconnect" or "Connect", function() net_does((vpn.up and "Disconnecting " or "Connecting ") .. vpn.name .. "…", "vpn", vpn.up and "down" or "up", vpn.name) end, "outline" } }
    if not vpn.incognito then
      buttons[#buttons + 1] = { "Use for Incognito", function() net_does(nil, "vpn", "use", vpn.name) end }
    end
    buttons[#buttons + 1] = { "Remove", function() net_does(nil, "vpn", "remove", vpn.name) end }
    left[#left + 1] = net_line("vpn-" .. vpn.name, vpn.name .. " · " .. (vpn.kind == "wireguard" and "WireGuard" or vpn.kind == "openvpn" and "OpenVPN" or vpn.kind),
      (vpn.up and "connected" or vpn.login and "asks for your name and password" or "saved") .. (vpn.incognito and " · used by Incognito" or ""), vpn.up, buttons)
    if net_asking == "vpn:" .. vpn.name then
      local keep = function()
        local user, password = net_user, net_password
        if user == "" or password == "" then
          return
        end
        net_asking, net_user, net_password = "", "", ""
        net_does(nil, "vpn", "login", vpn.name, user, password)
      end
      left[#left + 1] = ui.row({ key = "vpn-login", gap = 8, align = "center" }, {
        ui.input({ key = "vpn-user-" .. vpn.name, placeholder = "Name", focus = true, controlSize = "sm", flexGrow = 1, onChange = function(text) net_user = text end }),
        ui.input({ key = "vpn-password-" .. vpn.name, placeholder = "Password", password = true, controlSize = "sm", flexGrow = 1,
          onChange = function(text) net_password = text end, onSubmit = keep }),
        ui.button({ text = "Keep", variant = "primary", controlSize = "sm", onClick = keep }),
        ui.button({ text = "Cancel", variant = "ghost", controlSize = "sm", onClick = function() net_asking = ""; render() end }),
      })
      left[#left + 1] = note("The name and the password your provider gave for OpenVPN, which are not always those of its site. They are kept by NetworkManager, where only root reads them.")
    end
  end
  -- Proton VPN's own app, with its login: it makes its connections itself, so it is a line of its own
  if net.proton then
    local buttons = { { "Open", function() noctalia.runAsync("setsid -f protonvpn-app > /dev/null 2>&1"); panel.close() end, "outline" } }
    if not net.proton_incognito then
      buttons[#buttons + 1] = { "Use for Incognito", function() net_does(nil, "vpn", "use", "proton") end }
    end
    left[#left + 1] = net_line("vpn-proton-app", "Proton VPN · its own app", "signed in and connected from the app" .. (net.proton_incognito and " · used by Incognito" or ""), false, buttons)
  end
  left[#left + 1] = ui.row({ gap = 8 }, { ui.button({ text = "+ Add a VPN", variant = #net.vpns == 0 and "primary" or "outline", controlSize = "sm", onClick = function()
    noctalia.runInTerminal("kavat-network vpn add; echo; read -r -p 'Enter closes this window. ' _")
    panel.close()
  end }), not net.proton and ui.button({ text = "Install Proton VPN", variant = "outline", controlSize = "sm", onClick = function()
    noctalia.runInTerminal("kav install proton-vpn-gtk-app proton-vpn-cli; echo; read -r -p 'Enter closes this window. ' _")
    panel.close()
  end }) or nil })
  left[#left + 1] = note(#net.vpns == 0 and not net.proton
    and "Incognito mode sends everything through a VPN, and there is none yet. Your provider gives a file for it on its site: at Proton VPN, account.protonvpn.com, Downloads, WireGuard configuration; Mullvad and the others have the same. Download it, press Add a VPN and pick it. Proton VPN's own app, with its login, can be installed and used instead."
    or "A VPN is added from the file its provider gives, WireGuard or OpenVPN. The one Incognito uses comes up when that mode is entered. One whose name or password was typed wrong is removed and added again.")

  if net.bluetooth == "none" then
    right[#right + 1] = note("This machine has no Bluetooth, or its service is not running.")
  else
    right[#right + 1] = row("Enabled", switch(net.bluetooth == "on", function(on) net_does(nil, "bluetooth", on and "on" or "off") end))
    right[#right + 1] = row("On at startup", switch(net.bluetooth_startup, function(on) net_does(nil, "bluetooth", "startup", on and "on" or "off") end))
    right[#right + 1] = section("Devices")
    if #net.devices == 0 then
      right[#right + 1] = note(net.bluetooth == "on" and "No device yet. Put one in pairing mode and press Pair a device." or "Bluetooth is off.")
    end
    for _, device in ipairs(net.devices) do
      local state = device.state == "near" and "nearby" or device.state .. (device.battery ~= "-" and " · battery " .. device.battery .. "%" or "")
      local buttons
      if device.state == "connected" then
        buttons = { { "Disconnect", function() net_does(nil, "bluetooth", "disconnect", device.address) end }, { "Forget", function() net_does(nil, "bluetooth", "forget", device.address) end } }
      elseif device.state == "paired" then
        buttons = { { "Connect", function() net_does("Connecting to " .. device.name .. "…", "bluetooth", "connect", device.address) end, "outline" },
          { "Forget", function() net_does(nil, "bluetooth", "forget", device.address) end } }
      else
        buttons = { { "Pair", function() net_does("Pairing with " .. device.name .. "…", "bluetooth", "pair", device.address) end, "outline" },
          { "With a code", function() net_asking, net_password = "bt:" .. device.address, ""; render() end } }
      end
      right[#right + 1] = net_line("bt-" .. device.address, device.name, state, device.state == "connected", buttons)
      if net_asking == "bt:" .. device.address then
        local pair = function()
          local code = net_password
          if code == "" then
            return
          end
          net_asking, net_password = "", ""
          net_does("Pairing with " .. device.name .. "…", "bluetooth", "pair", device.address, code)
        end
        right[#right + 1] = ui.row({ key = "bt-code", gap = 8, align = "center" }, {
          ui.input({ key = "bt-code-" .. device.address, placeholder = "Code, often 0000", focus = true, controlSize = "sm", flexGrow = 1,
            onChange = function(text) net_password = text end, onSubmit = pair }),
          ui.button({ text = "Pair", variant = "primary", controlSize = "sm", onClick = pair }),
          ui.button({ text = "Cancel", variant = "ghost", controlSize = "sm", onClick = function() net_asking = ""; render() end }),
        })
      end
    end
    if net.bluetooth == "on" then
      right[#right + 1] = ui.row({ gap = 8 }, { ui.button({ text = "Pair a device", variant = "primary", controlSize = "sm", enabled = not busy,
        onClick = function() net_does("Looking for devices nearby for ten seconds…", "bluetooth", "scan") end }) })
      right[#right + 1] = note("Put the device in pairing mode first. The ones found show in the list with a Pair button. A device that shows a code has it said in a notice; one that asks for a code is paired with With a code.")
    end
  end
  if net_working ~= "" then
    right[#right + 1] = ui.label({ text = net_working, color = "primary" })
  end
  right[#right + 1] = note("The firewall and who uses the network are in Privacy & network.")
  return { title = "Network", sub = "connections and Bluetooth devices", file = "~/.config/kavat/network.toml", left = left, right = right }
end

-- Remote access: whether this computer is reached from your other devices, as kavat-remote
-- says it. Turning it on or off, and its parts, ask for an administrator's password.
local remote = { state = "off", parts = {}, server = "default", network = "down", address = "-", connected = {}, keys = {}, paired = {} }
local remote_typed = { key = "", pin = "", server = "" }
local remote_qr = ""  -- the picture of the address to sign in at, while there is one

local function read_remote()
  noctalia.runAsync("command -v kavat-remote > /dev/null || { echo missing; exit 0; }; kavat-remote", function(result)
    local found = { state = "off", missing = trim(result.stdout) == "missing", parts = {}, server = "default", network = "down", address = "-", connected = {}, keys = {}, paired = {} }
    for line in result.stdout:gmatch("[^\n]+") do
      local parts = {}
      for part in (line .. "\t"):gmatch("([^\t]*)\t") do
        parts[#parts + 1] = part
      end
      local what = parts[1]
      if what == "state" then
        found.state = parts[2]
      elseif what == "terminal" or what == "desktop" or what == "awake" then
        found.parts[what] = { on = parts[2] == "yes", up = parts[3] == "up" }
      elseif what == "server" then
        found.server = parts[2]
      elseif what == "network" then
        found.network, found.address = parts[2], parts[3] or "-"
      elseif what == "self" then
        found.name, found.own = parts[2], parts[3]
      elseif what == "connected" then
        found.connected[#found.connected + 1] = { device = parts[2], address = parts[3], through = parts[4] }
      elseif what == "key" then
        found.keys[#found.keys + 1] = { number = parts[2], kind = parts[3], comment = parts[4], tail = parts[5] }
      elseif what == "paired" then
        found.paired[#found.paired + 1] = { number = parts[2], name = parts[3] }
      end
    end
    remote = found
    if found.network == "login" and found.address ~= "-" then
      -- the address to sign in at, as a picture a phone can read
      local file = (os.getenv("XDG_RUNTIME_DIR") or "/tmp") .. "/kavat-remote-qr.png"
      noctalia.runAsync({ "kavat-remote", "login", "qr", file }, function(made)
        remote_qr = made.exitCode == 0 and file or ""
        render()
      end)
    else
      remote_qr = ""
    end
    render()
  end)
end

local function remote_does(working, ...)
  busy = true
  said = ""
  news = working or ""
  render()
  noctalia.runAsync({ "kavat-remote", ... }, function(result)
    busy = false
    news = ""
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-remote: ", "")
      if said == "" then said = "The password was not given, so nothing changed." end
    elseif trim(result.stdout) ~= "" then
      news = trim(result.stdout)
    end
    read_remote()
  end)
end

local function pages_remote()
  if remote.missing then
    return { title = "Remote access", sub = "not installed", left = {
      note("Remote access is not part of this system yet. It is kavat-remote, over Tailscale, the SSH server and Sunshine, and it comes with an update once Sunshine is built for the system."),
    }, right = {} }
  end
  local on = remote.state ~= "off"
  local part = function(name, label)
    local this = remote.parts[name] or { on = false, up = false }
    return row(label, ui.row({ justify = "end", align = "center", gap = 8, flexGrow = 1 }, {
      ui.label({ text = on and (this.up and "running" or this.on and "not running" or "") or "", fontSize = 11, color = this.up and "primary" or "on_surface_variant" }),
      ui.toggle({ checked = this.on, enabled = not busy, onChange = function(now) remote_does("Waiting for your password…", "set", name .. "=" .. (now == "true" and "yes" or "no")) end }),
    }))
  end
  local left = {
    section("Remote access"),
    row("Enabled", switch(on, function(now) remote_does("Waiting for your password…", now and "on" or "off") end)),
    note(remote.state == "paused" and "It is paused while Incognito is on, and comes back on leaving."
      or on and "This computer is reached from your own devices, through your private network and from nowhere else."
      or "Off: nothing of it runs, and the firewall has no door for it. Turning it on joins your Tailscale network, opens the terminal and the desktop to that network only, and keeps the computer awake."),
    section("What is open"),
    part("terminal", "Terminal · SSH"),
    part("desktop", "Desktop · Sunshine"),
    part("awake", "Stay awake"),
    note("The terminal takes keys only: no passwords and no root. The desktop is seen from Moonlight, once someone has logged in at this computer. Awake keeps it from suspending; the screen still locks."),
    section("Private network · Tailscale"),
  }
  if not on then
    left[#left + 1] = note("Joined when remote access is turned on.")
  elseif remote.network == "up" then
    left[#left + 1] = row("This computer", ui.label({ text = (remote.name or "?") .. " · " .. (remote.own or ""), fontFamily = "JetBrains Mono", fontSize = 12 }))
  elseif remote.network == "login" then
    left[#left + 1] = note("Sign in to join it, at this address or with the picture, from any device:")
    left[#left + 1] = ui.label({ text = remote.address, fontFamily = "JetBrains Mono", fontSize = 12, color = "primary", maxWidth = 440 })
    if remote_qr ~= "" then
      left[#left + 1] = ui.image({ path = remote_qr, width = 170, height = 170, radius = 6 })
    end
    left[#left + 1] = ui.row({ gap = 8 }, { ui.button({ text = "I have signed in", variant = "outline", controlSize = "sm", onClick = function() read_remote() end }) })
  else
    left[#left + 1] = note("The private network is not up.")
  end
  left[#left + 1] = ui.row({ gap = 8, align = "center" }, {
    ui.input({ key = "remote-server", placeholder = remote.server == "default" and "A server of your own, Headscale: https://…" or remote.server, controlSize = "sm", flexGrow = 1,
      onChange = function(text) remote_typed.server = text end }),
    ui.button({ text = "Use it", variant = "ghost", controlSize = "sm", enabled = not busy, onClick = function()
      if remote_typed.server ~= "" then remote_does("Waiting for your password…", "set", "server=" .. remote_typed.server) end
    end }),
    remote.server ~= "default" and ui.button({ text = "Tailscale's", variant = "ghost", controlSize = "sm", enabled = not busy,
      onClick = function() remote_does("Waiting for your password…", "set", "server=default") end }) or nil,
  })

  local right = { section("Connected now") }
  if #remote.connected == 0 then
    right[#right + 1] = note(on and "Nobody." or "Remote access is off.")
  end
  for index, device in ipairs(remote.connected) do
    local desktop = device.through == "desktop"
    right[#right + 1] = net_line("remote-in-" .. index, desktop and "A device is watching the desktop" or device.device, desktop and "desktop" or ("terminal · " .. device.address), true,
      { { "Disconnect", function() remote_does(nil, "disconnect", desktop and "desktop" or device.device) end } })
  end
  right[#right + 1] = section("Keys that may come in to the terminal")
  for _, key in ipairs(remote.keys) do
    right[#right + 1] = net_line("remote-key-" .. key.number, key.comment ~= "-" and key.comment or key.kind, key.kind .. " · …" .. key.tail, false,
      { { "Remove", function() remote_does(nil, "key", "remove", key.number) end } })
  end
  right[#right + 1] = ui.row({ gap = 8, align = "center" }, {
    ui.input({ key = "remote-key", placeholder = "Paste a public key: ssh-ed25519 AAAA… name", controlSize = "sm", flexGrow = 1, onChange = function(text) remote_typed.key = text end }),
    ui.button({ text = "Add", variant = "outline", controlSize = "sm", enabled = not busy, onClick = function()
      if remote_typed.key ~= "" then remote_does(nil, "key", "add", remote_typed.key) end
    end }),
  })
  right[#right + 1] = section("Devices paired for the desktop")
  for _, device in ipairs(remote.paired) do
    right[#right + 1] = net_line("remote-paired-" .. device.number, device.name, "Moonlight", false,
      { { "Forget", function() remote_does(nil, "unpair", device.number) end } })
  end
  right[#right + 1] = ui.row({ gap = 8, align = "center" }, {
    ui.input({ key = "remote-pin", placeholder = "The PIN Moonlight shows", controlSize = "sm", width = 200, onChange = function(text) remote_typed.pin = text end }),
    ui.button({ text = "Pair", variant = "outline", controlSize = "sm", enabled = not busy and on, onClick = function()
      if remote_typed.pin ~= "" then remote_does("Pairing…", "pair", remote_typed.pin) end
    end }),
  })
  right[#right + 1] = note("In Moonlight, add this computer by its name on the private network: it shows a PIN to type here.")
  return { title = "Remote access", sub = on and (remote.state == "paused" and "paused" or "on · your own devices only") or "off", file = "/etc/kavat/remote.conf", left = left, right = right }
end

-- The bar and the notifications are the shell's: what it says of them is read, and changed,
-- through the shell itself, which keeps what is changed in its own file.
local bar_now = { position = "top", margin_edge = 0, thickness = 34, auto_hide = false, start = {}, center = {}, ["end"] = {},
  notify_position = "top_right", notify_focused = false }
local bar_adding = "end"  -- the part of the bar a widget is added to

-- Every widget the bar can show, by its name in the shell's file
local WIDGETS = {
  { "launcher", "Launcher" }, { "workspaces", "Workspaces" }, { "active_window", "Window title" }, { "taskbar", "Open windows" },
  { "clock", "Clock and date" }, { "kavat/modes:mode", "Mode switcher" }, { "kavat/notes:due", "Notes due" }, { "media", "What is playing" },
  { "sysmon", "Processor and memory" }, { "privacy", "Recording indicator" }, { "tray", "Tray" }, { "notifications", "Notifications" },
  { "clipboard", "Clipboard" }, { "keyboard_layout", "Keyboard in use" }, { "kavat/settings:network", "Network" }, { "network", "Network, the shell's own" }, { "bluetooth", "Bluetooth" },
  { "volume", "Volume" }, { "brightness", "Brightness" }, { "battery", "Battery" }, { "session", "Power" },
  { "control-center", "Control center" }, { "wallpaper", "Background picker" }, { "weather", "Weather" }, { "nightlight", "Night light" },
  { "caffeine", "Keep awake" }, { "power_profile", "Power profile" }, { "screenshot", "Screenshot" }, { "theme_mode", "Light or dark" },
  { "lock_keys", "Caps and Num lock" }, { "audio_visualizer", "Sound bars" }, { "settings", "Shell settings" }, { "spacer", "Empty space" },
}

local function widget_name(id)
  for _, widget in ipairs(WIDGETS) do
    if widget[1] == id then
      return widget[2]
    end
  end
  return id
end

local function read_bar()
  noctalia.runAsync("for key in bar.default.position bar.default.margin_edge bar.default.thickness bar.default.auto_hide bar.default.start bar.default.center bar.default.end "
    .. "notification.position notification.follow_focused_output; do echo \"$key = $(kavat-shell msg config-get $key | tr -d '\\n')\"; done", function(result)
    local found = { position = "top", margin_edge = 0, thickness = 34, auto_hide = false, start = {}, center = {}, ["end"] = {},
      notify_position = "top_right", notify_focused = false }
    for line in result.stdout:gmatch("[^\n]+") do
      local key, value = line:match("^(%S+) = (.*)$")
      if key then
        local name = key:match("([%w_]+)$")
        if name == "start" or name == "center" or name == "end" then
          for widget in value:gmatch("'([^']+)'") do
            found[name][#found[name] + 1] = widget
          end
        elseif key == "notification.position" then
          found.notify_position = value:gsub("'", "")
        elseif key == "notification.follow_focused_output" then
          found.notify_focused = value == "true"
        elseif name == "position" then
          found.position = value:gsub("'", "")
        elseif name == "auto_hide" then
          found.auto_hide = value == "true"
        else
          found[name] = tonumber(value) or found[name]
        end
      end
    end
    bar_now = found
    render()
  end)
end

-- Set one of the shell's settings; the value is written as the shell's file writes it
local function bar_set(key, value)
  said = ""
  noctalia.runAsync({ "kavat-shell", "msg", "config-set", key, value }, function(result)
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    else
      keep("Bar: " .. key .. " " .. value)
    end
    read_bar()
  end)
end

local function bar_widgets(part, list)
  local quoted = {}
  for index, widget in ipairs(list) do
    quoted[index] = '"' .. widget .. '"'
  end
  bar_set("bar.default." .. part, "[" .. table.concat(quoted, ", ") .. "]")
end

local PARTS = { { "start", "Left" }, { "center", "Center" }, { "end", "Right" } }

local function bar()
  local vertical = bar_now.position == "left" or bar_now.position == "right"
  local positions = {}
  for _, position in ipairs({ { "top", "Top" }, { "bottom", "Bottom" }, { "left", "Left" }, { "right", "Right" } }) do
    positions[#positions + 1] = { text = position[2], on = bar_now.position == position[1], pick = function() bar_set("bar.default.position", '"' .. position[1] .. '"') end }
  end
  local sizes = {}
  for _, size in ipairs({ { 28, "Compact" }, { 34, "Normal" }, { 42, "Large" } }) do
    sizes[#sizes + 1] = { text = size[2], on = bar_now.thickness == size[1], pick = function() bar_set("bar.default.thickness", tostring(size[1])) end }
  end
  local places, place_names, place_at = { "top_right", "top_center", "top_left", "bottom_right", "bottom_center", "bottom_left" },
    { "Top right", "Top center", "Top left", "Bottom right", "Bottom center", "Bottom left" }, 1
  for index, place in ipairs(places) do
    if place == bar_now.notify_position then
      place_at = index
    end
  end

  -- the widgets, part by part of the bar; each can go up, down, to the next part, or away
  local right, on_bar = { section("Widgets") }, {}
  for at, part in ipairs(PARTS) do
    local list = bar_now[part[1]]
    right[#right + 1] = ui.label({ text = (vertical and ({ "Top", "Middle", "Bottom" })[at] or part[2]), fontSize = 12, fontWeight = "bold", color = "primary" })
    if #list == 0 then
      right[#right + 1] = note("Nothing here.")
    end
    for index, widget in ipairs(list) do
      on_bar[widget] = true
      local function move(to_part, to_index)
        local from, into = {}, nil
        for _, other in ipairs(list) do
          if other ~= widget then from[#from + 1] = other end
        end
        if to_part == part[1] then
          table.insert(from, to_index, widget)
          bar_widgets(part[1], from)
        else
          into = { table.unpack(bar_now[to_part]) }
          table.insert(into, to_index == 0 and #into + 1 or 1, widget)
          -- two changes, one after the other: the widget leaves one part and enters the next
          noctalia.runAsync({ "kavat-shell", "msg", "config-set", "bar.default." .. part[1], "[" .. (function()
            local quoted = {}
            for i, other in ipairs(from) do quoted[i] = '"' .. other .. '"' end
            return table.concat(quoted, ", ")
          end)() .. "]" }, function() bar_widgets(to_part, into) end)
        end
      end
      local key = "widget-" .. part[1] .. "-" .. index
      right[#right + 1] = ui.row({ key = key, align = "center", gap = 4, paddingH = 10, minHeight = 30, radius = 8, fill = "surface_container" }, {
        ui.label({ text = widget_name(widget), flexGrow = 1 }),
        ui.button({ key = key .. "-up", text = "↑", controlSize = "sm", variant = "ghost", tooltip = "Earlier on the bar",
          enabled = index > 1 or at > 1, onClick = function()
            if index > 1 then move(part[1], index - 1) else move(PARTS[at - 1][1], 0) end
          end }),
        ui.button({ key = key .. "-down", text = "↓", controlSize = "sm", variant = "ghost", tooltip = "Later on the bar",
          enabled = index < #list or at < #PARTS, onClick = function()
            if index < #list then move(part[1], index + 1) else move(PARTS[at + 1][1], 1) end
          end }),
        ui.button({ key = key .. "-remove", text = "✕", controlSize = "sm", variant = "ghost", tooltip = "Take it off the bar", onClick = function()
          local rest = {}
          for _, other in ipairs(list) do
            if other ~= widget then rest[#rest + 1] = other end
          end
          bar_widgets(part[1], rest)
        end }),
      })
    end
  end
  local spare, spare_names = {}, { "Add a widget…" }
  for _, widget in ipairs(WIDGETS) do
    if not on_bar[widget[1]] or widget[1] == "spacer" then
      spare[#spare + 1] = widget[1]
      spare_names[#spare_names + 1] = widget[2]
    end
  end
  local parts = {}
  for at, part in ipairs(PARTS) do
    parts[#parts + 1] = { text = vertical and ({ "Top", "Middle", "Bottom" })[at] or part[2], on = bar_adding == part[1], pick = function() bar_adding = part[1]; render() end }
  end

  return { title = "Bar & widgets", sub = "Kavat Shell", file = "~/.local/state/kavat-shell/settings.toml", left = {
    section("Bar"),
    row("Position", segmented(positions)),
    row("Floating", switch(bar_now.margin_edge > 0, function(on) bar_set("bar.default.margin_edge", on and "8" or "0") end)),
    row("Density", segmented(sizes)),
    row("Hides by itself", switch(bar_now.auto_hide, function(on) bar_set("bar.default.auto_hide", on and "true" or "false") end)),
    section("Notifications"),
    row("Position", select(place_names, place_at, function(index) bar_set("notification.position", '"' .. places[index] .. '"') end)),
    row("On the display in use", switch(bar_now.notify_focused, function(on) bar_set("notification.follow_focused_output", on and "true" or "false") end)),
    note("Gamer mode keeps them quiet by itself."),
    section("Add"),
    row("To the", segmented(parts)),
    row("Widget", select(spare_names, 1, function(index)
      if index > 1 then
        local list = { table.unpack(bar_now[bar_adding]) }
        list[#list + 1] = spare[index - 1]
        bar_widgets(bar_adding, list)
      end
    end)),
    section("Everything else"),
    ui.row({ gap = 8 }, { shell_panel("Open the shell's own settings", "settings-open") }),
    note("Each widget's own settings, the launcher, the dock and the desktop's widgets are there. Its page for colours is not for this system: the colours come from Appearance."),
  }, right = right }
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

-- The mouse and the touchpad: what kavat-conf keeps for Hyprland, which reads it again at
-- each change
local function pointer()
  local now = conf.pointer or {}
  local speed = tonumber(now.mouse_speed) or 0
  return { title = "Mouse & touchpad", sub = "Hyprland · changes apply live", file = "~/.config/kavat/desktop.json", left = {
    section("Mouse"),
    row("Acceleration", conf_toggle("pointer", "mouse_acceleration", "true", "false")),
    note("On, the pointer goes further the faster the hand moves. Off, it goes as far as the hand does, whatever the speed: what aiming in a game wants."),
    row("Speed", ui.slider({ min = -10, max = 10, step = 1, value = speed, flexGrow = 1, controlSize = "sm", enabled = not busy,
      onDragEnd = function(to)
        set_conf("pointer", "mouse_speed", string.format("%d", math.max(-10, math.min(10, math.floor((tonumber(to) or 0) + 0.5)))))
      end })),
    row("Scroll the other way", conf_toggle("pointer", "mouse_natural_scroll", "true", "false")),
    row("Left-handed", conf_toggle("pointer", "left_handed", "true", "false")),
    note("Left-handed swaps the two buttons, on the mouse and on the touchpad."),
  }, right = {
    section("Touchpad"),
    row("Tap to click", conf_toggle("pointer", "touchpad_tap", "true", "false")),
    row("Scroll the way the fingers go", conf_toggle("pointer", "touchpad_natural_scroll", "true", "false")),
    row("Off while typing", conf_toggle("pointer", "touchpad_off_while_typing", "true", "false")),
    note("These are for a laptop's touchpad; a machine with none keeps them for when one is plugged in."),
  } }
end

local displays_read = 0
local night = { state = "off", warmth = "4000", from = "20:00", to = "07:00" }  -- the night light, as kavat-displays says it

local function read_displays()
  noctalia.runAsync("kavat-displays; for name in $(kavat-displays | cut -f1); do echo \"== $name\"; kavat-displays modes $name; done; echo '== night light'; kavat-displays night", function(result)
    local found, part = {}, nil
    for line in result.stdout:gmatch("[^\n]+") do
      local heading = line:match("^== (.+)$")
      local name, what, mode, position, scale, turn, spaces = line:match("^([^\t]+)\t([^\t]*)\t([^\t]+)\t([^\t]+)\t([^\t]+)\t([^\t]+)\t([^\t]+)$")
      if heading then
        part = heading
      elseif part == "night light" then
        local state, warmth, from, to = line:match("^(%a+)\t(%d+)\t([%d:]+)\t([%d:]+)$")
        if state then
          night = { state = state, warmth = warmth, from = from, to = to }
        end
      elseif part == nil and name then
        local width, height, rate = mode:match("^(%d+)x(%d+)@([%d.]+)")
        local x, y = position:match("^(-?%d+)x(-?%d+)")
        found[#found + 1] = { name = name, what = what, mode = mode, size = width .. "x" .. height, rate = rate, width = tonumber(width), height = tonumber(height),
          x = tonumber(x), y = tonumber(y), scale = scale, turn = turn, modes = {}, spaces = {} }
        for number in spaces:gmatch("%d+") do
          found[#found].spaces[tonumber(number)] = true
        end
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

-- A change of a display is tried: it stays only when it is kept before ten seconds go by.
-- The seconds left are counted in view, and every change made meanwhile is part of the
-- same try: they are kept, or undone, together.
local display_left = 0     -- the seconds left to keep what is being tried
local display_undone = false
local display_token = 0    -- one for each try, so that the count of an earlier one stops

local function display_count(this)
  noctalia.runAsync("sleep 1", function()
    if display_tried ~= this then
      return
    end
    display_left = display_left - 1
    if display_left <= 0 then
      display_tried = 0
      display_undone = true
      -- kavat-displays undoes it by itself; what is drawn is read again once it has
      noctalia.runAsync("sleep 1.5", function() read_displays() end)
    else
      display_count(this)
    end
    render()
  end)
end

local function display_does(...)
  busy = true
  said = ""
  display_undone = false
  render()
  noctalia.runAsync({ "kavat-displays", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout)
    else
      display_token = display_token + 1
      display_tried = display_token
      display_left = 10
      display_count(display_token)
    end
    read_displays()
  end)
end

-- The night light is not tried as a display's settings are: it leaves nothing black
local function night_does(...)
  busy = true
  said = ""
  render()
  noctalia.runAsync({ "kavat-displays", "night", ... }, function(result)
    busy = false
    if result.exitCode ~= 0 then
      said = trim(result.stderr ~= "" and result.stderr or result.stdout):gsub("^kavat%-displays: ", "")
    end
    read_displays()
  end)
end

local function night_rows()
  local from, to = {}, {}
  for hour = 16, 23 do
    from[#from + 1] = string.format("%02d:00", hour)
    from[#from + 1] = string.format("%02d:30", hour)
  end
  for hour = 4, 10 do
    to[#to + 1] = string.format("%02d:00", hour)
    to[#to + 1] = string.format("%02d:30", hour)
  end
  local function at(list, value)
    for index, one in ipairs(list) do
      if one == value then
        return index
      end
    end
    -- an hour written in the file that the list does not have is shown as it is
    list[#list + 1] = value
    return #list
  end
  local from_at, to_at = at(from, night.from), at(to, night.to)
  return {
    section("Night light · Auto has it on between the two hours"),
    row("Warmer colours", segmented({
      { text = "Off", on = night.state == "off", pick = function() night_does("off") end },
      { text = "On", on = night.state == "on", pick = function() night_does("on") end },
      { text = "Auto", on = night.state == "auto", pick = function() night_does("auto") end },
    })),
    row("Warmth", segmented({
      { text = "Mild", on = night.warmth == "5000", pick = function() night_does("warmth", "5000") end },
      { text = "Warm", on = night.warmth == "4000", pick = function() night_does("warmth", "4000") end },
      { text = "Warmer", on = night.warmth == "3000", pick = function() night_does("warmth", "3000") end },
    })),
    row("From", select(from, from_at, function(index) night_does("hours", from[index], night.to) end)),
    row("Until", select(to, to_at, function(index) night_does("hours", night.from, to[index]) end)),
  }
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
    display_tried > 0 and ui.row({ key = "keep", gap = 14, align = "center", padding = 12, radius = 10, fill = "primary", width = 760 }, {
      ui.column({ gap = 2, flexGrow = 1 }, {
        ui.label({ text = "Keep these display settings?", fontWeight = "bold", fontSize = 15, color = "on_primary" }),
        ui.label({ text = "Everything goes back to how it was in " .. display_left .. (display_left == 1 and " second" or " seconds")
          .. " unless you press Keep. You can change more first: it is all kept together.", color = "on_primary", fontSize = 12 }),
      }),
      ui.button({ text = "Keep", variant = "secondary", onClick = function()
        display_tried = 0
        noctalia.runAsync({ "kavat-displays", "keep" }, function() read_displays() end)
      end }),
      ui.button({ text = "Undo now", variant = "outline", onClick = function()
        display_tried = 0
        noctalia.runAsync({ "kavat-displays", "undo" }, function() read_displays() end)
      end }),
    }) or ui.label({ text = display_undone and "The last change was not kept, so it was undone. Press Keep after a change to make it stay."
        or (#displays > 1 and (chosen.what .. " · drag a display to where it stands") or chosen.what),
      color = display_undone and "error" or "on_surface_variant", fontSize = 12 }),
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
  -- the ten workspaces: the ones of this display lit, the ones of another named in their tooltip
  local spaces = {}
  for number = 1, 10 do
    local owner
    for _, display in ipairs(displays) do
      if display.spaces[number] then
        owner = display.name
      end
    end
    spaces[number] = ui.button({ key = "space-" .. number, text = tostring(number), controlSize = "sm", width = 42,
      variant = owner == chosen.name and "primary" or owner and "outline" or "ghost", enabled = not busy and #displays > 1,
      tooltip = owner == chosen.name and "On this display" or owner and ("On " .. owner) or "On no display in particular",
      onClick = function()
        local mine = {}
        for other = 1, 10 do
          if (chosen.spaces[other] and other ~= number) or (other == number and owner ~= chosen.name) then
            mine[#mine + 1] = tostring(other)
          end
        end
        busy = true
        render()
        noctalia.runAsync({ "kavat-displays", "workspaces", chosen.name, #mine > 0 and table.concat(mine, ",") or "none" }, function(result)
          busy = false
          if result.exitCode ~= 0 then
            said = trim(result.stderr ~= "" and result.stderr or result.stdout)
          end
          read_displays()
        end)
      end })
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
    table.unpack(night_rows()),
  }, right = {
    section("Place"),
    #names > 0 and row("Put it", select(places, 1, function(index)
      if index > 1 then display_does("set", chosen.name, "at=" .. place_values[index]) end
    end)) or note("With one display there is nothing to arrange. A second one is put to the right of this one when it is plugged in."),
    note("Each set of displays keeps its own arrangement, known by what the displays are and not by where they are plugged in."),
    section("Workspaces on " .. chosen.name),
    ui.row({ gap = 4 }, spaces),
    note(#displays > 1 and "The workspaces that open on this display: press a number to give it to this display, or to let it go. A workspace is on one display, so what this one takes the others give up; the first of them is the one the display starts on. One nobody has opens on the display in use."
      or "With one display every workspace is on it. With a second one, each display can have its own."),
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
    -- the ones that are several shortcuts at once, or the mouse's, keep their keys
    local fixed = shortcut.first:find("Arrows", 1, true) or shortcut.first:find("…", 1, true) or shortcut.first:lower():find("mouse", 1, true)
    if key_changing == shortcut.first then
      local keep = function()
        if key_typed ~= "" then
          key_does("set", shortcut.first, key_typed)
        end
      end
      rows[#rows + 1] = ui.row({ key = "key-change", align = "center", gap = 6, minHeight = 30 }, {
        ui.input({ key = "key-input-" .. shortcut.first, placeholder = "Keys for " .. shortcut.what, focus = true, controlSize = "sm", width = 190,
          onChange = function(text) key_typed = text end, onSubmit = keep }),
        ui.button({ text = "Keep", variant = "primary", controlSize = "sm", onClick = keep }),
        ui.button({ text = "No keys", variant = "ghost", controlSize = "sm", onClick = function() key_does("off", shortcut.first) end }),
        ui.button({ text = "As it comes", variant = "ghost", controlSize = "sm", onClick = function() key_does("reset", shortcut.first) end }),
        ui.button({ text = "✕", variant = "ghost", controlSize = "sm", onClick = function() key_changing = ""; render() end }),
      })
    else
      rows[#rows + 1] = ui.row({ key = "key-" .. shortcut.first, align = "center", gap = 12, minHeight = 24, radius = 6,
        onClick = not fixed and function() key_changing, key_typed = shortcut.first, ""; said = ""; render() end or nil }, {
        ui.column({ width = 180, minWidth = 180 }, { ui.label({ text = shortcut.keys, fontFamily = "JetBrains Mono", fontSize = 12,
          color = shortcut.keys == "none" and "on_surface_variant" or "primary", fontWeight = shortcut.changed and "bold" or "normal" }) }),
        ui.label({ text = shortcut.what .. (shortcut.changed and "  · was " .. shortcut.first or ""), fontSize = 13 }),
      })
    end
    half = index
  end
  return { title = "Keybindings", sub = "the desktop's shortcuts · press one to give it other keys", file = "~/.config/hypr/keys.lua",
    left = columns[1], right = columns[2] }
end

local BUILT = { appearance = appearance, wallpaper = wallpaper, accessibility = accessibility, windows = windows, keys = keys,
  displays = pages_displays, sound = sound, network = network, terminal = terminal, shell = shell, editor = editor, files = files, notes = notes,
  bar = bar, modes = pages_modes, backup = pages_backup, gpu = pages_gpu, browser = pages_browser, projects = pages_projects, privacy = pages_privacy, remote = pages_remote, packages = pages_packages, boot = boot, login = login, language = language, pointer = pointer }

local function waiting(entry)
  return { title = entry.name, sub = entry.about or "", left = {
    note("This page is not built yet. It comes with " .. entry.waits .. "."),
  }, right = {} }
end

-- The panel ---------------------------------------------------------------------------

-- What each page has, by its settings' names: made while something is searched for, by
-- drawing every page out of sight, and forgotten when the search is emptied
local inside = nil

local function gather()
  inside = {}
  for id, draw in pairs(BUILT) do
    gathering = {}
    pcall(draw)
    inside[id] = gathering
  end
  gathering = nil
end

-- The first setting of a page that has what is searched for, or nil
local function found_inside(id)
  for _, name in ipairs(inside and inside[id] or {}) do
    if name:lower():find(search, 1, true) then
      return name
    end
  end
  return nil
end

-- The whole panel grows to the size the shell gives it, which on a small display is less
-- than the size it asks for: the list of pages and the page then scroll inside it.
function render()
  if search == "" then
    inside = nil
  else
    -- each time, since what a page has can depend on what was read since
    gather()
  end
  local nav = {
    ui.input({ key = "search", placeholder = "Search settings…", controlSize = "sm", value = search,
      onChange = function(text)
        -- the Displays page has no settings to name until the displays are read
        if search == "" and text ~= "" and #displays == 0 then
          read_displays()
        end
        search = text:lower()
        render()
      end }),
  }
  local shown
  for _, entry in ipairs(PAGES) do
    if entry.id == page then
      shown = entry
    end
    local by_name = search == "" or (entry.name .. " " .. (entry.about or "")):lower():find(search, 1, true)
    local setting = not by_name and found_inside(entry.id) or nil
    if by_name or setting then
      nav[#nav + 1] = ui.row({ key = "nav-" .. entry.id, align = "center", paddingH = 12, minHeight = 26, radius = 6,
        fill = entry.id == page and "surface_container_highest" or "surface_container_low", onClick = function() open_page(entry.id) end }, {
        ui.label({ text = entry.name, flexGrow = 1, fontWeight = entry.id == page and "medium" or "normal",
          color = BUILT[entry.id] and "on_surface" or "on_surface_variant" }),
        ui.label({ text = setting or entry.about or "", fontSize = 11, color = setting and "primary" or "on_surface_variant", maxWidth = 110 }),
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

  panel.render(ui.row({ gap = 0, align = "stretch", flexGrow = 1 }, {
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
  elseif id == "pointer" then
    read_conf("pointer")
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
  elseif id == "remote" then
    read_remote()
  elseif id == "network" then
    read_net()
  elseif id == "bar" then
    read_bar()
  elseif id == "sound" then
    read_conf("sound")
    read_sound()
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
    read_theme()
    read_boot()
  elseif id == "boot" then
    read_theme()
    read_boot()
  elseif id == "keys" then
    read_shortcuts()
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
