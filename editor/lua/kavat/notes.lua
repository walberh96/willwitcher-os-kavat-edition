-- Markdown, and with it the notes: what Enter does on a link or a checkbox, the switch
-- between formatted and plain text, and the reminders, set again when a note is saved.
local M = {}

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

function M.setup()
    local group = vim.api.nvim_create_augroup("kavat-notes", {})
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
