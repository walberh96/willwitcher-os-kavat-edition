-- The plugins the system brings, and how each one is set.
--
-- lazy.nvim manages them, but none is downloaded: they come with the kavat-nvim package,
-- each at the commit it was tried with, so the editor is whole on a machine with no
-- network. Plugins of the user's own, in lua/plugins/, are lazy.nvim's to fetch as usual.
local M = {}

-- The language servers that are turned on when their program is installed
local servers = {
    marksman = "marksman",
    lua_ls = "lua-language-server",
    bashls = "bash-language-server",
    rust_analyzer = "rust-analyzer",
    clangd = "clangd",
    pyright = "pyright-langserver",
    ruff = "ruff",
    ts_ls = "typescript-language-server",
    gopls = "gopls",
    jsonls = "vscode-json-language-server",
    yamlls = "yaml-language-server",
    taplo = "taplo",
}

-- The formatters Shift+Alt+F and saving use, by kind of file; one that is not installed
-- is passed over
local formatters = {
    lua = { "stylua" },
    sh = { "shfmt" },
    bash = { "shfmt" },
    python = { "ruff_format" },
    rust = { "rustfmt" },
    go = { "gofmt" },
    javascript = { "prettier" },
    typescript = { "prettier" },
    json = { "prettier" },
    yaml = { "prettier" },
    css = { "prettier" },
    html = { "prettier" },
    markdown = { "prettier" },
    toml = { "taplo" },
}

local function specs(dir)
    local function plugin(name, spec)
        spec = spec or {}
        spec.dir = dir .. "/plugins/" .. name
        spec.name = name
        return spec
    end

    return {
        plugin("lazy.nvim"),
        plugin("nvim-web-devicons", { lazy = true }),

        -- The one picker: files, text, buffers, recent files, commands, help
        plugin("fzf-lua", {
            cmd = "FzfLua",
            opts = {
                fzf_colors = true,
                winopts = { height = 0.8, width = 0.8, preview = { layout = "flex" } },
                files = { fd_opts = "--color never --type file --type symlink --hidden --exclude .git --exclude .cache" },
                grep = { rg_opts = "--column --line-number --no-heading --color always --smart-case --max-columns 400 --hidden --glob !.git" },
            },
        }),

        -- The explorer, on the left
        plugin("nvim-tree.lua", {
            cmd = { "NvimTreeToggle", "NvimTreeFindFile", "NvimTreeFocus" },
            opts = {
                view = { width = 32 },
                renderer = { group_empty = true, root_folder_label = ":t" },
                update_focused_file = { enable = true },
                filters = { dotfiles = false, git_ignored = false },
                sync_root_with_cwd = true,
            },
        }),

        -- The open files as tabs, and the status bar
        plugin("bufferline.nvim", {
            event = "VeryLazy",
            opts = {
                options = {
                    diagnostics = "nvim_lsp",
                    always_show_bufferline = true,
                    offsets = { { filetype = "NvimTree", text = "Explorer", highlight = "NvimTreeRootFolder", separator = false } },
                },
            },
        }),
        plugin("lualine.nvim", {
            event = "VeryLazy",
            opts = {
                options = { theme = "auto", globalstatus = true, component_separators = "", section_separators = "" },
                sections = {
                    lualine_b = { "branch", "diff" },
                    lualine_c = { { "filename", path = 1 } },
                    lualine_x = { "diagnostics", "filetype" },
                },
            },
        }),

        -- Colours by what the code means. The parsers come with the package, built.
        plugin("nvim-treesitter", {
            lazy = false,
            config = function()
                vim.opt.runtimepath:prepend(dir .. "/treesitter")
                vim.api.nvim_create_autocmd("FileType", {
                    callback = function(event)
                        local language = vim.treesitter.language.get_lang(vim.bo[event.buf].filetype)
                        if language and pcall(vim.treesitter.start, event.buf, language) then
                            vim.bo[event.buf].indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
                        end
                    end,
                })
            end,
        }),

        -- Language servers: each is turned on when its program is on the system
        plugin("nvim-lspconfig", {
            lazy = false,
            config = function()
                for name, program in pairs(servers) do
                    if vim.fn.executable(program) == 1 then
                        vim.lsp.enable(name)
                    end
                end
                vim.diagnostic.config({ severity_sort = true, float = { border = "rounded" } })
            end,
        }),

        -- Completion as you type: Enter or Tab takes what is offered
        plugin("blink.cmp", {
            event = { "InsertEnter", "CmdlineEnter" },
            opts = {
                keymap = {
                    preset = "enter",
                    ["<Tab>"] = { "select_and_accept", "snippet_forward", "fallback" },
                    ["<S-Tab>"] = { "snippet_backward", "fallback" },
                },
                completion = { documentation = { auto_show = true, auto_show_delay_ms = 300 } },
                signature = { enabled = true },
                -- the matcher written in Lua: the other one is a program to download
                fuzzy = { implementation = "lua" },
            },
        }),

        -- Formatting, with Shift+Alt+F and on saving
        plugin("conform.nvim", {
            event = "BufWritePre",
            cmd = "ConformInfo",
            opts = {
                formatters_by_ft = formatters,
                default_format_opts = { lsp_format = "fallback" },
                format_on_save = function()
                    if vim.g.kavat_format_on_save == false then
                        return nil
                    end
                    return { timeout_ms = 1500 }
                end,
            },
        }),

        -- Markdown shown with its formatting: headings, checkboxes, tables and code. The line
        -- under the cursor stays as it is written, to edit it.
        plugin("render-markdown.nvim", {
            ft = "markdown",
            opts = {
                completions = { lsp = { enabled = true } },
                heading = { sign = false },
                code = { sign = false },
            },
        }),

        -- The pictures of a note, like the one a clipping keeps, drawn in the note itself. It
        -- draws with Kitty's graphics and ImageMagick's program, so it is only set up in a
        -- Kitty that has it; a picture on the web is not fetched.
        plugin("image.nvim", {
            ft = "markdown",
            config = function()
                if not vim.env.KITTY_WINDOW_ID or vim.fn.executable("magick") == 0 or #vim.api.nvim_list_uis() == 0 then
                    return
                end
                require("image").setup({
                    backend = "kitty",
                    processor = "magick_cli",
                    integrations = {
                        markdown = { enabled = true, download_remote_images = false, only_render_image_at_cursor = false },
                        asciidoc = { enabled = false },
                        typst = { enabled = false },
                        neorg = { enabled = false },
                        syslang = { enabled = false },
                    },
                    max_height_window_percentage = 40,
                    hijack_file_patterns = {},
                })
            end,
        }),

        -- The marketplace: plugins to look for, read about and install, on Ctrl+Shift+X. It
        -- asks the network for its list when it is opened, not before, and what it installs
        -- is a file of yours in lua/plugins/, which lazy.nvim fetches. Its count of what is
        -- looked at and installed, sent to its author's server, is turned off.
        plugin("store.nvim", {
            cmd = "Store",
            dependencies = { plugin("markview.nvim", { lazy = true, opts = {} }) },
            opts = { telemetry = false },
        }),

        -- Git: what changed, beside the line numbers
        plugin("gitsigns.nvim", {
            event = { "BufReadPre", "BufNewFile" },
            opts = { current_line_blame = false },
        }),

        -- The terminal, floating over the editor
        plugin("toggleterm.nvim", {
            cmd = "ToggleTerm",
            opts = {
                direction = "float",
                float_opts = { border = "rounded" },
                shade_terminals = false,
            },
        }),

        -- What the keys do, shown when one is held
        plugin("which-key.nvim", {
            event = "VeryLazy",
            opts = {
                preset = "modern",
                spec = {
                    { "<leader>f", group = "find" },
                    { "<leader>w", group = "windows" },
                    { "<leader>g", group = "git" },
                },
            },
        }),

        -- More than one cursor: Ctrl+D adds one at the next match
        plugin("multicursor.nvim", {
            event = "VeryLazy",
            config = function()
                local cursors = require("multicursor-nvim")
                cursors.setup()
                cursors.addKeymapLayer(function(set)
                    set({ "n", "x" }, "<Left>", cursors.prevCursor)
                    set({ "n", "x" }, "<Right>", cursors.nextCursor)
                    set("n", "<Esc>", function()
                        if not cursors.cursorsEnabled() then
                            cursors.enableCursors()
                        else
                            cursors.clearCursors()
                        end
                    end)
                end)
            end,
        }),
    }
end

function M.setup(dir)
    local lazy = dir .. "/plugins/lazy.nvim"
    if not vim.uv.fs_stat(lazy) then
        vim.notify("kavat-nvim: its plugins are not in " .. dir .. "; the editor goes on without them", vim.log.levels.WARN)
        return
    end
    vim.opt.runtimepath:prepend(lazy)

    local spec = specs(dir)
    -- the user's own plugins, when there are any
    if vim.uv.fs_stat(vim.fn.stdpath("config") .. "/lua/plugins") then
        spec[#spec + 1] = { import = "plugins" }
    end

    require("lazy").setup({
        spec = spec,
        -- nothing is looked for on the network unless the user asks
        checker = { enabled = false },
        change_detection = { notify = false },
        rocks = { enabled = false },
        install = { missing = true, colorscheme = { "kavat" } },
        -- the system's own folders stay where Neovim looks: the colours and this very
        -- module live there
        performance = { rtp = { reset = false } },
    })
end

return M
