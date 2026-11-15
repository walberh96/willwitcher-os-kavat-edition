-- The colours follow the system's theme: kavat-theme writes them, and they are put on
-- again whenever it writes them anew.
local M = {}

local function folder()
    local state = vim.env.XDG_STATE_HOME
    if not state or state == "" then
        state = vim.env.HOME .. "/.local/state"
    end
    return state .. "/kavat/theme"
end

--- The theme's colours, or nothing when there is no theme to be had.
function M.palette()
    local found, palette = pcall(dofile, folder() .. "/current/nvim/palette.lua")
    if found and type(palette) == "table" then
        return palette
    end
end

function M.follow()
    vim.cmd.colorscheme("kavat")

    -- kavat-theme changes the theme by pointing "current" at a new folder
    local watch = vim.uv.new_fs_event()
    if not watch or vim.fn.isdirectory(folder()) == 0 then
        return
    end
    local waiting = false
    watch:start(folder(), {}, function()
        if waiting then
            return
        end
        waiting = true
        vim.defer_fn(function()
            waiting = false
            if vim.g.colors_name == "kavat" then
                vim.cmd.colorscheme("kavat")
            end
        end, 100)
    end)
end

return M
