-- The shortcuts: those of VS Code, in any mode. Vim's modes and its own keys stay.
--
-- A few of Vim's keys give way to them: Ctrl+V pastes, so Ctrl+Q starts a block selection;
-- Ctrl+W closes a tab, so the window commands are on <leader>w; and Ctrl+F, Ctrl+B and
-- Ctrl+D find, open the explorer and add a cursor, so the page moves with PageUp and
-- PageDown.
local map = vim.keymap.set

-- A shortcut for a selection. Shift with the arrows leaves Select mode, where a key typed
-- replaces the text: Ctrl+G goes over to Visual mode first, where it is a command.
local function on_selection(keys, what, desc)
    map("x", keys, what, { desc = desc, silent = true })
    map("s", keys, "<C-g>" .. what, { desc = desc, silent = true })
end

local function picker(name)
    return function()
        require("fzf-lua")[name]()
    end
end

-- Save; a file with no name is asked for one
local function save()
    if vim.api.nvim_buf_get_name(0) ~= "" then
        vim.cmd.update()
        return
    end
    vim.ui.input({ prompt = "Save as: ", completion = "file" }, function(name)
        if name and name ~= "" then
            vim.cmd.write(vim.fn.fnameescape(name))
        end
    end)
end

-- Close the tab: the file goes, the window and its place stay
local function close_tab()
    local buffer = vim.api.nvim_get_current_buf()
    if vim.bo[buffer].buftype ~= "" then
        vim.cmd.close()
        return
    end
    if vim.bo[buffer].modified then
        local choice = vim.fn.confirm("Save the changes to " .. vim.fn.expand("%:t") .. "?", "&Yes\n&No\n&Cancel")
        if choice == 1 then
            save()
        elseif choice ~= 2 then
            return
        end
    end
    local others = vim.tbl_filter(function(other)
        return other ~= buffer and vim.bo[other].buflisted
    end, vim.api.nvim_list_bufs())
    if #others > 0 then
        vim.cmd.bprevious()
    else
        vim.cmd.enew()
    end
    pcall(vim.api.nvim_buf_delete, buffer, { force = true })
end

local function go_to_line()
    vim.ui.input({ prompt = "Go to line: " }, function(text)
        local line = tonumber(text)
        if line then
            vim.api.nvim_win_set_cursor(0, { math.max(1, math.min(line, vim.api.nvim_buf_line_count(0))), 0 })
        end
    end)
end

local function format()
    require("conform").format({ async = true })
end

local function add_cursor()
    require("multicursor-nvim").matchAddCursor(1)
end

local everywhere = { "n", "i", "x", "s" }

-- Files and tabs
map(everywhere, "<C-s>", save, { desc = "Save" })
map(everywhere, "<C-w>", close_tab, { desc = "Close the tab", nowait = true })
map(everywhere, "<C-Tab>", "<Cmd>BufferLineCycleNext<CR>", { desc = "Next tab" })
map(everywhere, "<C-S-Tab>", "<Cmd>BufferLineCyclePrev<CR>", { desc = "Previous tab" })
map(everywhere, "<C-p>", picker("files"), { desc = "Find a file" })
map(everywhere, "<C-S-p>", picker("commands"), { desc = "Command palette" })
map(everywhere, "<C-S-f>", picker("live_grep"), { desc = "Find in the project" })
map(everywhere, "<C-b>", "<Cmd>NvimTreeToggle<CR>", { desc = "Explorer" })
map({ "n", "i", "x", "s", "t" }, "<C-`>", "<Cmd>ToggleTerm<CR>", { desc = "Terminal" })

-- Moving
map(everywhere, "<C-g>", go_to_line, { desc = "Go to a line" })
map({ "n", "i" }, "<M-Left>", "<Cmd>normal! <C-o><CR>", { desc = "Go back" })
map({ "n", "i" }, "<M-Right>", "<Cmd>execute \"normal! \\<C-i>\"<CR>", { desc = "Go forward" })

-- Selecting. Shift with the arrows is Vim's own, through 'keymodel'.
map("n", "<C-l>", "V", { desc = "Select the line" })
map("i", "<C-l>", "<Esc>V", { desc = "Select the line" })
on_selection("<C-l>", "j", "Select one line more")

-- Copy, cut and paste with the desktop's clipboard; undo and redo
map("n", "<C-c>", '"+yy', { desc = "Copy the line" })
map("i", "<C-c>", '<Cmd>normal! "+yy<CR>', { desc = "Copy the line" })
on_selection("<C-c>", '"+ygv', "Copy")
map("n", "<C-x>", '"+dd', { desc = "Cut the line" })
map("i", "<C-x>", '<Cmd>normal! "+dd<CR>', { desc = "Cut the line" })
on_selection("<C-x>", '"+d', "Cut")
map("n", "<C-v>", '"+p', { desc = "Paste" })
map("i", "<C-v>", "<C-r><C-o>+", { desc = "Paste" })
map("c", "<C-v>", "<C-r>+", { desc = "Paste" })
on_selection("<C-v>", '"+P', "Paste over the selection")
map("n", "<C-z>", "u", { desc = "Undo" })
map("i", "<C-z>", "<Cmd>undo<CR>", { desc = "Undo" })
map("n", "<C-y>", "<C-r>", { desc = "Redo" })
map("i", "<C-y>", "<Cmd>redo<CR>", { desc = "Redo" })
map("i", "<C-BS>", "<C-w>", { desc = "Delete the word before" })

-- Find and replace
map("n", "<C-f>", "/", { desc = "Find" })
map("i", "<C-f>", "<Esc>/", { desc = "Find" })
on_selection("<C-f>", '"zy/\\V<C-r>z', "Find the selection")
map("n", "<C-h>", ":%s//gc<Left><Left><Left>", { desc = "Replace" })
map("i", "<C-h>", "<Esc>:%s//gc<Left><Left><Left>", { desc = "Replace" })
on_selection("<C-h>", ":s//gc<Left><Left><Left>", "Replace in the selection")
map("n", "<Esc>", "<Cmd>nohlsearch<CR>", { desc = "Clear the search" })

-- Lines
for _, keys in ipairs({ "<C-/>", "<C-_>" }) do
    map("n", keys, "gcc", { desc = "Comment the line", remap = true })
    map("i", keys, "<Cmd>normal gcc<CR>", { desc = "Comment the line" })
    map("x", keys, "gcgv", { desc = "Comment the lines", remap = true })
    map("s", keys, "<C-g>gcgv", { desc = "Comment the lines", remap = true })
end
map({ "n", "i" }, "<M-Down>", "<Cmd>move .+1<CR><Cmd>normal! ==<CR>", { desc = "Move the line down" })
map({ "n", "i" }, "<M-Up>", "<Cmd>move .-2<CR><Cmd>normal! ==<CR>", { desc = "Move the line up" })
on_selection("<M-Down>", ":move '>+1<CR>gv=gv", "Move the lines down")
on_selection("<M-Up>", ":move '<-2<CR>gv=gv", "Move the lines up")
map({ "n", "i" }, "<M-S-Down>", "<Cmd>copy .<CR>", { desc = "Duplicate the line down" })
map({ "n", "i" }, "<M-S-Up>", "<Cmd>copy .-1<CR>", { desc = "Duplicate the line up" })
on_selection("<M-S-Down>", ":copy '><CR>gv", "Duplicate the lines down")
on_selection("<M-S-Up>", ":copy '<-1<CR>gv", "Duplicate the lines up")

-- More than one cursor
map({ "n", "x" }, "<C-d>", add_cursor, { desc = "A cursor at the next match" })
map("i", "<C-d>", function()
    vim.cmd.stopinsert()
    add_cursor()
end, { desc = "A cursor at the next match" })
map("s", "<C-d>", "<C-g><C-d>", { desc = "A cursor at the next match", remap = true })

-- The code
map({ "n", "i" }, "<F12>", vim.lsp.buf.definition, { desc = "Go to the definition" })
map({ "n", "i" }, "<F2>", vim.lsp.buf.rename, { desc = "Rename" })
map({ "n", "i", "x" }, "<C-.>", vim.lsp.buf.code_action, { desc = "Quick action" })
for _, keys in ipairs({ "<M-S-f>", "<M-F>" }) do
    map(everywhere, keys, format, { desc = "Format" })
end

-- What Ctrl+W and Ctrl+V used to be
map("n", "<leader>w", "<C-w>", { desc = "windows", remap = false })
map("n", "<C-q>", "<C-v>", { desc = "Select a block" })

-- The <leader> shortcuts, in normal mode
map("n", "<leader>ff", picker("files"), { desc = "Files" })
map("n", "<leader>fg", picker("live_grep"), { desc = "Text in the project" })
map("n", "<leader>fb", picker("buffers"), { desc = "Open files" })
map("n", "<leader>fr", picker("oldfiles"), { desc = "Recent files" })
map("n", "<leader>fh", picker("helptags"), { desc = "Help" })
map("n", "<leader>fk", picker("keymaps"), { desc = "Shortcuts" })
map("n", "<leader>e", "<Cmd>NvimTreeFindFile<CR>", { desc = "Show the file in the explorer" })
map("n", "<leader>gb", "<Cmd>Gitsigns blame_line<CR>", { desc = "Who wrote the line" })
map("n", "<leader>gp", "<Cmd>Gitsigns preview_hunk<CR>", { desc = "What changed here" })

-- In the terminal, Esc twice goes back to the editor's keys
map("t", "<Esc><Esc>", "<C-\\><C-n>", { desc = "Leave the terminal's keys" })
