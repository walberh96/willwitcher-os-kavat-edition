-- /f in the launcher: files by their name. Your own folder first, found as it is now; then
-- the whole disk, from the index. Enter opens the file with what opens its kind.
function onQuery(text)
  if #text < 2 then
    launcher.setResults(text, { { id = "", title = "Type two letters or more of a file's name", glyph = "file-search" } })
    return
  end
  launcher.setResults(text, { { id = "", title = "Looking…", glyph = "loader" } })
  -- the words are handed over as arguments, never as a line of shell
  noctalia.runAsync({ "sh", "-c", 'fd --max-results 12 --fixed-strings --ignore-case -- "$1" "$HOME" 2>/dev/null; plocate --limit 12 --ignore-case -- "$1" 2>/dev/null', "sh", text }, function(result)
    local results, seen = {}, {}
    for path in result.stdout:gmatch("[^\n]+") do
      if not seen[path] and #results < 20 then
        seen[path] = true
        local name = path:match("([^/]+)/?$") or path
        results[#results + 1] = { id = path, title = name, subtitle = path, glyph = "file", score = 100 - #results }
      end
    end
    if #results == 0 then
      results[1] = { id = "", title = "No file by that name", glyph = "file-off" }
    end
    launcher.setResults(text, results)
  end)
end

function onActivate(id)
  if id ~= "" then
    noctalia.runAsync({ "xdg-open", id })
  end
end
