-- The system's mode on the bar: the one in force, alone, in the accent. A press opens it
-- into one button for each mode, and a press on one of them changes the mode and closes it
-- again; a press with the right button opens the Modes page. kavat-mode is asked what
-- there is and what is in force, and tells this widget when that changes. While a VPN is
-- up its name stands beside the mode, so that it is seen at a glance that what leaves the
-- machine goes through it.

local modes = {}      -- each: word, name, on
local changing = nil  -- the word of the mode being entered, while it is
local vpn = ""        -- the VPN that is up, when one is
local open = false    -- whether every mode is shown, to choose one

local function draw()
  local buttons = {}
  for _, mode in ipairs(modes) do
    local on = mode.on and changing == nil or changing == mode.word
    if on or open then
      buttons[#buttons + 1] = ui.row({ key = "mode-" .. mode.word, align = "center", paddingH = 9, minHeight = 20, radius = 10,
        fill = on and "primary" or "surface_container",
        onClick = function()
          if changing ~= nil then
            return
          end
          if not open then
            open = true
          elseif mode.on then
            open = false
          else
            open = false
            changing = mode.word
            noctalia.runAsync({ "kavat-mode", mode.word }, function() changing = nil; read() end)
          end
          draw()
        end }, {
        ui.label({ text = mode.name, fontSize = 11, fontWeight = on and "bold" or "normal", color = on and "on_primary" or "on_surface_variant" }),
      })
    end
  end
  if vpn ~= "" then
    buttons[#buttons + 1] = ui.row({ key = "vpn", align = "center", gap = 4, paddingH = 9, minHeight = 20, radius = 10,
      onClick = function() noctalia.runAsync("kavat-shell msg panel-open kavat/settings:panel network") end }, {
      ui.label({ text = barWidget.isVertical() and "VPN" or ("VPN · " .. vpn), fontSize = 11, fontWeight = "bold", color = "primary", maxWidth = 160 }),
    })
  end
  local container = barWidget.isVertical() and ui.column or ui.row
  barWidget.render(container({ gap = 2, align = "center", padding = open and 2 or 0, radius = 12, fill = open and "surface_container" or nil }, buttons))
end

function read()
  noctalia.runAsync({ "kavat-mode", "list" }, function(result)
    if result.exitCode ~= 0 then
      return
    end
    local found = {}
    for mark, word, name in result.stdout:gmatch("([* ]) ([^\t\n]+)\t([^\t\n]*)\t[^\n]*") do
      found[#found + 1] = { word = word, name = name, on = mark == "*" }
    end
    modes = found
    draw()
  end)
  -- the first VPN that is up, by its name in NetworkManager
  noctalia.runAsync("nmcli -t -f TYPE,NAME connection show --active 2>/dev/null | grep -m 1 -E '^(vpn|wireguard):' | cut -d: -f2-", function(result)
    local name = (result.stdout or ""):gsub("%s+$", "")
    if name ~= vpn then
      vpn = name
      draw()
    end
  end)
end

function update() read() end
function onIpc() read() end
function onRightClick() noctalia.runAsync("kavat-shell msg panel-open kavat/settings:panel modes") end

noctalia.setUpdateInterval(5000)
read()
