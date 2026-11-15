-- Build the treesitter parsers kavat-nvim ships: nvim -l build-parsers.lua <plugin> <list> <out>
-- <plugin> is nvim-treesitter's folder, <list> a file of language names and <out> where
-- the parsers and their queries go. Each parser is fetched at the revision the plugin names.
local plugin, list, out = arg[1], arg[2], arg[3]
vim.opt.runtimepath:prepend(plugin)

local languages = {}
for name in io.open(list):read("a"):gmatch("%S+") do
    languages[#languages + 1] = name
end

local treesitter = require("nvim-treesitter")
treesitter.setup({ install_dir = out })
treesitter.install(languages):wait(1800000)

local missing = {}
for _, name in ipairs(languages) do
    if not vim.uv.fs_stat(out .. "/parser/" .. name .. ".so") then
        missing[#missing + 1] = name
    end
end
if #missing > 0 then
    io.stderr:write("not built: " .. table.concat(missing, " ") .. "\n")
    os.exit(1)
end
print(#languages .. " parsers built")
