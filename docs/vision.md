# Promem — Full Vision, Stack, Architecture, and Implementation Guide

**Project name:** Promem  
**Positioning:** Git-native project memory for AI-assisted development  
**Primary surface:** Local CLI  
**Core storage:** `.promem/` Markdown files committed with the repository  
**Primary language recommendation:** Rust-only for core product  
**Optional future layer:** TypeScript only for editor/web/plugin surfaces if the product later expands beyond local CLI/MCP

---

## 1. Executive Summary

Promem is a local developer tool that turns AI-assisted development sessions into durable, reusable project memory.

The key idea is simple:

> Git stores what changed. Promem stores why it was built this way.

A developer may spend 45 minutes with Claude, ChatGPT, Cursor, Codex, Gemini CLI, Aider, or another assistant designing authentication, debugging payments, choosing an architecture, or refactoring a feature. Today, that reasoning usually disappears inside a chat window. Promem turns the useful outcome of that session into structured Markdown files inside the repository:

```txt
.promem/
  project.md
  shared/
    coding-guidelines.md
    architecture.md
  features/
    authentication/
      context.md
      architecture.md
      decisions.md
      api.md
      todos.md
      changelog.md
```

Those files are committed to Git.

Three months later, another developer can run:

```bash
promem load authentication
```

and immediately give a new AI session the relevant project context.

Promem should **not** be positioned as a chat backup tool. It does not archive raw transcripts. It distills project knowledge.

Promem’s core promise:

> A developer should never need to explain the same feature twice.

---

## 2. Product Philosophy

### 2.1 Conversations should not be archived. They should be distilled.

Promem should not save giant chat logs. The source conversation is temporary. The durable artifact is extracted project knowledge.

Promem saves:

- summaries
- architecture notes
- implementation decisions
- tradeoffs
- APIs
- database notes
- referenced files
- TODOs
- open questions
- future work
- changelog entries

Promem should avoid saving:

- raw chat transcripts
- personal side comments
- model hallucinations without review
- accidental secrets
- noisy intermediate reasoning
- generated code dumps unless deliberately saved as documentation

### 2.2 The repository owns the memory

The memory should belong to the project, not to a specific AI provider.

Promem should work with:

- ChatGPT
- Claude
- Claude Code
- Cursor
- GitHub Copilot
- OpenAI Codex
- Gemini CLI
- Aider
- local LLMs
- future MCP-capable tools

The durable project memory should remain useful even if the user stops using Promem.

### 2.3 Markdown first

Everything important should be human-readable.

Promem should produce files that a developer can open, edit, diff, commit, and delete manually.

Preferred format:

```txt
Markdown + optional YAML/TOML frontmatter + JSON index metadata
```

Avoid locking core memory into a proprietary database.

### 2.4 Git native

Promem should use Git as the collaboration layer.

The workflow should feel like this:

```bash
promem save-json authentication < handoff.json
git diff .promem
git add .promem/features/authentication
git commit -m "Add authentication project memory"
git push
```

Another developer pulls the repository:

```bash
git pull
promem load authentication --copy
```

No hosted backend is required for the core product.

### 2.5 CLI is the source of truth

The CLI is the product.

Future surfaces like MCP, VS Code, Cursor, JetBrains, or a web UI should be clients of the same core engine. They should not create a second source of truth.

---

## 3. Final Stack Recommendation

For the current full vision — local CLI, `.promem/` folder, Git-native memory, MCP support, automatic handoff extraction, no web app, no hosted backend — the recommendation is:

> Build Promem in Rust only.

Do not start hybrid.

Do not start with TypeScript.

Use TypeScript only later if you add a web dashboard, editor extension, plugin ecosystem, hosted API, or complex JavaScript ecosystem integrations.

### 3.1 Rust stack

| Concern | Recommendation |
|---|---|
| Language | Rust |
| CLI parser | `clap` |
| Async runtime | `tokio` |
| HTTP client | `reqwest` |
| Serialization | `serde`, `serde_json` |
| Schema generation | `schemars` |
| Markdown parsing/rendering | `comrak` |
| Config | TOML via `serde` |
| Errors | `anyhow`, `thiserror` |
| Logging/tracing | `tracing`, `tracing-subscriber` |
| Git integration | shell out to `git` first |
| File walking | `walkdir` or `ignore` |
| Search v1 | local keyword search |
| Search v2 | SQLite + `rusqlite` |
| Vector search v2/v3 | `sqlite-vec` or replaceable local vector backend |
| Clipboard support | optional crate later |
| MCP server | Rust MCP implementation over stdio |
| Packaging | Cargo, Homebrew, GitHub Releases |

### 3.2 Why Rust

Rust fits Promem because the product is mostly:

- filesystem work
- Git integration
- Markdown parsing
- local indexing
- deterministic file updates
- prompt context building
- single-binary distribution
- CLI workflows
- local MCP server

Rust gives Promem a serious developer-tool feel.

The ideal user experience:

```bash
brew install promem
promem init
promem load authentication --copy
```

No Node runtime. No npm global install issues. No web account. No daemon. No backend.

### 3.3 Why not TypeScript first

TypeScript is excellent for:

- web apps
- dashboards
- hosted APIs
- VS Code extensions
- npm ecosystems
- provider SDKs
- MCP wrappers
- plugin systems

But the current Promem product is not primarily any of those.

Starting TypeScript-first would be useful if the actual product was:

```txt
Next.js dashboard
team SaaS
workspace accounts
hosted semantic search
VS Code/Cursor-first extension
plugin marketplace
```

That is not the current vision.

### 3.4 Why not hybrid from day one

Hybrid creates immediate complexity:

- Rust build system
- Node build system
- two release pipelines
- two schema systems
- JSON boundary bugs
- version mismatch bugs
- more installation complexity
- harder documentation
- harder tests

A hybrid stack is useful only when the product truly needs both.

For Promem’s current shape, build one excellent Rust binary first.

---

## 4. Core Architecture

Promem should be designed as a layered local engine.

```txt
┌─────────────────────────────────────────────┐
│                 Promem CLI                  │
│ init, save-json, load, search, snapshot     │
└───────────────────┬─────────────────────────┘
                    │
┌───────────────────▼─────────────────────────┐
│              Promem Core Engine             │
│ use cases, validation, orchestration         │
└───────┬────────────┬────────────┬───────────┘
        │            │            │
┌───────▼─────┐ ┌────▼──────┐ ┌───▼──────────┐
│ Store Layer │ │ Git Layer │ │ Search Layer │
│ .promem files   │ │ diff/log  │ │ keyword/vec  │
└───────┬─────┘ └────┬──────┘ └───┬──────────┘
        │            │            │
┌───────▼────────────▼────────────▼───────────┐
│             Local Repository                 │
│ src/, docs/, .promem/, git history               │
└─────────────────────────────────────────────┘

Optional:
┌─────────────────────────────────────────────┐
│              Promem MCP Server              │
│ exposes tools/resources/prompts over stdio   │
└─────────────────────────────────────────────┘
```

### 4.1 Core rule

Only Promem’s core engine should write `.promem/` memory files.

Other interfaces — MCP, future editor extensions, future wrappers — should call the core engine, not mutate files independently.

---

## 5. Repository Memory Structure

Recommended committed structure:

```txt
.promem/
  project.md
  config.toml
  index.json

  shared/
    architecture.md
    coding-guidelines.md
    conventions.md
    glossary.md

  features/
    authentication/
      context.md
      architecture.md
      decisions.md
      api.md
      data-model.md
      todos.md
      changelog.md
      prompts.md

    payments/
      context.md
      architecture.md
      decisions.md
      api.md
      data-model.md
      todos.md
      changelog.md
      prompts.md

  decisions/
    adr-0001-use-postgres.md
    adr-0002-use-jwt-access-tokens.md

  cache/
    embeddings.db
    search-index.sqlite
```

### 5.1 What should be committed

Commit:

```txt
.promem/project.md
.promem/config.toml
.promem/index.json
.promem/shared/**/*.md
.promem/features/**/*.md
.promem/decisions/**/*.md
```

### 5.2 What should not be committed

Do not commit generated cache files:

```txt
.promem/cache/
.promem/embeddings.db
.promem/search-index.sqlite
```

Recommended `.gitignore`:

```gitignore
.promem/cache/
.promem/embeddings.db
.promem/search-index.sqlite
```

### 5.3 Why cache should not be committed

Embeddings and search indexes are derived data.

They can be regenerated with:

```bash
promem sync
```

Binary indexes create noisy diffs and Git merge conflicts. Markdown should be the durable source of truth.

---

## 6. Feature Folder Contract

Each feature should have a predictable structure.

Example:

```txt
.promem/features/authentication/
  context.md
  architecture.md
  decisions.md
  api.md
  data-model.md
  todos.md
  changelog.md
  prompts.md
```

### 6.1 `context.md`

Purpose: high-level feature overview.

```md
---
feature: authentication
type: context
updated_at: 2026-06-26
---

# Authentication

## Summary

Authentication supports email login, refresh token rotation, and future MFA support.

## Current State

Implemented:
- Email login
- JWT access tokens
- Refresh token rotation

Pending:
- MFA
- Password reset
- Passkeys
```

### 6.2 `architecture.md`

Purpose: feature-level system design.

```md
# Authentication Architecture

## Components

- `AuthMiddleware` validates access tokens.
- `TokenService` signs and verifies JWTs.
- `RefreshTokenStore` persists refresh token state.

## Flow

1. User logs in.
2. API returns short-lived access token and refresh token.
3. Refresh token is rotated on renewal.
```

### 6.3 `decisions.md`

Purpose: capture why choices were made.

```md
# Authentication Decisions

## Accepted

### Use JWT access tokens

Reason:
- Keeps API stateless.
- Works well for horizontally scaled services.

Tradeoffs:
- Token revocation is harder.
- Requires careful refresh token design.

### Rotate refresh tokens

Reason:
- Reduces damage if a refresh token leaks.
```

### 6.4 `api.md`

Purpose: endpoint and contract memory.

```md
# Authentication API

## Endpoints

### POST /auth/login

Purpose:
Authenticate a user with email and password.

Returns:
- access token
- refresh token
- user profile
```

### 6.5 `data-model.md`

Purpose: database/schema decisions.

```md
# Authentication Data Model

## Tables

### users

Stores core user identity.

### refresh_tokens

Stores hashed refresh tokens and rotation metadata.
```

### 6.6 `todos.md`

Purpose: unresolved work.

```md
# Authentication TODOs

## High Priority

- Add MFA.
- Add password reset.

## Later

- Add passkeys.
- Add audit logging.
```

### 6.7 `changelog.md`

Purpose: append-friendly evolution log.

```md
# Authentication Changelog

## 2026-06-26

- Added JWT access token decision.
- Added refresh token rotation design.
```

### 6.8 `prompts.md`

Purpose: reusable prompts that worked well.

```md
# Authentication Prompts

## Useful Prompt

Explain the authentication architecture and identify security risks before modifying token refresh logic.
```

---

## 7. JSON Contract

Promem should support JSON input/output from the start.

This makes the tool easy to use from:

- MCP
- shell scripts
- AI assistants
- future editor extensions
- future TypeScript wrappers
- CI checks

### 7.1 `save-json` input

Command:

```bash
promem save-json authentication < memory.json
```

Input schema:

```json
{
  "title": "Authentication",
  "summary": "Authentication supports email login, JWT access tokens, refresh token rotation, and planned MFA.",
  "current_state": {
    "implemented": [
      "Email login",
      "JWT access tokens",
      "Refresh token rotation"
    ],
    "pending": [
      "MFA",
      "Password reset"
    ]
  },
  "architecture": [
    {
      "title": "Token validation",
      "details": "Auth middleware validates short-lived JWT access tokens on protected routes."
    }
  ],
  "decisions": [
    {
      "title": "Use JWT access tokens",
      "reason": "Keeps API stateless and easier to scale horizontally.",
      "tradeoffs": [
        "Harder token revocation",
        "Requires careful refresh token design"
      ],
      "status": "accepted"
    }
  ],
  "api": [
    {
      "method": "POST",
      "path": "/auth/login",
      "description": "Authenticates a user and returns access and refresh tokens."
    }
  ],
  "data_model": [
    {
      "name": "refresh_tokens",
      "description": "Stores hashed refresh tokens and rotation metadata."
    }
  ],
  "files": [
    {
      "path": "src/auth/middleware.rs",
      "reason": "Validates access tokens."
    }
  ],
  "todos": [
    {
      "text": "Add MFA",
      "priority": "high"
    }
  ],
  "open_questions": [
    "Should refresh tokens be device-scoped?"
  ],
  "future_work": [
    "Support passkeys."
  ],
  "prompts": [
    {
      "title": "Auth risk review",
      "prompt": "Review the auth system for security risks before changing token logic."
    }
  ]
}
```

### 7.2 Rust structs

Use Rust structs as the source of truth.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MemoryInput {
    pub title: String,
    pub summary: String,
    pub current_state: Option<CurrentState>,
    pub architecture: Vec<ArchitectureNote>,
    pub decisions: Vec<Decision>,
    pub api: Vec<ApiItem>,
    pub data_model: Vec<DataModelItem>,
    pub files: Vec<FileReference>,
    pub todos: Vec<TodoItem>,
    pub open_questions: Vec<String>,
    pub future_work: Vec<String>,
    pub prompts: Vec<PromptItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Decision {
    pub title: String,
    pub reason: Option<String>,
    pub tradeoffs: Vec<String>,
    pub status: DecisionStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Deprecated,
}
```

### 7.3 JSON output mode

Every important command should support `--json`.

Examples:

```bash
promem list --json
promem load authentication --json
promem search "refresh token" --json
promem snapshot authentication --json
```

This future-proofs integrations.

---

## 8. CLI Commands

### 8.1 Core commands

```bash
promem init
promem save-json <feature>
promem save <feature> --from <file>
promem save <feature> --stdin
promem load <feature>
promem list
promem tree
promem search <query>
promem sync
promem snapshot <feature>
promem doctor
```

### 8.2 `promem init`

Creates:

```txt
.promem/
  project.md
  config.toml
  index.json
  features/
  shared/
```

### 8.3 `promem save-json <feature>`

Reads structured JSON from stdin and writes Markdown files.

```bash
cat memory.json | promem save-json authentication
```

### 8.4 `promem save <feature> --from <file>`

Reads a handoff text or Markdown file.

```bash
promem save authentication --from handoff.md
```

This may either:

1. save directly if the file is already structured, or
2. call the configured AI provider to extract structured memory.

### 8.5 `promem save <feature> --stdin`

Useful with pipes:

```bash
pbpaste | promem save authentication --stdin
```

### 8.6 `promem load <feature>`

Builds prompt-ready context.

```bash
promem load authentication
```

Recommended flags:

```bash
promem load authentication --copy
promem load authentication --json
promem load authentication --max-tokens 8000
promem load authentication --include-related
promem load authentication --out context.md
```

### 8.7 `promem search <query>`

V1: keyword search.

```bash
promem search "refresh token"
```

V2: semantic search after `promem sync`.

```bash
promem search "why did we choose stateless auth"
```

### 8.8 `promem snapshot <feature>`

Collects current repository context:

- branch
- git status
- diff stat
- changed files
- recent commits
- TODO comments
- existing feature memory
- optional issue/PR note

Then produces or updates feature memory.

```bash
promem snapshot authentication
```

### 8.9 `promem mcp`

Starts local MCP server over stdio.

```bash
promem mcp --repo .
```

This allows MCP-capable assistants to load and save memory automatically.

---

## 9. Configuration

`.promem/config.toml`

```toml
[project]
name = "my-project"
default_branch = "main"

[storage]
root = ".promem"
commit_cache = false

[ai]
mode = "external" # external | api | local
provider = "openai_compatible"
model = "gpt-4.1-mini"
api_key_env = "OPENAI_API_KEY"

[embeddings]
enabled = false
mode = "local" # local | api
provider = "ollama"
model = "nomic-embed-text"

[search]
mode = "keyword" # keyword | semantic | hybrid
cache_dir = ".promem/cache"

[privacy]
store_raw_transcripts = false
redact_secrets = true

[git]
auto_add = false
auto_commit = false
```

### 9.1 AI modes

#### `external`

The AI chat produces structured memory and sends it to Promem.

Promem does not call an AI API.

Best for MCP.

```toml
[ai]
mode = "external"
```

#### `api`

Promem calls a cloud AI provider using the user’s own API key.

```toml
[ai]
mode = "api"
provider = "openai_compatible"
api_key_env = "OPENAI_API_KEY"
```

#### `local`

Promem calls a local model runtime like Ollama or LM Studio.

```toml
[ai]
mode = "local"
provider = "ollama"
model = "llama3.1"
```

---

## 10. Cost Model

Promem itself can be nearly free to use.

### 10.1 Free parts

- Rust CLI
- local `.promem/` files
- Markdown storage
- Git commits
- keyword search
- local MCP server
- local cache/index
- local LLMs, aside from user machine/electricity

### 10.2 Paid parts

Costs appear only when cloud AI APIs are used.

Possible paid operations:

- summarizing a handoff using OpenAI/Claude/Gemini
- generating embeddings through cloud embedding APIs
- hosted services if Promem later becomes SaaS
- code signing or distribution polish

### 10.3 Recommended default

Default Promem should not require the creator’s API key.

Use:

```txt
MCP assistant summarizes → Promem receives structured memory → Promem writes Markdown
```

In that model, the AI cost belongs to the user’s existing AI tool/session.

Promem should also support BYOK:

```txt
Bring your own OpenAI/Anthropic/Gemini key
```

Do not pay for users’ model usage unless building a paid hosted service.

---

## 11. MCP Vision

Promem should support MCP without requiring a web app.

MCP lets an AI assistant call Promem as a local tool.

```txt
AI assistant
  ↓ MCP tool call
promem mcp
  ↓
Promem core engine
  ↓
.promem/ Markdown files
```

### 11.1 MCP transport

Use stdio for local mode.

Example MCP config:

```json
{
  "mcpServers": {
    "promem": {
      "command": "promem",
      "args": ["mcp", "--repo", "."]
    }
  }
}
```

### 11.2 MCP resources

Expose project memory as resources:

```txt
promem://project
promem://features
promem://feature/authentication/context
promem://feature/authentication/decisions
promem://shared/coding-guidelines
```

### 11.3 MCP tools

Expose:

```txt
promem_load_context
promem_search
promem_save_memory
promem_snapshot
promem_list_features
promem_get_decisions
promem_get_project_tree
```

### 11.4 MCP prompts

Expose reusable prompts:

```txt
promem_handoff
promem_before_coding
promem_after_coding
promem_decision_capture
promem_architecture_review
```

Example prompt behavior:

> At the end of this session, extract durable project memory: summary, decisions, tradeoffs, files changed, APIs, TODOs, open questions. Save only the distilled memory using `promem_save_memory`. Do not save the raw transcript.

---

## 12. Automatic Extraction Strategy

Automatic extraction should not mean secretly scraping every chat app.

Promem should support realistic automation layers.

### 12.1 Level 1: MCP-driven handoff

Best workflow.

User says:

```txt
Save this session to Promem under authentication.
```

The assistant calls:

```txt
promem_save_memory
```

Promem receives structured JSON and writes Markdown.

No copy-paste.

No raw transcript storage.

### 12.2 Level 2: Snapshot-driven extraction

Useful even without chat access.

```bash
promem snapshot authentication
```

Promem collects:

```txt
git branch
git diff
changed files
recent commits
TODO comments
existing .promem memory
```

Then extracts feature memory.

### 12.3 Level 3: CLI wrapper

For terminal-based AI tools:

```bash
promem record authentication -- claude
promem record authentication -- aider
promem record authentication -- codex
```

The wrapper captures session text temporarily, extracts memory at the end, discards raw capture, and writes `.promem/`.

This should require explicit consent.

### 12.4 Level 4: Importers

Support exported handoff files later:

```bash
promem import authentication --from cursor-export.json
promem import authentication --from claude-export.md
```

### 12.5 Honest limitation

Promem cannot automatically capture every AI chat everywhere unless one of these is true:

- the AI client supports MCP
- the AI client exposes exports
- the AI client stores local session files
- the user runs the AI through a wrapper
- a future editor/browser extension is built

Promem should be honest about this.

---

## 13. Context Loading

The `load` command is the second most important command after `save`.

Goal:

```bash
promem load authentication
```

Output a compact, prompt-ready context that includes:

- project overview
- shared conventions
- feature summary
- architecture
- accepted decisions
- API notes
- data model
- important files
- TODOs
- open questions
- related features if enabled

### 13.1 Load algorithm

Basic algorithm:

```txt
1. Read .promem/project.md
2. Read .promem/shared/*.md
3. Read .promem/features/<feature>/*.md
4. Optionally search related features
5. Rank sections by relevance
6. Remove duplicate sections
7. Compress if max token budget is set
8. Output Markdown or JSON
```

### 13.2 Example output

```md
# Promem Context: Authentication

## Project Overview

...

## Shared Coding Guidelines

...

## Feature Summary

...

## Accepted Decisions

...

## APIs

...

## TODOs

...
```

### 13.3 Important flags

```bash
promem load auth --copy
promem load auth --max-tokens 8000
promem load auth --json
promem load auth --include-related
```

`--copy` is critical for non-MCP workflows.

---

## 14. Search and Indexing

### 14.1 V1 search

Keyword search across Markdown:

```bash
promem search "refresh token"
```

This should be simple and reliable.

Search output:

```txt
.promem/features/authentication/decisions.md
  Refresh token rotation was chosen to reduce risk if a token leaks.
```

### 14.2 V2 semantic search

Semantic search should be local and cache-based.

Flow:

```txt
scan .promem/**/*.md
split by headings
hash chunks
embed changed chunks
store embeddings in .promem/cache/
query by vector similarity
return relevant snippets
```

### 14.3 Hybrid search

Eventually use both:

```txt
keyword score + semantic score + recency + feature relevance
```

### 14.4 Do not overbuild early

Semantic search is powerful but not needed for the first useful version.

Promem should work well with simple Markdown loading and keyword search before adding embeddings.

---

## 15. Git Integration

Promem should respect Git instead of replacing it.

### 15.1 Basic Git behavior

Promem writes files.

Developer reviews:

```bash
git diff .promem
```

Developer commits:

```bash
git add .promem
git commit -m "Update authentication memory"
```

### 15.2 Optional helper flags

```bash
promem save-json authentication --git-add
promem snapshot authentication --git-add
```

Avoid auto-commit by default.

### 15.3 Future auto commit

Optional config:

```toml
[git]
auto_add = true
auto_commit = false
```

Auto-commit should be opt-in.

### 15.4 Merge-friendly design

Promem should:

- avoid rewriting every file
- use stable section ordering
- prefer append-friendly changelog entries
- keep files small
- avoid giant generated JSON blobs
- keep cache out of Git

---

## 16. Security and Privacy

Promem should be local-first and privacy-conscious.

### 16.1 No raw transcript storage

Default:

```toml
[privacy]
store_raw_transcripts = false
```

### 16.2 Secret redaction

Before writing memory, Promem should detect common secrets:

- API keys
- tokens
- passwords
- private keys
- connection strings
- `.env` values

If found, Promem should warn or redact.

### 16.3 User review

Promem should encourage:

```bash
git diff .promem
```

before commit.

### 16.4 Cloud API use

If Promem itself calls AI APIs, it should use the user’s key.

Never ship with the developer’s shared API key.

### 16.5 Sensitive files

Snapshot should avoid reading sensitive files by default:

```txt
.env
.env.local
secrets.*
private keys
node_modules
target
dist
build
```

---

## 17. Rust Crate Layout

Recommended workspace:

```txt
promem/
  Cargo.toml
  README.md
  crates/
    promem-cli/
      src/main.rs

    promem-core/
      src/
        lib.rs
        commands.rs
        models.rs
        config.rs
        errors.rs

    promem-store/
      src/
        lib.rs
        fs.rs
        markdown.rs
        frontmatter.rs
        index_json.rs

    promem-render/
      src/
        lib.rs
        context.rs
        architecture.rs
        decisions.rs
        api.rs
        todos.rs
        changelog.rs

    promem-search/
      src/
        lib.rs
        keyword.rs
        chunking.rs
        sqlite.rs
        semantic.rs

    promem-git/
      src/
        lib.rs
        status.rs
        diff.rs
        commits.rs
        snapshot.rs

    promem-ai/
      src/
        lib.rs
        provider.rs
        openai_compatible.rs
        ollama.rs

    promem-mcp/
      src/
        lib.rs
        server.rs
        tools.rs
        resources.rs
        prompts.rs
```

### 17.1 Simpler early layout

If starting smaller, use one crate with modules:

```txt
src/
  main.rs
  cli.rs
  models.rs
  config.rs
  store.rs
  render.rs
  index.rs
  search.rs
  git.rs
  snapshot.rs
  mcp.rs
```

Start simple. Split crates when the codebase grows.

---

## 18. Internal Module Responsibilities

### 18.1 `cli`

Parses commands and flags.

Should not contain business logic.

### 18.2 `models`

Contains Rust structs for:

- memory input
- decisions
- API items
- feature metadata
- search result
- load result
- snapshot result

### 18.3 `store`

Handles `.promem/` folder operations:

- create directories
- read/write Markdown
- update index
- validate paths
- avoid path traversal

### 18.4 `render`

Turns structured memory into Markdown.

### 18.5 `search`

Handles keyword/semantic search.

### 18.6 `git`

Collects Git state.

Use shell commands first:

```bash
git status --short
git diff --stat
git diff --name-only
git log -5 --oneline
```

### 18.7 `ai`

Optional provider calls.

Support:

- OpenAI-compatible APIs
- Anthropic later
- Gemini later
- Ollama/local later

### 18.8 `mcp`

Exposes Promem operations to AI clients.

---

## 19. AI Provider Strategy

Promem should be provider-agnostic.

### 19.1 Start with external mode

First, Promem can receive already-structured memory from MCP or `save-json`.

This requires no AI provider integration.

### 19.2 Add OpenAI-compatible API next

Many services support OpenAI-compatible endpoints.

Config:

```toml
[ai]
mode = "api"
provider = "openai_compatible"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1-mini"
api_key_env = "OPENAI_API_KEY"
```

### 19.3 Add local providers

Support local runtimes:

```toml
[ai]
mode = "local"
provider = "ollama"
model = "llama3.1"
```

### 19.4 Provider trait

```rust
#[async_trait::async_trait]
pub trait AiProvider {
    async fn extract_memory(&self, input: ExtractionRequest) -> anyhow::Result<MemoryInput>;
    async fn compress_context(&self, input: CompressionRequest) -> anyhow::Result<String>;
    async fn embed(&self, input: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>>;
}
```

---

## 20. Snapshot Design

Snapshot is the most powerful non-MCP automation.

Command:

```bash
promem snapshot authentication
```

### 20.1 Snapshot input

Collect:

```txt
current branch
git status
diff stat
changed file names
recent commits
TODO/FIXME comments
existing .promem feature memory
optional user note
optional issue/PR reference
```

### 20.2 Avoid huge diffs by default

Do not dump all code into the model blindly.

Use limits:

```toml
[snapshot]
max_diff_lines = 500
max_file_bytes = 20000
include_untracked = false
```

### 20.3 Snapshot output

Update:

```txt
context.md
architecture.md
decisions.md
api.md
data-model.md
todos.md
changelog.md
```

### 20.4 Snapshot prompt goal

The model should answer:

- What changed?
- Why did it change?
- What decisions were made?
- What files are important?
- What future work remains?
- What should future AI sessions know?

---

## 21. MCP Tool Contracts

### 21.1 `promem_load_context`

Input:

```json
{
  "feature": "authentication",
  "max_tokens": 8000,
  "include_related": true
}
```

Output:

```json
{
  "feature": "authentication",
  "context_markdown": "...",
  "included_files": [
    ".promem/project.md",
    ".promem/features/authentication/context.md",
    ".promem/features/authentication/decisions.md"
  ]
}
```

### 21.2 `promem_search`

Input:

```json
{
  "query": "refresh token rotation",
  "limit": 10
}
```

Output:

```json
{
  "results": [
    {
      "path": ".promem/features/authentication/decisions.md",
      "heading": "Rotate refresh tokens",
      "snippet": "Refresh token rotation reduces risk if a token leaks.",
      "score": 0.91
    }
  ]
}
```

### 21.3 `promem_save_memory`

Input:

Same as `save-json` memory schema.

Output:

```json
{
  "feature": "authentication",
  "written_files": [
    ".promem/features/authentication/context.md",
    ".promem/features/authentication/decisions.md"
  ],
  "message": "Memory saved. Review git diff before committing."
}
```

### 21.4 `promem_snapshot`

Input:

```json
{
  "feature": "authentication",
  "note": "Capture the work from the current branch."
}
```

Output:

```json
{
  "feature": "authentication",
  "summary": "Snapshot generated.",
  "written_files": [
    ".promem/features/authentication/changelog.md"
  ]
}
```

---

## 22. Packaging and Distribution

### 22.1 Development

```bash
cargo build
cargo test
cargo run -- init
```

### 22.2 Local install

```bash
cargo install --path .
```

### 22.3 Release targets

Build binaries for:

```txt
macOS arm64
macOS x64
Linux x64
Linux arm64
Windows x64
```

### 22.4 Distribution options

- GitHub Releases
- Homebrew tap
- Cargo install
- install script
- package managers later

### 22.5 Binary naming

CLI command:

```bash
promem
```

MCP mode:

```bash
promem mcp
```

Do not create a separate binary unless necessary.

---

## 23. Testing Strategy

### 23.1 Unit tests

Test:

- JSON parsing
- Markdown rendering
- feature key normalization
- path safety
- index updates
- search snippets
- config parsing

### 23.2 Integration tests

Use temp directories.

Test:

```bash
promem init
promem save-json authentication
promem load authentication
promem list
promem search jwt
```

### 23.3 Snapshot tests

Use snapshot testing for generated Markdown.

This prevents accidental formatting churn.

### 23.4 Git tests

Create temporary Git repos and test:

- snapshot collection
- changed files
- diff stat
- branch detection

### 23.5 MCP tests

Test MCP tools by sending JSON-RPC-like calls to the server in stdio mode.

---

## 24. Roadmap for Full Vision

This is not a tiny v0/v1 roadmap. This is the full staged vision.

### Phase A: Core local memory

Goal: Git-native project memory works.

Build:

- `promem init`
- `promem save-json`
- `promem save --from`
- Markdown rendering
- `.promem/index.json`
- `promem load`
- `promem list`
- `promem tree`
- keyword search

Success condition:

```txt
A developer can save structured feature memory, commit it, and load it later.
```

### Phase B: Git-native workflows

Goal: memory evolves with code.

Build:

- `promem snapshot`
- Git diff collection
- changed file detection
- changelog updates
- TODO scanner
- project tree awareness
- secret redaction warnings

Success condition:

```txt
A developer can capture feature progress from repo changes without copy-paste.
```

### Phase C: MCP automation

Goal: AI assistants can read/write memory automatically.

Build:

- `promem mcp`
- resources
- tools
- prompts
- handoff save tool
- context load tool
- search tool

Success condition:

```txt
An MCP-capable assistant can save distilled session memory directly into .promem/.
```

### Phase D: AI provider support

Goal: Promem can extract memory when given notes, snapshots, or handoff text.

Build:

- external mode
- OpenAI-compatible provider
- Anthropic provider
- Gemini provider
- Ollama/local provider
- structured JSON extraction
- context compression

Success condition:

```txt
Promem can generate useful memory from notes or snapshots using the user's chosen provider.
```

### Phase E: Search and context intelligence

Goal: memory becomes searchable and compact.

Build:

- chunking
- local cache
- embeddings
- semantic search
- hybrid search
- related feature expansion
- token-budgeted context loading

Success condition:

```txt
promem load can produce a compact, relevant prompt for large projects.
```

### Phase F: Knowledge graph

Goal: Promem understands relationships between project areas.

Build:

- feature links
- dependency graph
- referenced files graph
- decision graph
- related feature suggestions
- graph export

Success condition:

```txt
Promem can identify that payments depends on checkout, auth, and user roles.
```

### Phase G: Advanced integrations

Only if the product needs them.

Possible additions:

- VS Code extension
- Cursor integration
- JetBrains plugin
- GitHub PR comment importer
- issue tracker importer
- terminal UI
- hosted team mode
- web dashboard

These are optional and should not compromise the local-first core.

---

## 25. Codex Implementation Guide

Promem can be built with Codex, but the task should be broken into precise chunks.

### 25.1 Initial Codex prompt

```txt
Build a Rust CLI tool called promem.

It is a local Git-native project memory tool for AI-assisted development.

Core rules:
- Rust only.
- No TypeScript.
- No web app.
- No backend.
- No raw transcript storage.
- Store durable memory as Markdown in `.promem/`.
- Use JSON stdin/stdout contracts for integrations.
- Cache/index files are generated and should not be committed.

Use:
- clap
- serde / serde_json
- anyhow / thiserror
- toml
- walkdir or ignore

Implement commands:
- promem init
- promem save-json <feature>
- promem load <feature>
- promem list
- promem tree
- promem search <query>
- promem doctor

Create modules:
- cli.rs
- models.rs
- config.rs
- store.rs
- render.rs
- index.rs
- search.rs

Add tests:
- init creates .promem structure
- save-json writes Markdown files
- load combines context
- list reads index.json
- search finds snippets
```

### 25.2 Second Codex task

```txt
Add Git snapshot support.

Implement:
- promem snapshot <feature> --dry-run
- collect branch
- collect git status --short
- collect git diff --stat
- collect git diff --name-only
- collect recent commits
- collect TODO/FIXME comments

Do not call AI yet.
Return snapshot as JSON with --json.
```

### 25.3 Third Codex task

```txt
Add AI provider abstraction.

Implement:
- provider trait
- openai-compatible provider
- config loading from .promem/config.toml
- extract-memory command that turns handoff text into MemoryInput JSON
- do not store raw input
```

### 25.4 Fourth Codex task

```txt
Add MCP stdio server.

Expose tools:
- promem_load_context
- promem_search
- promem_save_memory
- promem_list_features
- promem_snapshot

Each tool must call existing core functions.
Do not duplicate file writing logic inside MCP.
```

---

## 26. Naming and Branding

Project name:

```txt
Promem
```

Possible tagline:

```txt
Project memory for AI development.
```

Other positioning lines:

```txt
Git-native memory for AI-assisted software teams.
```

```txt
Turn AI sessions into committed project context.
```

```txt
Promem helps AI assistants remember the project, not the chat.
```

Avoid:

```txt
Chat backup
Transcript archive
Conversation manager
```

Use:

```txt
project memory
AI context
feature memory
implementation reasoning
Git-native context
```

---

## 27. Success Metrics

Promem is successful if:

1. A developer never has to explain the same feature twice.
2. A new AI session can understand months of project context in seconds.
3. `.promem/` becomes as valuable as `README.md` or `docs/`.
4. Git diffs show reasoning changes alongside code changes.
5. Developers can switch AI providers without losing project memory.
6. Teams can share AI-derived context just by committing `.promem/`.
7. The tool remains useful even if Promem itself is deleted.

---

## 28. Final Recommendation

Build Promem as a **Rust-only, local-first, Git-native CLI**.

Core source of truth:

```txt
.promem/ Markdown files
```

Sharing layer:

```txt
Git
```

Automation layer:

```txt
MCP + snapshot
```

Integration contract:

```txt
JSON stdin/stdout
```

AI cost model:

```txt
external assistant summarizes, or user brings their own API key
```

Do not start with TypeScript.  
Do not start with a web app.  
Do not start with a hosted backend.  
Do not store raw transcripts.

The ideal first-class workflow should be:

```bash
promem init

# AI assistant or MCP produces structured memory
cat memory.json | promem save-json authentication

# Developer reviews and commits
git diff .promem
git add .promem
git commit -m "Add authentication memory"

# Later
promem load authentication --copy
```

The ideal future workflow should be:

```txt
AI assistant works on feature
↓
assistant calls Promem MCP tool
↓
Promem writes distilled feature memory
↓
developer reviews git diff
↓
memory evolves with code
```

Promem should become the local memory layer that makes every future AI coding session smarter.
