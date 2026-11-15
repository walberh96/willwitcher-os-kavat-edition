-- Settings: how the editor looks and behaves before any plugin.
local o = vim.opt

o.number = true
o.cursorline = true
o.signcolumn = "yes"
o.termguicolors = true
o.showmode = false          -- the status bar says it
o.laststatus = 3            -- one status bar for all the windows
o.scrolloff = 4
o.wrap = false
o.splitright = true
o.splitbelow = true
o.mouse = "a"

o.expandtab = true
o.shiftwidth = 4
o.tabstop = 4
o.smartindent = true

o.ignorecase = true
o.smartcase = true

o.undofile = true
o.swapfile = false
o.confirm = true            -- ask instead of failing when something is not saved
o.updatetime = 250
o.timeoutlen = 400

-- Shift with the arrows selects, as in any editor, and typing replaces what is selected.
o.keymodel = "startsel,stopsel"
o.selectmode = "key"
o.whichwrap:append("<,>,[,]")

-- The clipboard is the desktop's only when asked for, with Ctrl+C, Ctrl+X and Ctrl+V:
-- Vim's own yanks and deletes stay out of it.
