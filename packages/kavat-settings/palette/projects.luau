-- /pr in the launcher: your projects. Enter goes to the project's workspace and opens what
-- it is made of.
function onQuery(text)
  noctalia.runAsync({ "kavat-project" }, function(result)
    local results = {}
    for mark, name, folder in result.stdout:gmatch("([* ]) ([^\t\n]+)\t([^\t\n]*)\t[^\n]*") do
      local score = text == "" and 1 or noctalia.fuzzyScore(text, name .. " " .. folder)
      if score then
        results[#results + 1] = { id = name, title = name, subtitle = (mark == "*" and "open · " or "") .. folder, glyph = "folders", score = score }
      end
    end
    if #results == 0 then
      results[1] = { id = "", title = "No project yet", subtitle = "kavat-project new <name> in a terminal, inside its folder", glyph = "folder-plus" }
    end
    launcher.setResults(text, results)
  end)
end

function onActivate(id)
  if id ~= "" then
    noctalia.runAsync({ "kavat-project", "open", id })
  end
end
