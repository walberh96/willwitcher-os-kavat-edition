-- Markdown, and with it the notes: what Enter does on a link or a checkbox, the switch
-- between formatted and plain text, the chips under a note's title, and the reminders, set
-- again when a note is saved.
local M = {}

local chips_of = vim.api.nvim_create_namespace("kavat-notes-chips")

local function notes_folder()
    return vim.env.NB_DIR or (vim.env.HOME .. "/Notes")
end

-- The link under the cursor: what is between [[ and ]], or between ( and ) after ]
local function link_here()
    local line = vim.api.nvim_get_current_line()
    local column = vim.api.nvim_win_get_cursor(0)[2] + 1
    for _, pattern in ipairs({ "%[%[(.-)%]%]", "%]%((.-)%)" }) do
        local from = 1
        while true do
            local first, last, target = line:find(pattern, from)
            if not first then
                break
            end
            if column >= first and column <= last then
                return target
            end
            from = last + 1
        end
    end
end

-- Open what a link points to: a folder in Yazi, a file in the editor, at its line when
-- the link names one, and a page in the browser
local function open(target)
    if target:match("^%a+://") then
        vim.ui.open(target)
        return
    end
    local path, line = target:match("^(.-):(%d+)$")
    path = vim.fn.expand(path or target)
    if not path:match("^/") then
        path = vim.fn.expand("%:p:h") .. "/" .. path
    end
    if vim.fn.isdirectory(path) == 1 then
        vim.fn.jobstart({ "kitty", "yazi", path }, { detach = true })
    elseif vim.fn.filereadable(path) == 1 then
        vim.cmd.edit(vim.fn.fnameescape(path))
        if line then
            vim.api.nvim_win_set_cursor(0, { math.min(tonumber(line), vim.api.nvim_buf_line_count(0)), 0 })
        end
    else
        -- a link to another note, by its name: the language server knows where it is
        vim.lsp.buf.definition()
    end
end

-- Check or uncheck the box of the line under the cursor. True when the line has one.
local function toggle_box()
    local line = vim.api.nvim_get_current_line()
    local changed, count = line:gsub("^(%s*[-*#]+ )%[ %]", "%1[x]", 1)
    if count == 0 then
        changed, count = line:gsub("^(%s*[-*#]+ )%[[xX]%]", "%1[ ]", 1)
    end
    if count == 1 then
        vim.api.nvim_set_current_line(changed)
    end
    return count == 1
end

function M.enter()
    local target = link_here()
    if target then
        open(target)
    elseif not toggle_box() then
        vim.cmd.normal({ args = { vim.keycode("<CR>") }, bang = true })
    end
end

-- What a note says about itself, read as the notes' screen reads it: the reminder, the
-- repetition and whether it is urgent, from the front matter; the day under "## Due"; the
-- tags, which are the words that start with # before the first section or under "## Tags";
-- and the line of its title.
function M.read(lines)
    local note = { tags = {}, title = nil }
    local at = 1
    if lines[1] == "---" then
        at = 2
        while lines[at] and lines[at] ~= "---" do
            local name, value = lines[at]:match("^(%a+): (.+)$")
            if name == "remind" or name == "repeat" or name == "urgent" then
                note[name] = vim.trim(value)
            end
            at = at + 1
        end
        at = at + 1
    end
    local section = ""
    for number = at, #lines do
        local line = lines[number]
        if line:match("^# ") then
            note.title = note.title or number
        elseif line:match("^## ") then
            section = vim.trim(line:sub(4)):lower()
        elseif section == "due" and not note.due and line:match("^%d%d%d%d%-%d%d%-%d%d") then
            note.due = line:sub(1, 10)
        elseif section == "" or section == "tags" then
            for tag in (" " .. line):gmatch("%f[%S]#[%w_-]+") do
                note.tags[#note.tags + 1] = tag
            end
        end
    end
    return note
end

-- A day as the chips say it: "Fri 13 Nov"
local function day(date)
    local year, month, number = date:match("^(%d+)-(%d+)-(%d+)")
    if not year then
        return date
    end
    local said = os.date("%a %d %b", os.time({ year = tonumber(year), month = tonumber(month), day = tonumber(number), hour = 12 }))
    return (said:gsub(" 0", " "))
end

-- Draw the chips of a note: one line under its title, or under its first line when it
-- has no title, with the day it is due, its reminder, its repetition and its tags. They
-- are drawn, not written: the file does not change.
function M.chips(buf)
    vim.api.nvim_buf_clear_namespace(buf, chips_of, 0, -1)
    local note = M.read(vim.api.nvim_buf_get_lines(buf, 0, 400, false))
    local now = os.date("%Y-%m-%d %H:%M")
    local chips = {}
    local function chip(text, group)
        chips[#chips + 1] = { " " .. text .. " ", group }
        chips[#chips + 1] = { " " }
    end
    if note.due then
        chip("Due " .. day(note.due), note.due < now:sub(1, 10) and "KavatChipLate" or "KavatChipDue")
    end
    if note.remind then
        local time = note.remind:match("%d+:%d+$")
        chip("⏰ " .. day(note.remind) .. (time and " " .. time or ""), note.remind <= now and "KavatChipLate" or "KavatChip")
    end
    if note["repeat"] then
        chip("↻ " .. note["repeat"], "KavatChip")
    end
    if note.urgent == "true" then
        chip("! urgent", "KavatChipLate")
    end
    for _, tag in ipairs(note.tags) do
        chip(tag, "KavatChipTag")
    end
    if #chips > 0 and vim.api.nvim_buf_line_count(buf) > 0 then
        vim.api.nvim_buf_set_extmark(buf, chips_of, (note.title or 1) - 1, 0, { virt_lines = { chips } })
    end
    return chips
end

function M.setup()
    local group = vim.api.nvim_create_augroup("kavat-notes", {})
    -- the chips' colours, for a colour scheme that does not name them
    local function colours()
        for name, link in pairs({ KavatChip = "CursorLine", KavatChipDue = "IncSearch", KavatChipLate = "DiagnosticError", KavatChipTag = "Directory" }) do
            vim.api.nvim_set_hl(0, name, { link = link, default = true })
        end
    end
    colours()
    vim.api.nvim_create_autocmd("ColorScheme", { group = group, callback = colours })
    -- a note opens with its chips, and they are drawn again when it is saved
    vim.api.nvim_create_autocmd({ "BufReadPost", "BufNewFile", "BufWritePost" }, {
        group = group,
        pattern = "*.md",
        callback = function(event)
            if vim.startswith(vim.api.nvim_buf_get_name(event.buf), notes_folder() .. "/") then
                M.chips(event.buf)
            end
        end,
    })
    vim.api.nvim_create_autocmd("FileType", {
        group = group,
        pattern = "markdown",
        callback = function(event)
            local map = function(mode, keys, what, desc)
                vim.keymap.set(mode, keys, what, { buffer = event.buf, desc = desc })
            end
            map("n", "<CR>", M.enter, "Follow the link, or check the box")
            map({ "n", "i" }, "<C-k>v", "<Cmd>RenderMarkdown buf_toggle<CR>", "Formatted or plain text")
            vim.opt_local.wrap = true
            vim.opt_local.linebreak = true
            vim.opt_local.conceallevel = 2
        end,
    })
    -- a note that is saved may have a new reminder, or have lost one
    vim.api.nvim_create_autocmd("BufWritePost", {
        group = group,
        pattern = "*.md",
        callback = function(event)
            local file = vim.api.nvim_buf_get_name(event.buf)
            if vim.startswith(file, notes_folder() .. "/") and vim.fn.executable("kavat-notes") == 1 then
                vim.fn.jobstart({ "kavat-notes", "sync" }, { detach = true })
            end
        end,
    })
end

return M
