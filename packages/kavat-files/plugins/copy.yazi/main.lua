-- Copy or cut, as Yazi does, and leave the files on the clipboard too, so that they can be
-- pasted into another program: a chat, a mail, another file manager.

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

return {
	entry = function(_, job)
		local paths = chosen()
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
