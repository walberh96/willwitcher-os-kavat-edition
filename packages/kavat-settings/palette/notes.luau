-- /n in the launcher: your notes, by a word in them. With nothing typed, Enter takes a new
-- one; with words that find nothing, Enter saves them as a note.
function onQuery(text)
  if text == "" then
    launcher.setResults(text, { { id = "quick", title = "A quick note", subtitle = "or type words to find a note; if none has them, Enter saves them as one", glyph = "notes" } })
    return
  end
  noctalia.runAsync({ "sh", "-c", 'grep -r -i -l --include="*.md" -- "$1" "$HOME/Notes" 2>/dev/null | head -n 15', "sh", text }, function(result)
    local results = {}
    for path in result.stdout:gmatch("[^\n]+") do
      local name = path:match("([^/]+)%.md$") or path
      results[#results + 1] = { id = "open " .. path, title = name:gsub("%.todo$", ""), subtitle = path:gsub("^.*/Notes/", ""), glyph = "note", score = 100 - #results }
    end
    results[#results + 1] = { id = "add " .. text, title = "Save as a note: " .. text, subtitle = "a day and a time at its end set a reminder", glyph = "plus", score = 0 }
    launcher.setResults(text, results)
  end)
end

function onActivate(id)
  local what, rest = id:match("^(%a+) ?(.*)$")
  if what == "quick" then
    noctalia.runAsync("kavat-notes quick")
  elseif what == "open" then
    noctalia.runInTerminal({ "nvim", rest })
  elseif what == "add" then
    noctalia.runAsync({ "kavat-notes", "add", rest })
  end
end
