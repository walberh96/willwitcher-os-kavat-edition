-- /k in the launcher: packages, from the lists pacman already has. Enter opens a terminal
-- with kav ready to install the one chosen, or to remove it when it is installed.
function onQuery(text)
  if #text < 2 then
    launcher.setResults(text, { { id = "", title = "Type two letters or more of a package's name", glyph = "package" } })
    return
  end
  launcher.setResults(text, { { id = "", title = "Looking…", glyph = "loader" } })
  noctalia.runAsync({ "sh", "-c", 'pacman -Ss -- "$1" 2>/dev/null | head -n 60', "sh", text }, function(result)
    local results, name, installed = {}, nil, false
    for line in result.stdout:gmatch("[^\n]+") do
      local repository, package, rest = line:match("^(%S+)/(%S+) (.*)$")
      if repository then
        name, installed = package, rest:find("%[installed") ~= nil
      elseif name and #results < 20 then
        local score = noctalia.fuzzyScore(text, name) or 0
        results[#results + 1] = { id = (installed and "remove " or "install ") .. name, title = name,
          subtitle = (installed and "installed · " or "") .. line:gsub("^%s+", ""), glyph = installed and "package-export" or "package-import", score = score }
        name = nil
      end
    end
    if #results == 0 then
      results[1] = { id = "", title = "No package by that name", subtitle = "the lists are as old as the last update", glyph = "package-off" }
    end
    launcher.setResults(text, results)
  end)
end

function onActivate(id)
  local what, name = id:match("^(%a+) ([%w@._+-]+)$")
  if what then
    noctalia.runInTerminal("kav " .. what .. " " .. name .. "; echo; read -r -n 1 -s -p 'Any key closes.'")
  end
end
