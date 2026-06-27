# Promem

Promem is Git-native project memory for AI-assisted development.

Git stores what changed. Promem stores why it was built this way.

Promem keeps durable project memory as Markdown inside the repository, under
`.promem/`, so important architecture notes, decisions, APIs, TODOs, and
handoff prompts can be reviewed, diffed, merged, and committed like code.

Each feature gets a canonical `.promem/features/<feature>/memory.md` file.
Promem renders that file in a round-trip-friendly Markdown schema and can parse
reasonable human edits back into structured memory. The sibling files such as
`context.md`, `decisions.md`, and `api.md` are browsable generated views.

Promem-owned content is wrapped in generated-region markers:

```md
<!-- promem:generated:start -->
...
<!-- promem:generated:end -->
```

Content outside those markers is preserved. For free-form feature notes, use
`.promem/features/<feature>/notes.md`; Promem creates it but does not overwrite
it.

## Local-First Promise

- Rust CLI first.
- No hosted backend required.
- No raw transcript storage.
- Markdown is the durable source of truth.
- Markdown should remain human-editable: Promem parses loose input and renders a
  canonical format.
- JSON stdin/stdout contracts make automation possible without coupling the
  core tool to a specific assistant.
- Generated caches and indexes stay out of Git.

## Current Phase

This repository starts with the docs-first foundation and a small single-crate
Rust MVP. The long-form product and architecture guide lives at
[`docs/vision.md`](docs/vision.md), the memory-engine addendum lives at
[`docs/vision-p2.md`](docs/vision-p2.md), assistant workflow guidance lives at
[`docs/assistant-usage.md`](docs/assistant-usage.md), and the versioned roadmap
starts at [`docs/roadmap/README.md`](docs/roadmap/README.md).

## Planned CLI

```bash
promem init
promem save-json <feature> [--dry-run]
promem save <feature> --from <file> [--dry-run]
promem import handoff <file> --feature <feature> [--dry-run]
promem load <feature>
promem list
promem tree
promem search <query>
promem open <feature>
promem doctor
promem snapshot <feature> --dry-run --json
```

## Quick Start

Install the latest released tag from GitHub:

```bash
cargo install --git https://github.com/bhagath-krishna/promem.git --tag v0.2.0
```

Or install from a local checkout:

```bash
cargo install --path .
```

Check the installed version:

```bash
promem --version
```

Initialize memory for a repository:

```bash
promem init
```

Save structured feature memory:

```bash
promem save-json authentication < examples/memory.json
```

Review the generated memory files:

```bash
promem tree
```

The canonical feature memory lives at:

```txt
.promem/features/authentication/memory.md
```

Promem also writes browsable generated views such as `context.md`,
`decisions.md`, and `todos.md`. Free-form human notes belong in:

```txt
.promem/features/authentication/notes.md
```

Save assistant-distilled handoff Markdown without writing JSON:

```bash
promem save product-direction --from examples/handoff.md
```

Preview a save without writing files:

```bash
promem save product-direction --from examples/handoff.md --dry-run
```

Load prompt-ready context:

```bash
promem load authentication
```

Inspect saved memory:

```bash
promem list
promem tree --json
promem search "refresh token"
promem open authentication
promem doctor --json
```

Preview repository state for a feature:

```bash
promem snapshot authentication --dry-run --json
```

## Development

Install Rust, then run:

```bash
cargo test
cargo run -- init
```

`cargo run -- <command>` is the from-source development form of `promem
<command>`.

## Versioning

Promem's CLI version comes from `Cargo.toml`:

```toml
version = "0.2.0"
```

For a release, update that version, tag the matching commit, and reinstall from
the checkout:

```bash
git tag v0.2.0
cargo install --path .
promem --version
```

Install or update from a release tag:

```bash
cargo install --git https://github.com/bhagath-krishna/promem.git --tag v0.2.0 --force
```

Homebrew and npm distribution are planned after GitHub release artifacts are
published. See [`docs/distribution.md`](docs/distribution.md).
