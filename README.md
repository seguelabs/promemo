# Promem

Promem is Git-native project memory for AI-assisted development.

Git stores what changed. Promem stores why it was built this way.

Promem keeps durable project memory as Markdown inside the repository, under
`.promem/`, so important architecture notes, decisions, APIs, TODOs, and
handoff prompts can be reviewed, diffed, merged, and committed like code.

## Local-First Promise

- Rust CLI first.
- No hosted backend required.
- No raw transcript storage.
- Markdown is the durable source of truth.
- JSON stdin/stdout contracts make automation possible without coupling the
  core tool to a specific assistant.
- Generated caches and indexes stay out of Git.

## Current Phase

This repository starts with the docs-first foundation and a small single-crate
Rust MVP. The long-form product and architecture guide lives at
[`docs/vision.md`](docs/vision.md), and the versioned roadmap starts at
[`docs/roadmap/README.md`](docs/roadmap/README.md).

## Planned CLI

```bash
promem init
promem save-json <feature>
promem load <feature>
promem list
promem tree
promem search <query>
promem doctor
promem snapshot <feature> --dry-run --json
```

## Quick Start

Initialize memory for a repository:

```bash
cargo run -- init
```

Save structured feature memory:

```bash
cargo run -- save-json authentication < examples/memory.json
```

Load prompt-ready context:

```bash
cargo run -- load authentication
```

Inspect saved memory:

```bash
cargo run -- list
cargo run -- tree --json
cargo run -- search "refresh token"
cargo run -- doctor --json
```

Preview repository state for a feature:

```bash
cargo run -- snapshot authentication --dry-run --json
```

## Development

Install Rust, then run:

```bash
cargo test
cargo run -- init
```
