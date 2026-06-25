# Promem Implementation Plan

This plan distills the full vision in `docs/vision.md` into build phases.

## Phase 1: Docs Foundation

- Preserve the full vision guide as `docs/vision.md`.
- Keep the README focused on positioning, local-first constraints, and how to
  run the project.
- Use this file as the implementation roadmap that future agents can follow
  without rereading the full guide every time.

Success: the repository explains what Promem is, why it exists, and how it will
be built.

## Phase 2: Rust MVP CLI

Build one Rust crate first. Do not split into a workspace until the core
behavior is stable.

Commands:

- `promem init`
- `promem save-json <feature>`
- `promem load <feature>`
- `promem list`
- `promem tree`
- `promem search <query>`
- `promem doctor`

Storage:

- Commit `.promem/project.md`.
- Commit `.promem/config.toml`.
- Commit `.promem/index.json`.
- Commit `.promem/shared/**/*.md`.
- Commit `.promem/features/**/*.md`.
- Do not commit `.promem/cache/` or generated indexes.

Success: a developer can save structured feature memory, commit it, and load it
later as prompt-ready context.

## Phase 3: Git-Native Workflows

Add `promem snapshot <feature> --dry-run`.

Collect:

- current branch
- `git status --short`
- `git diff --stat`
- `git diff --name-only`
- recent commits
- TODO/FIXME comments

Return JSON with `--json`. Warn about likely secrets before anything is saved.

Success: a developer can inspect a memory snapshot from current repository
changes without copy-paste.

## Phase 4: Automation

Add AI provider abstraction only after local storage and loading are reliable.

Add:

- provider trait
- OpenAI-compatible provider
- local provider option later
- structured extraction into the existing `MemoryInput` shape

Do not store raw handoff text or transcripts.

Success: Promem can turn notes or snapshots into reviewed memory files using a
configured provider.

## Phase 5: MCP And Search Intelligence

Add MCP after core functions exist and can be called without duplicating file
write logic.

Add:

- `promem mcp`
- load-context tool
- save-memory tool
- search tool
- snapshot tool
- chunking
- embeddings cache
- hybrid search

Success: MCP-capable assistants can read and write distilled project memory
through the same core engine used by the CLI.

## Defaults

- Repository: `/Users/bk/Documents/promem`
- Branch: `develop`
- GitHub repository: private `promem` under the authenticated user
- Canonical guide: `docs/vision.md`
- Initial Rust shape: single crate

