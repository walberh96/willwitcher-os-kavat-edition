-- Compress what is selected, or the file under the cursor, into an archive in this folder.
-- The kind of archive is the ending of the name that is typed: .zip, .7z, .tar.zst ...

local selected = ya.sync(function()
	local paths = {}
	for _, url in pairs(cx.active.selected) do
		paths[#paths + 1] = tostring(url)
	end
	if #paths == 0 and cx.active.current.hovered then
		paths[1] = tostring(cx.active.current.hovered.url)
	end
	return paths, tostring(cx.active.current.cwd)
end)

return {
	entry = function()
		local paths, folder = selected()
		if #paths == 0 then
			return
		end
		local name, event = ya.input({
			title = "Compress into:",
			value = "archive.zip",
			pos = { "top-center", y = 3, w = 50 },
		})
		if event ~= 1 or not name or name == "" then
			return
		end

		local arguments = { "compress", "--yes" }
		for _, path in ipairs(paths) do
			arguments[#arguments + 1] = path
		end
		arguments[#arguments + 1] = folder .. "/" .. name
		local output, problem = Command("ouch"):arg(arguments):stdout(Command.PIPED):stderr(Command.PIPED):output()
		if not output or not output.status.success then
			ya.notify({
				title = "Compress",
				content = "It could not be compressed: " .. (output and output.stderr or tostring(problem)),
				level = "error",
				timeout = 6,
			})
		else
			ya.notify({ title = "Compress", content = name .. " is ready", timeout = 3 })
		end
	end,
}
