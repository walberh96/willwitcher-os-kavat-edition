-- /s in the launcher: the pages of the settings panel, by their name.
local PAGES = {
  { "appearance", "Appearance", "palette, mode, accent, fonts, profiles" },
  { "wallpaper", "Wallpaper", "image or video, colours from it" },
  { "accessibility", "Accessibility", "motion, text size, contrast" },
  { "windows", "Windows", "gaps, corners, borders, blur" },
  { "keys", "Keybindings", "the desktop's shortcuts" },
  { "pointer", "Mouse & touchpad", "acceleration, speed, tap to click" },
  { "displays", "Displays", "resolution, scale, rotation, place" },
  { "gpu", "GPU", "the graphics card's sensors" },
  { "sound", "Sound", "devices, microphone noise" },
  { "network", "Network", "cable, Wi-Fi, VPN, Bluetooth" },
  { "terminal", "Terminal", "Kitty" },
  { "editor", "Editor", "Neovim" },
  { "files", "Files", "Yazi" },
  { "notes", "Notes", "what is due" },
  { "shell", "Shell", "zsh: prompt, completions, history" },
  { "browser", "Browser", "default, console, shield" },
  { "bar", "Bar & widgets", "what the bar shows, and each widget's own settings" },
  { "launcher", "Launcher", "its layout, what it finds" },
  { "notifications", "Notifications", "where they show, the on-screen display" },
  { "panels", "Panels", "control center, clipboard, window switcher, power menu" },
  { "dock", "Dock & desktop", "the dock, desktop widgets, the screen's corners" },
  { "screenshots", "Screenshots", "what Print takes and where it goes" },
  { "modes", "Modes", "Coder, Gamer, Incognito" },
  { "projects", "Projects", "workspaces under a name" },
  { "packages", "Packages", "installed, updates" },
  { "backup", "Backup & sync", "snapshots, backups, history" },
  { "privacy", "Privacy & network", "firewall, who reaches the network" },
  { "remote", "Remote access", "this computer from your other devices" },
  { "boot", "Boot", "the look before the desktop" },
  { "login", "Login & lock", "lock times" },
  { "language", "Language & region", "the keyboard" },
  { "kavatshell", "Kavat Shell", "time and date formats, weather, calendar, hooks" },
}

function onQuery(text)
  local results = {}
  for _, page in ipairs(PAGES) do
    local score = text == "" and 1 or noctalia.fuzzyScore(text, page[2] .. " " .. page[3])
    if score then
      results[#results + 1] = { id = page[1], title = page[2], subtitle = page[3], glyph = "settings", score = score }
    end
  end
  launcher.setResults(text, results)
end

function onActivate(id)
  noctalia.runAsync({ "kavat-shell", "msg", "panel-open", "kavat/settings:panel", id })
end
