-- /t in the launcher: the tabs that are open in the browser, by their title or their
-- address. Enter brings the tab to the front, and its window with it.
function onQuery(text)
  noctalia.runAsync({ "kavat-browser", "tabs" }, function(result)
    local results = {}
    for id, title, address in result.stdout:gmatch("(%d+)\t([^\t\n]*)\t([^\n]*)") do
      local score = text == "" and 1 or noctalia.fuzzyScore(text, title .. " " .. address)
      if score then
        results[#results + 1] = { id = id, title = title, subtitle = address, glyph = "world", score = score }
      end
    end
    if #results == 0 then
      results[1] = { id = "", title = text == "" and "No tab open" or "No tab by that name", subtitle = "the browser's own, not the private ones", glyph = "world-off" }
    end
    launcher.setResults(text, results)
  end)
end

function onActivate(id)
  if id ~= "" then
    noctalia.runAsync({ "kavat-browser", "tab", id })
  end
end
