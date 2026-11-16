-- Copy or cut, as Yazi does, and leave the files on the clipboard too, so that they can be
-- pasted into another program: a chat, a mail, another file manager. While they wait, the
-- status bar says how many are ready to paste.

-- the selection, or the file under the cursor, as Yazi itself takes them
local chosen = ya.sync(function()
	local paths = {}
	for _, file in pairs(cx.active.selected) do
		paths[#paths + 1] = tostring(file.url)
	end
	local hovered = cx.active.current.hovered
	if #paths == 0 and hovered then
		paths[1] = tostring(hovered.url)
	end
	return paths
end)

-- the line in the status bar while something waits to be pasted: how many items, and
-- whether they will be copied or moved. It is put there the first time something is
-- copied, and says nothing once they are pasted or let go
local reminder = ya.sync(function(self)
	if self.status then
		return
	end
	self.status = Status:children_add(function()
		local count = 0
		for _ in pairs(cx.yanked) do
			count = count + 1
		end
		if count == 0 then
			return ""
		end
		return ui.Line {
			ui.Span(count .. (count == 1 and " item" or " items") .. " ready to paste"):fg("blue"):bold(),
			ui.Span(cx.yanked.is_cut and " · as a move " or " · as a copy "):fg("darkgray"),
		}
	end, 300, Status.RIGHT)
end)

return {
	entry = function(_, job)
		local paths = chosen()
		reminder()
		ya.emit("yank", { cut = job.args[1] == "cut" })
		if #paths == 0 then
			return
		end
		-- a list of files is their addresses, one to a line, with what is not a plain
		-- letter written as its number
		local list = ""
		for _, path in ipairs(paths) do
			list = list .. "file://" .. path:gsub("[^%w/%-%._~]", function(c) return string.format("%%%02X", c:byte()) end) .. "\r\n"
		end
		local child = Command("wl-copy"):arg({ "--type", "text/uri-list" }):stdin(Command.PIPED):stdout(Command.NULL):stderr(Command.NULL):spawn()
		if child then
			child:write_all(list)
			child:flush()
			child:wait()
		end
	end,
}
