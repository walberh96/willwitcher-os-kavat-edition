# kavatd

The daemon between Kavat, the local agent, and the system. The agent never gets a shell:
it gets tools, each with a name, typed parameters, a permission level and one exact
command line. This is the skeleton phase 4 builds on. There is no model behind it yet.

## What is there

| Part | Where | What it does |
| --- | --- | --- |
| The tools | `tools/*.toml` | The system's first tools, one file to an area; installed in `/usr/share/kavat/agent/tools/` |
| The registry | `core/registry.rs` | Reads the tools, checks a call against its tool, builds the command line. The format is in its header |
| Running | `core/run.rs` | Runs a command with no shell and no input, a time limit, and a limit to the output it keeps |
| The gate and the log's line | `core/daemon.rs` | The requests, the confirmation in two steps, the undo, the refusal of what is privileged |
| The log | `core/log.rs` | `~/.local/state/kavat/agent/log.jsonl`, one line of JSON for each call |
| The socket | `core/serve.rs` | `$XDG_RUNTIME_DIR/kavatd.sock`, mode 600, a request and an answer to a line |
| The stubs | `core/model.rs` | Where the search model, the skills' phrases and the language model go |
| `kavatd` | `core/main.rs` | The daemon; `kavatd --check` reads the tools and lists them |
| `kavat-ask` | `core/ask.rs` | The terminal's side: `tools`, `call`, `log`, and free language for the model |

The first tools: `packages.search`, `packages.info`, `packages.list`, `packages.install`,
`packages.remove`, `theme.get`, `theme.set`, `windows.list`, `workspaces.switch`,
`mode.get`, `mode.set`, `wallpaper.get`, `wallpaper.set`, `volume.get`, `volume.set`,
`volume.mute`, `notes.add` and `notes.due`. Their names and parameters are those of
`agent/bench/tools.json`, the catalogue of the model's test, where both have the tool.

## The levels

| Level | What the daemon does |
| --- | --- |
| `read`, `reversible` | Runs at once. It stays in the log, with its undo when the tool declares one |
| `with-snapshot` | Does not run. Answers `confirm`, with the exact command and a token; runs when `confirm` comes back with the token and the same call. A token is for one call, works once and lasts a minute |
| `privileged` | Refused: the step that asks for the password through polkit is a stub |

A call marked `from_reading`, proposed because of something the agent read, waits for a
confirmation whatever its level. What a tool writes comes back in `output`, a field of
its own, and nothing reads it for something to do.

## What is a stub

Each is marked `STUB` in the code, with what comes next.

- **The language model**, `model::choose`: answers "the model is not built yet".
- **The search model**, `model::retrieve`: a match of plain words stands in for it.
- **The skills' exact phrases**, `model::skill_for`: there are no skills; nothing matches.
- **The polkit step**, `Daemon::privileged`: refuses every privileged call.
- **The snapshot** of a `with-snapshot` tool that is not kav's: today only kav's tools are
  at that level, and kav takes its own.
- **Who asked**: the client's name is what the client says it is.

## Trying it by hand

The system's tools run programs the build PC does not have, or should not be changed
with, so a folder with a tool that is harmless is used here:

```sh
mkdir -p /tmp/kavatd/run /tmp/kavatd/tools && chmod 700 /tmp/kavatd/run
cat > /tmp/kavatd/tools/try.toml <<'EOF'
[[tool]]
name = "try.say"
description = "Write a text"
level = "read"
command = ["/usr/bin/printf", "%s\n", "{text}"]
[tool.params.text]
type = "string"
description = "The text"

[[tool]]
name = "try.change"
description = "Stand in for a change of the system"
level = "with-snapshot"
command = ["/usr/bin/printf", "changed %s\n", "{name}"]
[tool.params.name]
type = "string"
form = "name"
description = "What to change"
EOF

# one terminal
XDG_RUNTIME_DIR=/tmp/kavatd/run XDG_STATE_HOME=/tmp/kavatd/state cargo run --bin kavatd -- --tools /tmp/kavatd/tools

# another
export XDG_RUNTIME_DIR=/tmp/kavatd/run
cargo run -q --bin kavat-ask -- tools
cargo run -q --bin kavat-ask -- call try.say text='hello; rm -rf $HOME'
cargo run -q --bin kavat-ask -- call try.change name=something    # asks before it runs
cargo run -q --bin kavat-ask -- log
cargo run -q --bin kavat-ask -- "make it dark"                    # the model is not built yet
```

`cargo run --bin kavatd -- --check --tools tools` reads the system's tools and lists
them. `cargo test` runs the tests; they use `printf`, `cp`, `cat` and `touch` in a
temporary folder and nothing of the system.

## Decisions the design leaves open

- **A Unix socket with a line of JSON, not D-Bus or MCP.** The design says "over D-Bus or
  MCP". The skeleton needs neither a bus nor a library for it, a test starts the daemon
  in a temporary folder, and the socket's mode is the whole of who may speak. The
  confirmation in two steps is the part neither gives ready-made. An MCP front is a small
  program on standard input and output that turns `tools/list` into `tools` and
  `tools/call` into `call`: the answer to `tools` already describes each tool's
  parameters as JSON Schema, which is what MCP, and Ollama, take. It must not carry
  `confirm`: a confirmation goes to the person, never back through the model.
- **Rust**, as `kav` and `kavat-theme`: `serde`, `serde_json` and `toml`, threads, and no
  async runtime. One `unsafe` line: `kill` for a whole group of processes, so that a
  command out of time is stopped with what it started.
- **Where things are.** Tools in `/usr/share/kavat/agent/tools/`; the log in
  `~/.local/state/kavat/agent/log.jsonl`; the socket in `$XDG_RUNTIME_DIR/kavatd.sock`.
- **Only the system's tools.** There is no folder of the person's for tools: a file
  written in the home folder would widen what the agent can run. What a person adds is a
  skill, which only chains tools that exist.
- **Names**: `area.verb`, as in the design's tables. The levels in a tool's file are
  written `read`, `reversible`, `with-snapshot` and `privileged`.
- **A session service**, `kavatd.service`, installed and not turned on.
- **Nothing runs that cannot be written to the log.**
- **The program of a tool is a whole path**, and a text is always an argument by itself
  that does not start with `-`: a call cannot choose what runs or pass for an option.

## What is known not to work yet

- `packages.install` and `packages.remove` reach `kav`, which asks for the password with
  `sudo` on a terminal and lets pacman ask its question there. The daemon has no terminal:
  until `kav` has a way to be asked from outside one, these two end with kav's error.
- `packages.list` has no `updates`, which the test's catalogue has: it is another command,
  `kav update --list`, and a tool is one command.
- `theme.set` takes `light` and `dark`; `auto`, in the test's catalogue, is another
  setting of `kavat-theme`, `auto.follow`.
- `volume.set`, `wallpaper.set`, `workspaces.switch` and `notes.add` declare no undo: what
  there was before is not one plain value a command prints.
