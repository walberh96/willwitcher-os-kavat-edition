-- Paste with a window of its own: what is being copied and where to, how far each file
-- and the whole are, the speed and the time left. When a name is already taken it asks
-- whether to replace it, skip it or keep both, instead of answering by itself as Yazi does.
--
-- The copy is done here, one item after another, with cp and mv, and followed by looking
-- at what has arrived. A copy that is over at once never shows the window. Esc leaves it
-- working out of sight, and it says when it is done.

local KEYS = {
	{ on = "r", run = "replace" },
	{ on = "s", run = "skip" },
	{ on = "k", run = "both" },
	{ on = "a", run = "all" },
	{ on = "c", run = "cancel" },
	{ on = "<Esc>", run = "hide" },
}
local ROWS = 8 -- how many items are listed at once

-- what is waiting to be pasted, and the folder it would go to
local waiting = ya.sync(function()
	local files, regular = {}, cx.active.current.cwd.spec.is_regular
	for _, file in pairs(cx.yanked) do
		files[#files + 1] = { path = tostring(file.url), name = file.name }
		regular = regular and file.url.spec.is_regular
	end
	return files, tostring(cx.active.current.cwd), cx.yanked.is_cut, regular
end)

-- the window: put up or taken down
local window = ya.sync(function(self, on)
	if on and not self.children then
		self.children = Modal:children_add(self, 10)
	elseif not on and self.children then
		Modal:children_remove(self.children)
		self.children = nil
	end
	ui.render()
end)

-- what the window says: the whole of it, or one of its items
local say = ya.sync(function(self, changes)
	self.shown = self.shown or {}
	for name, value in pairs(changes) do
		self.shown[name] = value
	end
	ui.render()
end)
local say_item = ya.sync(function(self, at, changes)
	for name, value in pairs(changes) do
		self.shown.items[at][name] = value
	end
	ui.render()
end)

local function human(bytes)
	local units = { "B", "kB", "MB", "GB", "TB" }
	local at = 1
	while bytes >= 1000 and at < #units do
		bytes, at = bytes / 1000, at + 1
	end
	return string.format(at == 1 and "%.0f %s" or "%.1f %s", bytes, units[at])
end

local function exists(path) return fs.cha(Url(path)) end

-- how much there is under each path, in bytes; nothing for a path that is not there
local function sizes(paths)
	local output = Command("du"):arg({ "-sb", "--" }):arg(paths):stdout(Command.PIPED):stderr(Command.NULL):output()
	local found = {}
	for bytes, path in ((output and output.stdout) or ""):gmatch("(%d+)\t([^\n]*)") do
		found[path] = tonumber(bytes)
	end
	return found
end

-- a name nothing has yet: the file's, with a number before its ending
local function free_name(folder, name)
	local stem, ending = name:match("^(.+)(%.[^.]+)$")
	if not stem then
		stem, ending = name, ""
	end
	local number = 1
	while exists(folder .. "/" .. stem .. "_" .. number .. ending) do
		number = number + 1
	end
	return stem .. "_" .. number .. ending
end

-- Start a program, and say which process it is, so that it can be stopped: the shell
-- says its number and then becomes the program
local function run(program, arguments)
	local child, err = Command("sh")
		:arg({ "-c", 'echo $$; exec "$@"', "sh", program })
		:arg(arguments)
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()
	if not child then
		return nil, err
	end
	return child, nil, tonumber((child:read_line()))
end

local M = {}

function M:new(area)
	self:layout(area)
	return self
end

function M:layout(area) self._area = area end

function M:reflow() return { self } end

function M:redraw()
	local shown, area = self.shown, self._area
	if not shown or not area then
		return {}
	end
	local width = math.max(40, math.min(88, area.w - 4))
	local inner = width - 4
	local dim, accent = ui.Style():fg("darkgray"), ui.Style():fg("blue")

	local function bar(part, cells)
		local full = math.floor(math.max(0, math.min(1, part)) * cells + 0.5)
		return ui.Span(string.rep("━", full)):style(accent), ui.Span(string.rep("━", cells - full)):style(dim)
	end
	local name_cells = math.max(12, math.floor(inner * 0.36))
	local state_cells = 20
	local bar_cells = math.max(6, inner - name_cells - state_cells - 2)
	local function row(name, part, state, style, bold)
		local label = ui.truncate(name, { max = name_cells })
		local first = ui.Span(label .. string.rep(" ", name_cells - ui.Line(label):width() + 1))
		local filled, rest = bar(part, bar_cells)
		return ui.Line { bold and first:style(ui.Style():bold()) or first, filled, rest, ui.Span(" " .. state):style(style or ui.Style()) }
	end

	local finished = 0
	for _, item in ipairs(shown.items) do
		finished = finished + ((item.state == "done" or item.state == "skipped") and 1 or 0)
	end
	local lines = {
		ui.Line {
			ui.Span((shown.moving and "Moving " or "Copying ") .. #shown.items .. (#shown.items == 1 and " item" or " items")):style(ui.Style():bold()),
			ui.Span("  to " .. ui.truncate(shown.folder:gsub("^" .. (os.getenv("HOME") or "/nowhere"):gsub("%p", "%%%0"), "~"), { max = math.max(10, inner - 36), rtl = true })):style(dim),
			ui.Span("   " .. finished .. " of " .. #shown.items .. " done"):style(dim),
		},
		ui.Line(""),
	}
	-- the items around the one being copied, when they do not all fit
	local first = math.max(1, math.min((shown.at or 1) - 2, #shown.items - ROWS + 1))
	for at = first, math.min(#shown.items, first + ROWS - 1) do
		local item = shown.items[at]
		local style = (item.state == "done" and ui.Style():fg("green"))
			or (item.state == "waiting for you" and ui.Style():fg("yellow"))
			or (item.state == "failed" and ui.Style():fg("red"))
			or dim
		local state = item.state ~= "" and item.state or (item.done > 0 and math.floor(100 * item.done / math.max(1, item.size)) .. "%" or "")
		lines[#lines + 1] = row(item.name, item.size > 0 and item.done / item.size or (item.state == "done" and 1 or 0), state, style)
	end
	if #shown.items > ROWS then
		lines[#lines + 1] = ui.Line("… " .. (#shown.items - ROWS) .. " more"):style(dim)
	end
	lines[#lines + 1] = row("Total", shown.total > 0 and shown.done / shown.total or 0, human(shown.done) .. " of " .. human(shown.total), nil, true)
	if shown.speed and shown.speed > 0 and not shown.ask then
		local left = (shown.total - shown.done) / shown.speed
		local time = left >= 90 and math.floor(left / 60 + 0.5) .. " min left" or math.floor(left + 0.5) .. " s left"
		lines[#lines + 1] = ui.Line(human(shown.speed) .. "/s · " .. time):style(dim)
	else
		lines[#lines + 1] = ui.Line("")
	end
	lines[#lines + 1] = ui.Line("")
	if shown.ask then
		lines[#lines + 1] = ui.Line(ui.truncate(shown.ask, { max = inner })):style(ui.Style():fg("yellow"))
		lines[#lines + 1] = ui.Line {
			ui.Span(" r "):style(ui.Style():fg("black"):bg("blue")), ui.Span(" Replace   "),
			ui.Span(" s "):style(ui.Style():reverse()), ui.Span(" Skip   "),
			ui.Span(" k "):style(ui.Style():reverse()), ui.Span(" Keep both   "),
			ui.Span(" a "):style(ui.Style():reverse()), ui.Span(shown.all and " ☑ do this for all" or " ☐ do this for all"):style(shown.all and accent or dim),
		}
		lines[#lines + 1] = ui.Line("")
	end
	lines[#lines + 1] = ui.Line {
		ui.Span("Esc"):style(ui.Style():bold()), ui.Span(shown.ask and " waits for your answer   " or " run in background   "):style(dim),
		ui.Span("c"):style(ui.Style():bold()), ui.Span(" cancel"):style(dim),
	}

	local height = #lines + 2
	local rect = ui.Rect {
		x = area.x + math.floor((area.w - width) / 2),
		y = area.y + math.max(0, math.floor((area.h - height) / 2)),
		w = width,
		h = math.min(height, area.h),
	}
	return {
		ui.Clear(rect),
		ui.Border(ui.Edge.ALL):area(rect):type(ui.Border.ROUNDED):style(accent),
		ui.Text(lines):area(rect:pad(ui.Pad(1, 2, 1, 2))),
	}
end

function M:entry()
	local files, folder, moving, regular = waiting()
	if #files == 0 then
		return ya.notify { title = "Paste", content = "Nothing was copied or cut.", timeout = 3 }
	elseif not regular then
		-- a folder that is not on a disk of this machine: Yazi's own way knows how
		return ya.emit("paste", {})
	end

	local paths = {}
	for at, file in ipairs(files) do
		paths[at] = file.path
	end
	local size_of = sizes(paths)
	local items, total = {}, 0
	for at, file in ipairs(files) do
		file.size = size_of[file.path] or 0
		total = total + file.size
		items[at] = { name = file.name, size = file.size, done = 0, state = "" }
	end
	say { items = items, folder = folder, moving = moving, total = total, done = 0, speed = 0, ask = false, all = false, at = 1 }

	local visible, hidden, asking, finished, cancelled = false, false, false, false, false
	local all, for_all, process = false, nil, nil
	local wake, woken = ya.chan("mpsc")
	local answer, answered = ya.chan("mpsc")
	local started, done_before = ya.time(), 0

	local function show()
		if not visible then
			visible = true
			window(true)
			wake:send(true)
		end
	end

	-- the keys, while the window is up
	local function keys()
		while woken:recv() do
			while visible and not finished do
				local pressed = ya.which { cands = KEYS, silent = true }
				local key = pressed and KEYS[pressed].run
				if finished then
					return
				elseif key == "cancel" then
					cancelled = true
					if process then
						Command("kill"):arg({ tostring(process) }):status()
					end
					if asking then
						answer:send("skip")
					end
				elseif key == "all" then
					all = not all
					say { all = all }
				elseif asking and (key == "replace" or key == "skip" or key == "both") then
					answer:send(key)
				elseif key == "hide" and not asking then
					visible, hidden = false, true
					window(false)
				end
			end
			if finished then
				return
			end
		end
	end

	-- one item: where it goes, what to do when that name is taken, and the copy itself
	local function one(at, file)
		say { at = at }
		local target, how = folder .. "/" .. file.name, "new"
		local there = exists(target)
		if there then
			how = for_all
			if not how then
				local source = exists(file.path)
				asking = true
				say_item(at, { state = "waiting for you" })
				say {
					ask = file.name .. " already exists there (" .. human(there.len) .. (there.mtime and os.date(", from %d %b %H:%M", math.floor(there.mtime)) or "") .. ")."
						.. (source and target == file.path and " It is the same file." or ""),
				}
				show()
				how = answered:recv()
				asking = false
				say { ask = false }
				if all and not cancelled then
					for_all = how
				end
			end
			if how == "replace" and target == file.path then
				how = "skip" -- a file cannot replace itself
			end
			if how == "skip" then
				total = total - file.size
				say { total = total }
				return say_item(at, { state = cancelled and "cancelled" or "skipped", done = 0, size = 0 })
			elseif how == "both" then
				target = folder .. "/" .. free_name(folder, file.name)
				there = nil
			end
		end
		say_item(at, { state = "" })

		local source = exists(file.path)
		if not source then
			return say_item(at, { state = "failed" }), "it is not there any more"
		end
		-- what is in the way and is not a folder to be filled is taken away first
		if there and not (there.is_dir and source.is_dir) then
			local removing = run("rm", { "-rf", "--", target })
			if removing then
				removing:wait()
			end
			there = nil
		end
		local before = there and (sizes({ target })[target] or 0) or 0
		local merge = there ~= nil -- a folder over a folder: what it has stays, what comes is added
		local child, err
		if moving and not merge then
			child, err, process = run("mv", { "-T", "--", file.path, target })
		else
			child, err, process = run("cp", { "-a", "--reflink=auto", "-T", "--", file.path, target })
		end
		if not child then
			return say_item(at, { state = "failed" }), tostring(err)
		end

		local status, copying = nil, true
		ya.join(function()
			status = child:wait_with_output()
			copying = false
		end, function()
			while copying do
				ya.sleep(0.15)
				local arrived = exists(target)
				local bytes = 0
				if arrived and arrived.is_dir then
					bytes = (sizes({ target })[target] or 0) - before
				elseif arrived then
					bytes = arrived.len
				end
				bytes = math.max(0, math.min(file.size, bytes))
				local passed = ya.time() - started
				say_item(at, { done = bytes })
				say { done = done_before + bytes, speed = passed > 0.5 and (done_before + bytes) / passed or 0 }
				if passed > 0.4 and not hidden then
					show()
				end
			end
		end)
		process = nil
		if cancelled and not (status and status.status.success) then
			-- half a copy is of no use: it is taken away, unless it went into a folder that was there
			if not merge then
				local removing = run("rm", { "-rf", "--", target })
				if removing then
					removing:wait()
				end
			end
			return say_item(at, { state = "cancelled" })
		elseif not status or not status.status.success then
			return say_item(at, { state = "failed" }), (status and status.stderr or ""):gsub("%s+$", "")
		end
		if moving and merge then
			local removing = run("rm", { "-rf", "--", file.path })
			if removing then
				removing:wait()
			end
		end
		done_before = done_before + file.size
		say_item(at, { state = "done", done = file.size })
		say { done = done_before }
	end

	local function work()
		local failed, why, copied = 0, "", 0
		for at, file in ipairs(files) do
			if cancelled then
				say_item(at, { state = "cancelled" })
			else
				local _, err = one(at, file)
				if err then
					failed, why = failed + 1, err
				end
			end
		end
		finished = true
		if visible then
			ya.emit("which:dismiss", {})
		end
		wake:send(false)
		window(false)

		if moving and not cancelled and failed == 0 then
			ya.emit("unyank", {})
		end
		ya.emit("refresh", {})
		if failed > 0 then
			ya.notify { title = moving and "Move" or "Copy", content = failed .. (failed == 1 and " item" or " items") .. " could not be " .. (moving and "moved" or "copied") .. ": " .. why, timeout = 8, level = "error" }
		elseif cancelled then
			ya.notify { title = moving and "Move" or "Copy", content = "Cancelled. What was already done stays.", timeout = 4, level = "warn" }
		elseif hidden then
			ya.notify { title = moving and "Move" or "Copy", content = #files .. (#files == 1 and " item is" or " items are") .. " in " .. folder .. ".", timeout = 5 }
		end
	end

	ya.join(keys, work)
end

return M
