-- The side panels: programs beside the editor, the first one Claude.
--
-- A panel is not a window of Neovim's but one of Kitty's, next to the editor's, which is
-- opened and closed here by Kitty's remote control: a full terminal program has keys of
-- its own, which clash inside Neovim's terminal and do not in a window of Kitty's.
-- kavat-kitty turns the remote control on, with a password of the user's that only lets
-- windows be opened, focused, resized, closed and listed.
--
-- Claude's panel is connected to the editor by claudecode.nvim, which is handed this
-- module as the terminal it opens Claude in, and gives it the command and the variables
-- that lead Claude to this Neovim.
local M = {}

-- The panels: the program each one opens, on which side, and how much of the window it
-- takes, in percent. One more panel is one more line.
M.panels = {
    claude = { program = "claude", location = "vsplit", size = 40 },
}

-- the panels this Neovim has opened
local opened = {}

local function password()
    local file = io.open((vim.env.XDG_CONFIG_HOME or (vim.env.HOME .. "/.config")) .. "/kitty/rc-pass")
    if not file then
        return nil
    end
    local text = vim.trim(file:read("a"))
    file:close()
    return text ~= "" and text or nil
end

--- Whether the panels can be windows of Kitty's: the editor runs in a Kitty that takes
--- commands, and the password is there.
function M.in_kitty()
    return vim.env.KITTY_WINDOW_ID ~= nil
        and vim.env.KITTY_LISTEN_ON ~= nil
        and vim.fn.executable("kitten") == 1
        and password() ~= nil
end

-- One command for Kitty; says whether it was done, and what Kitty answered if not
local function kitty(...)
    local started, job = pcall(vim.system, { "kitten", "@", ... }, { env = { KITTY_RC_PASSWORD = password() }, text = true })
    if not started then
        return false, tostring(job)
    end
    local done = job:wait(3000)
    return done.code == 0, vim.trim(done.stderr or "")
end

-- What tells this editor's panel from the same panel of another Neovim
local function match(name)
    return ("var:kavat_panel=^%s-%d$"):format(name, vim.fn.getpid())
end

function M.is_open(name)
    return opened[name] ~= nil and (kitty("ls", "--match", match(name)))
end

--- Open a panel, or go to it when it is open.
--- @param name string one of M.panels
--- @param command? string[] what runs in it, when it is not the panel's own program
--- @param env? table<string, string> variables for the program
--- @param focus? boolean false leaves the keyboard in the editor
function M.open(name, command, env, focus)
    local panel = M.panels[name]
    command = command or { panel.program }
    if vim.fn.executable(command[1]) == 0 then
        vim.notify(command[1] .. " is not installed: the panel has nothing to open", vim.log.levels.WARN)
        return false
    end
    if M.is_open(name) then
        return focus == false or M.focus(name)
    end

    local editor = vim.env.KITTY_WINDOW_ID
    local args = {
        "launch", "--type=window", "--match=window_id:" .. editor, "--next-to=id:" .. editor,
        "--location=" .. panel.location, "--bias=" .. panel.size, "--cwd=" .. vim.fn.getcwd(),
        "--var=kavat_panel=" .. name .. "-" .. vim.fn.getpid(),
    }
    for key, value in pairs(env or {}) do
        args[#args + 1] = "--env=" .. key .. "=" .. value
    end
    if focus == false then
        args[#args + 1] = "--keep-focus"
    end
    args[#args + 1] = "--"
    vim.list_extend(args, command)

    local done, answer = kitty(unpack(args))
    if not done then
        vim.notify("The panel did not open: " .. answer, vim.log.levels.ERROR)
        return false
    end
    opened[name] = true
    return true
end

function M.focus(name)
    return opened[name] ~= nil and (kitty("focus-window", "--match", match(name)))
end

function M.close(name)
    if opened[name] then
        opened[name] = nil
        kitty("close-window", "--match", match(name))
    end
end

--- Go to the panel on one side of the editor, "vsplit" or "hsplit", when one is open.
function M.focus_side(location)
    for name in pairs(opened) do
        if M.panels[name].location == location and M.focus(name) then
            return true
        end
    end
    return false
end

--- Whether Claude Code is there to open. It is installed apart, so a key that needs it
--- says so in a line rather than open an empty panel.
function M.has_claude()
    if vim.fn.executable("claude") == 1 then
        return true
    end
    vim.notify("Claude Code is not installed: kav install claude-code", vim.log.levels.WARN)
    return false
end

--- Ctrl+Alt+B: open Claude's panel, or close it.
function M.toggle_claude()
    if M.has_claude() then
        vim.cmd.ClaudeCode()
    end
end

--- Ctrl+Alt+K: send Claude the selection, opening its panel if it is closed.
function M.send_selection()
    if M.has_claude() then
        vim.cmd.ClaudeCodeSend()
    end
end

--- Claude's panel, as claudecode.nvim asks for a terminal of its own.
function M.claude()
    local function open(command, env, _, focus)
        return M.open("claude", require("claudecode.utils").parse_command(command), env, focus)
    end
    local function toggle(command, env)
        if M.is_open("claude") then
            M.close("claude")
        else
            open(command, env)
        end
    end
    return {
        setup = function() end,
        open = open,
        close = function()
            M.close("claude")
        end,
        simple_toggle = toggle,
        focus_toggle = toggle,
        get_active_bufnr = function()
            return nil
        end,
        is_available = M.in_kitty,
    }
end

-- A panel does not outlive its editor
vim.api.nvim_create_autocmd("VimLeavePre", {
    callback = function()
        for name in pairs(opened) do
            M.close(name)
        end
    end,
})

return M
