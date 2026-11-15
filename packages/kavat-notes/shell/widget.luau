-- What is due in the notes, on the bar: how many tasks and reminders are due today or
-- overdue. With none, the widget is not there. A press opens the notes on their own
-- screen, as Super+Shift+N does. kavat-notes is asked for the number every minute.

local function show(count)
  if count > 0 then
    barWidget.setGlyph("checklist")
    barWidget.setText(tostring(count))
    barWidget.setTooltip(count == 1 and "1 note is due today or overdue" or count .. " notes are due today or overdue")
  end
  barWidget.setVisible(count > 0)
end

local function read()
  noctalia.runAsync({ "kavat-notes", "count" }, function(result)
    show(result.exitCode == 0 and tonumber(result.stdout:match("%d+")) or 0)
  end)
end

function update() read() end
function onIpc() read() end
function onClick()
  noctalia.runAsync({ "kitty", "--class", "kavat-notes", "-o", "remember_window_size=no", "kavat-notes" })
end

noctalia.setUpdateInterval(60000)
barWidget.setVisible(false)
read()
