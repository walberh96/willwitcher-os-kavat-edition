-- The network on the bar: whether the machine is connected, and through what. A press
-- opens the list of networks, to join one, as a press on Bluetooth opens its devices; a
-- press with the right button, the Network page of the settings. On the installer's medium it says it in words too, since what the
-- installer can add to the system depends on it: "Online", or "No network · offline install".
-- NetworkManager is asked every few seconds; nothing is asked of the network itself.

local function show(online, wifi, live)
  barWidget.setGlyph(not online and "wifi-off" or wifi and "wifi" or "world")
  barWidget.setText(live and (online and "Online" or "No network · offline install") or "")
  barWidget.setTooltip(online and (wifi and "Connected by Wi-Fi" or "Connected by cable") or "No network")
end

local function read()
  noctalia.runAsync("nmcli -t -f STATE general; nmcli -t -f TYPE connection show --active | grep -c -m 1 wireless; [ -d /run/archiso ] && echo live", function(result)
    local out = result.stdout or ""
    show(out:match("^connected\n") ~= nil or out:match("^connected %(site%)") ~= nil, out:match("\n1\n") ~= nil, out:match("live") ~= nil)
  end)
end

function update() read() end
function onIpc() read() end
function onClick() noctalia.runAsync("kavat-shell msg panel-toggle control-center network") end
function onRightClick() noctalia.runAsync("kavat-shell msg panel-open kavat/settings:panel network") end

noctalia.setUpdateInterval(5000)
read()
