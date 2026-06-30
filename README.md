# Promemo

Promemo is Git-native project memory for AI-assisted development.

Git stores what changed. Promemo stores why it was built this way.

Promemo keeps durable project memory as Markdown inside the repository, under
`.promemo/`, so important architecture notes, decisions, APIs, TODOs, and
handoff prompts can be reviewed, diffed, merged, and committed like code.

Each feature gets a canonical `.promemo/features/<feature>/memory.md` file.
Promemo renders that file in a round-trip-friendly Markdown schema and can parse
reasonable human edits back into structured memory. The sibling files such as
`context.md`, `decisions.md`, and `api.md` are browsable generated views.

Promemo-owned content is wrapped in generated-region markers:

```md
<!-- promemo:generated:start -->
...
<!-- promemo:generated:end -->
```

Content outside those markers is preserved. For free-form feature notes, use
`.promemo/features/<feature>/notes.md`; Promemo creates it but does not overwrite
it.

## Local-First Promise

- Rust CLI first.
- No hosted backend required.
- No raw transcript storage.
- Markdown is the durable source of truth.
- Markdown should remain human-editable: Promemo parses loose input and renders a
  canonical format.
- JSON stdin/stdout contracts make automation possible without coupling the
  core tool to a specific assistant.
- Generated caches and indexes stay out of Git.

## Current Phase

This repository contains the Promemo CLI, MCP server, public usage guidance, and
distribution notes. Assistant workflow guidance lives at
[`docs/assistant-usage.md`](docs/assistant-usage.md), and package/release
details live at [`docs/distribution.md`](docs/distribution.md).

## Quick Start

Run with npm without a global install:

```bash
npx -y promemo@latest --version
npx -y promemo@latest init
```

Or install globally:

```bash
npm install -g promemo
promemo --version
```

Install the latest release binary directly:

```bash
curl -fsSL https://raw.githubusercontent.com/seguelabsai/promemo/main/scripts/install.sh | sh
```

Install the latest released tag from GitHub with Cargo:

```bash
cargo install --git https://github.com/seguelabsai/promemo.git --tag v0.3.9
```

Or install from a local checkout:

```bash
cargo install --path .
```

Check the installed version:

```bash
promemo --version
```

Initialize memory for a repository:

```bash
promemo init
```

Save structured feature memory:

```bash
promemo save-json authentication < examples/memory.json
```

Review the generated memory files:

```bash
promemo tree
```

The canonical feature memory lives at:

```txt
.promemo/features/authentication/memory.md
```

Promemo also writes browsable generated views such as `context.md`,
`decisions.md`, and `todos.md`. Free-form human notes belong in:

```txt
.promemo/features/authentication/notes.md
```

Save assistant-distilled handoff Markdown without writing JSON:

```bash
promemo save product-direction --from examples/handoff.md
```

Preview a save without writing files:

```bash
promemo save product-direction --from examples/handoff.md --dry-run
```

Extract structured memory from messy notes with a configured provider:

```bash
promemo extract authentication --from notes.md --dry-run
```

Provider extraction reads `.promemo/config.toml` and uses the configured API key
environment variable. It produces `MemoryInput` JSON internally, validates it,
then uses the same preview/save pipeline as `save-json`.

Load prompt-ready context:

```bash
promemo load authentication
```

Load compact context with related feature expansion:

```bash
promemo load authentication --related --token-budget 1200
```

Inspect saved memory:

```bash
promemo list
promemo tree --json
promemo search "refresh token"
promemo search "why did we choose passkeys" --hybrid --limit 5
promemo index rebuild
promemo index status
promemo open authentication
promemo doctor --json
```

Run the MCP stdio server for MCP-capable assistants:

```bash
promemo mcp
```

The MCP server exposes tools for loading token-budgeted context, keyword and
hybrid/semantic search, index status and rebuilds, related feature discovery,
project memory maps, previewing and saving structured memory, extracting memory
with the configured provider, listing features, reading feature memory files,
inspecting the memory tree, running doctor checks, and collecting repository
snapshots.
Provider extraction previews return both the exact generated `MemoryInput` and
the save report, so assistants can ask for approval and then save the reviewed
memory with `promemo_save_memory`.

### MCP Client Configuration

Promemo runs as one local stdio MCP server. You do not need one Promemo config
per model provider; you only need the JSON shape expected by your MCP host app.

For Cursor, Claude Desktop, Cline, Roo Code, Windsurf, Antigravity, and many
other MCP hosts that use the common `mcpServers` shape:

```json
{
  "mcpServers": {
    "promemo": {
      "command": "promemo",
      "args": ["mcp"]
    }
  }
}
```

For VS Code MCP user configuration:

```json
{
  "servers": {
    "promemo": {
      "type": "stdio",
      "command": "promemo",
      "args": ["mcp"]
    }
  }
}
```

If a host app cannot find `promemo` on `PATH`, replace `"promemo"` with the
absolute path from:

```bash
which promemo
```

See [`docs/assistant-usage.md`](docs/assistant-usage.md) for a fuller host
configuration guide.

Preview repository state for a feature:

```bash
promemo snapshot authentication --dry-run --json
```

Generate shell completions:

```bash
promemo completions zsh > _promemo
promemo completions bash > promemo.bash
```

Generate CLI usage documentation:

```bash
promemo docs > docs/cli.md
```

## Development

Install Rust, then run:

```bash
cargo test
cargo run -- init
```

`cargo run -- <command>` is the from-source development form of `promemo
<command>`.

## Versioning

Promemo's CLI version comes from `Cargo.toml`:

```toml
version = "0.3.9"
```

For a release, update that version, tag the matching commit, and reinstall from
the checkout:

```bash
git tag v0.3.9
cargo install --path .
promemo --version
```

Install or update from a release tag:

```bash
cargo install --git https://github.com/seguelabsai/promemo.git --tag v0.3.9 --force
```

npm and GitHub release binaries are available now. A Homebrew formula seed lives
in `packaging/homebrew/promemo.rb`. See
[`docs/distribution.md`](docs/distribution.md).
