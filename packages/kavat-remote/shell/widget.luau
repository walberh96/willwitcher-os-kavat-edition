-- Remote access on the bar: a mark for as long as it is on, with how many devices are
-- connected. While it is off, the widget is not there. A press opens the Remote access
-- page. kavat-remote is asked every five seconds; it is also the one that says, in a
-- notification, when a device comes in.

local function show(state, devices)
  if state == "on" then
    barWidget.setGlyph("device-desktop-share")
    barWidget.setText(tostring(#devices))
    local rows = { { "Remote access", #devices == 0 and "on, no device connected"
      or #devices == 1 and "on, 1 device connected" or "on, " .. #devices .. " devices connected" } }
    for _, device in ipairs(devices) do
      rows[#rows + 1] = device
    end
    barWidget.setTooltip(rows)
  elseif state == "login" then
    barWidget.setGlyph("device-desktop-share")
    barWidget.setText("!")
    barWidget.setTooltip("Remote access is on, and this computer is not signed in to Tailscale yet")
  elseif state == "paused" then
    barWidget.setGlyph("device-desktop-share")
    barWidget.setText("")
    barWidget.setTooltip("Remote access is paused while in Incognito")
  end
  barWidget.setVisible(state == "on" or state == "login" or state == "paused")
end

local function read()
  noctalia.runAsync({ "kavat-remote", "bar" }, function(result)
    if result.exitCode ~= 0 then
      barWidget.setVisible(false)
      return
    end
    local state, devices = nil, {}
    for line in result.stdout:gmatch("[^\n]+") do
      if state == nil then
        state = line
      else
        local device, what = line:match("^([^\t]*)\t([^\t]*)$")
        if what == "desktop" then
          devices[#devices + 1] = { "Desktop", "being watched" }
        elseif device then
          devices[#devices + 1] = { device, what }
        end
      end
    end
    show(state or "off", devices)
  end)
end

function update() read() end
function onIpc() read() end
function onClick() noctalia.runAsync("kavat-shell msg panel-open kavat/settings:panel remote") end

noctalia.setUpdateInterval(5000)
barWidget.setVisible(false)
read()
