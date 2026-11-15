-- kavat-nvim: the system's Neovim. A user's init.lua calls setup() and goes on from there.
local M = {}

-- where the package keeps its plugins and its parsers
M.dir = vim.env.KAVAT_NVIM_DIR or "/usr/share/kavat/nvim"

--- Set Neovim up the system's way.
--- @param opts? { start_typing?: boolean, format_on_save?: boolean, relative_numbers?: boolean, wrap?: boolean, indent?: number }
function M.setup(opts)
    opts = opts or {}
    -- what the Editor page of the settings wrote, under what is said here in the call
    local file = io.open((vim.env.XDG_CONFIG_HOME or (vim.env.HOME .. "/.config")) .. "/kavat/editor.json")
    if file then
        local read, values = pcall(vim.json.decode, file:read("a"))
        file:close()
        if read and type(values) == "table" then
            opts = vim.tbl_extend("keep", opts, values)
        end
    end
    vim.g.mapleader = " "
    vim.g.maplocalleader = " "
    if opts.format_on_save ~= nil then
        vim.g.kavat_format_on_save = opts.format_on_save
    end

    require("kavat.options")
    if opts.relative_numbers ~= nil then
        vim.opt.relativenumber = opts.relative_numbers
    end
    if opts.wrap ~= nil then
        vim.opt.wrap = opts.wrap
    end
    if type(opts.indent) == "number" then
        vim.opt.shiftwidth = opts.indent
        vim.opt.tabstop = opts.indent
    end
    require("kavat.plugins").setup(M.dir)
    require("kavat.keys")
    require("kavat.notes").setup()
    require("kavat.theme").follow()

    -- The editor opens ready to type; Esc goes to normal mode.
    if opts.start_typing ~= false then
        vim.api.nvim_create_autocmd("VimEnter", {
            callback = function()
                if vim.bo.buftype == "" and vim.bo.modifiable and vim.fn.mode() == "n" then
                    vim.cmd.startinsert()
                end
            end,
        })
    end
end

return M
