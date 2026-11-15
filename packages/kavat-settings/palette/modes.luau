-- /m in the launcher: the system's modes. Enter changes to one.
function onQuery(text)
  noctalia.runAsync({ "kavat-mode", "list" }, function(result)
    local results = {}
    for mark, word, name, about in result.stdout:gmatch("([* ]) ([^\t\n]+)\t([^\t\n]*)\t([^\n]*)") do
      local score = text == "" and 1 or noctalia.fuzzyScore(text, name .. " " .. about)
      if score then
        results[#results + 1] = { id = word, title = name, subtitle = (mark == "*" and "in force · " or "") .. about, glyph = "toggle-right", score = score }
      end
    end
    launcher.setResults(text, results)
  end)
end

function onActivate(id)
  noctalia.runAsync({ "kavat-mode", id })
end
