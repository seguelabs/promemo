# Promem — Part 2: Memory Engine Features

**Project:** Promem  
**Document Type:** Product Architecture / Feature Expansion Guide  
**Scope:** Supermemory-inspired memory-engine features only  
**Status:** Full Vision Addendum  
**Primary Storage:** `.promem/` committed to Git  
**Philosophy:** Repo-native memory, not general personal memory  

---

## 1. Purpose of This Addendum

The first Promem guide defined the core product:

```txt
AI coding session / handoff / snapshot
↓
Structured project memory
↓
.promem/ Markdown files
↓
Git commit
↓
Reusable context for future AI sessions
```

This second part defines the **memory-engine layer** that makes Promem stronger than a Markdown writer.

The goal is not to copy Supermemory as a general-purpose user memory platform.  
The goal is to borrow useful memory-system ideas and adapt them to a **Git-native software repository memory tool**.

Promem should remain:

```txt
local-first
repo-native
Markdown-first
Git-compatible
AI-provider agnostic
no raw transcript storage by default
```

The recommended feature layer:

```txt
1. Semantic search
2. Query rewriting
3. Reranking
4. Memory graph
5. Status and staleness tracking
6. Contradiction detection
7. Repo-focused connectors and importers
```

---

## 2. Product Positioning

Promem should not become:

```txt
general user memory
personal profile memory
browser-wide memory
chat history backup
hosted memory API first
```

Promem should become:

```txt
Supermemory-style recall for a Git repository.
```

Simple positioning:

```txt
Promem is Git-native memory for AI-assisted software projects.
```

Sharper positioning:

```txt
Promem turns AI coding sessions into committed project memory that future AI sessions can search, load, and trust.
```

The core distinction:

```txt
Supermemory remembers users for agents.
Promem remembers repositories for AI-assisted development.
```

---

## 3. Design Principle

The `.promem/` folder remains the source of truth.

```txt
.promem/
  project.md
  config.toml
  index.json

  features/
    authentication/
      context.md
      decisions.md
      architecture.md
      api.md
      todos.md
      changelog.md

    payments/
      context.md
      decisions.md
      architecture.md
      api.md
      todos.md

  shared/
    coding-guidelines.md
    conventions.md
    architecture.md

  cache/
    chunks.db
    embeddings.db
    graph.db
    rerank-cache.db
```

Commit:

```txt
.promem/project.md
.promem/config.toml
.promem/index.json
.promem/features/**/*.md
.promem/shared/**/*.md
```

Do not commit:

```txt
.promem/cache/
.promem/embeddings.db
.promem/chunks.db
.promem/graph.db
.promem/rerank-cache.db
```

Reason:

```txt
Markdown is durable memory.
Cache is generated intelligence.
```

---

# Feature 1 — Semantic Search

## 1.1 Goal

Keyword search is not enough.

A developer may ask:

```txt
why did we rotate tokens
```

But the memory may say:

```txt
Refresh token rotation was chosen to reduce replay risk.
```

Semantic search allows Promem to find meaning, not just exact words.

## 1.2 Command Examples

```bash
promem search "why did we rotate tokens"
promem search "refund retry behavior"
promem search "what handles webhook idempotency"
promem search "why admin theme is separate"
```

With smart mode:

```bash
promem search "why did we rotate tokens" --semantic
promem search "refund retry behavior" --smart
```

## 1.3 What It Searches

Promem should search across:

```txt
project.md
shared/*.md
features/*/context.md
features/*/decisions.md
features/*/architecture.md
features/*/api.md
features/*/todos.md
features/*/changelog.md
```

## 1.4 Chunking Strategy

Promem should split Markdown by headings.

Example:

```md
# Authentication Decisions

## Refresh Token Rotation

Decision:
Use refresh token rotation.

Reason:
Reduces replay risk if a refresh token leaks.

Tradeoffs:
Requires token family tracking.
```

Chunk:

```json
{
  "document": ".promem/features/authentication/decisions.md",
  "heading": "Refresh Token Rotation",
  "content": "Decision: Use refresh token rotation. Reason: Reduces replay risk...",
  "type": "decision",
  "feature": "authentication"
}
```

## 1.5 Storage

Local cache:

```txt
.promem/cache/chunks.db
.promem/cache/embeddings.db
```

Suggested local stack:

```txt
SQLite
sqlite-vec
```

Alternative later:

```txt
LanceDB
tantivy
hybrid keyword + vector index
```

## 1.6 Search Pipeline

```txt
User query
↓
optional query rewrite
↓
embed query
↓
retrieve top candidate chunks
↓
optional rerank
↓
return ranked snippets
```

## 1.7 JSON Output Contract

```bash
promem search "refresh token replay risk" --json
```

Output:

```json
{
  "query": "refresh token replay risk",
  "results": [
    {
      "path": ".promem/features/authentication/decisions.md",
      "feature": "authentication",
      "type": "decision",
      "heading": "Refresh Token Rotation",
      "score": 0.91,
      "snippet": "Refresh token rotation was chosen to reduce replay risk..."
    }
  ]
}
```

## 1.8 Why This Matters

This turns Promem from static notes into a retrieval engine.

Without semantic search:

```txt
Promem is documentation.
```

With semantic search:

```txt
Promem becomes AI-readable project memory.
```

---

# Feature 2 — Query Rewriting

## 2.1 Goal

Developers ask messy questions.

Examples:

```txt
auth token thing
why retry payments
what was issue with admin theme
that websocket thing
```

Promem should rewrite vague questions into better search queries.

## 2.2 Example

Input:

```txt
why auth token thing done like this
```

Rewritten query:

```txt
authentication JWT refresh token rotation decision rationale tradeoffs
```

Input:

```txt
refund retry
```

Rewritten query:

```txt
payments refund webhook retry policy idempotency failure handling
```

## 2.3 Command Examples

```bash
promem search "auth token thing" --rewrite
promem search "refund retry" --smart
```

## 2.4 Pipeline

```txt
Raw user query
↓
Read project index and feature names
↓
Rewrite query into repo-aware terms
↓
Run search
↓
Return results with both original and rewritten query
```

## 2.5 Context for Rewriting

Promem can use:

```txt
.promem/index.json
feature names
known APIs
known file paths
known decisions
project.md
shared/conventions.md
```

It should not need the whole repo for rewriting.

## 2.6 JSON Output

```json
{
  "original_query": "auth token thing",
  "rewritten_query": "authentication JWT refresh token rotation decision rationale",
  "results": []
}
```

## 2.7 Implementation Modes

Promem should support three modes:

```txt
off
local heuristic
AI-assisted
```

Config:

```toml
[search]
query_rewriting = "heuristic"
```

or:

```toml
[search]
query_rewriting = "ai"
```

## 2.8 Why This Matters

Most users do not remember the exact technical words used in old decisions.

Query rewriting bridges:

```txt
human vague memory
↓
repo-specific technical memory
```

---

# Feature 3 — Reranking

## 3.1 Goal

Semantic search may retrieve related chunks, but not always the best ones.

Reranking improves final context quality.

Promem should retrieve more than it needs, then rerank the candidates based on the actual user request.

## 3.2 Example

Query:

```txt
why did we use refresh token rotation
```

Initial retrieval:

```txt
authentication/context.md
authentication/todos.md
security/conventions.md
authentication/decisions.md
payments/webhooks.md
```

Reranked output:

```txt
1. authentication/decisions.md#Refresh Token Rotation
2. authentication/architecture.md#Token Lifecycle
3. security/conventions.md#Session Security
4. authentication/context.md#Current State
```

## 3.3 Command Examples

```bash
promem search "refresh token replay risk" --rerank
promem load authentication --smart
promem load payments --smart --max-tokens 8000
```

## 3.4 Reranking Pipeline

```txt
Retrieve top 30 chunks
↓
Score each chunk against the user query
↓
Boost decisions, architecture, APIs, current feature
↓
Penalize stale/deprecated docs
↓
Return top 5–10 chunks
```

## 3.5 Scoring Factors

Promem should consider:

```txt
semantic similarity
feature match
document type
recency
status
decision importance
file relevance
cross-feature relationship
staleness
```

Suggested weights:

```txt
decision document: +0.15
architecture document: +0.10
current feature: +0.20
deprecated status: -0.50
superseded status: -0.35
recent update: +0.05
linked file match: +0.10
```

## 3.6 JSON Output

```json
{
  "query": "refresh token replay risk",
  "results": [
    {
      "path": ".promem/features/authentication/decisions.md",
      "heading": "Refresh Token Rotation",
      "initial_score": 0.77,
      "rerank_score": 0.94,
      "reasons": [
        "decision document",
        "feature match",
        "high semantic similarity",
        "accepted status"
      ]
    }
  ]
}
```

## 3.7 Why This Matters

The value of Promem is not just finding something.

The value is loading the **right** context into an AI session.

Bad retrieval creates noisy context.

Good reranking creates useful context.

---

# Feature 4 — Memory Graph

## 4.1 Goal

Software features are connected.

Promem should understand relationships like:

```txt
Authentication → User Model
Payments → Checkout
Checkout → Webhooks
Webhooks → Retry Policy
Admin → Design System
```

This allows Promem to expand context intelligently.

## 4.2 Why a Graph Matters

If a user runs:

```bash
promem load refunds --smart
```

Promem may need to include:

```txt
payments
checkout
webhooks
idempotency
```

Even if the user only asked for refunds.

## 4.3 Relationship Types

Promem should support:

```txt
depends_on
related_to
calls_api
uses_database_table
references_file
supersedes
superseded_by
conflicts_with
implements
owned_by_feature
```

## 4.4 Source of Relationships

Relationships can come from:

```txt
frontmatter tags
referenced files
API references
database table names
explicit Markdown links
semantic similarity
snapshot analysis
manual user edits
AI extraction
```

## 4.5 Frontmatter Example

```md
---
feature: authentication
type: decisions
status: accepted
related:
  - users
  - sessions
references:
  files:
    - src/auth/token-service.ts
    - src/db/session-table.ts
  api:
    - POST /auth/refresh
depends_on:
  - users
---
```

## 4.6 Graph Storage

Generated graph cache:

```txt
.promem/cache/graph.db
```

Do not commit graph cache.

If a relationship is important and human-authored, store it in frontmatter or Markdown.

## 4.7 Command Examples

```bash
promem graph authentication
promem related payments
promem load refunds --include-related
promem load checkout --depth 2
```

## 4.8 JSON Output

```json
{
  "feature": "refunds",
  "related": [
    {
      "feature": "payments",
      "relationship": "depends_on",
      "score": 0.91
    },
    {
      "feature": "webhooks",
      "relationship": "related_to",
      "score": 0.84
    }
  ]
}
```

## 4.9 Context Expansion Rules

Promem should avoid dumping the whole graph.

Use rules:

```txt
depth 1 by default
include accepted decisions first
include architecture before todos
include stale docs only if explicitly requested
summarize related features
do not include everything
```

## 4.10 Why This Matters

A codebase is not a folder tree.

It is a network of decisions.

The graph lets Promem load memory the way software is actually connected.

---

# Feature 5 — Status and Staleness Tracking

## 5.1 Goal

Bad memory is worse than no memory.

Promem must know which memories are current, proposed, deprecated, superseded, stale, or conflicting.

## 5.2 Status Types

Supported status values:

```txt
accepted
proposed
experimental
deprecated
superseded
stale
conflicting
unknown
```

## 5.3 Frontmatter Example

```md
---
feature: authentication
type: decision
status: superseded
superseded_by: .promem/features/authentication/decisions.md#server-side-sessions
updated_at: 2026-06-25
review_after: 2026-09-25
---
```

## 5.4 Command Examples

```bash
promem status
promem stale
promem review
promem mark authentication/decisions.md --status deprecated
promem mark authentication --review-after 90d
```

## 5.5 Staleness Signals

Promem should detect possible staleness from:

```txt
old updated_at
referenced files deleted
referenced APIs removed
git diff changed related files
conflicting newer decision exists
feature no longer appears in repo
TODOs unchanged for too long
```

## 5.6 Load Behavior

By default:

```txt
accepted: include
proposed: include if relevant
experimental: include with warning
deprecated: exclude unless requested
superseded: exclude and reference replacement
stale: include only with warning
conflicting: warn loudly
```

Commands:

```bash
promem load auth
promem load auth --include-stale
promem load auth --current-only
```

## 5.7 JSON Output

```json
{
  "feature": "authentication",
  "status_summary": {
    "accepted": 8,
    "proposed": 2,
    "deprecated": 1,
    "stale": 1
  },
  "warnings": [
    {
      "path": ".promem/features/authentication/decisions.md",
      "message": "Decision references deleted file src/auth/legacy-session.ts"
    }
  ]
}
```

## 5.8 Why This Matters

LLMs follow context strongly.

If Promem gives stale context, the AI may implement the wrong thing.

Staleness tracking protects future AI sessions from outdated reasoning.

---

# Feature 6 — Contradiction Detection

## 6.1 Goal

As project memory grows, conflicts will happen.

Promem should detect contradictions before they poison context.

## 6.2 Example

Old decision:

```txt
Use JWT-only stateless authentication.
```

New decision:

```txt
Move sessions to Redis-backed server-side sessions.
```

Promem should flag:

```txt
Potential conflict:
authentication/decisions.md says JWT-only.
sessions/architecture.md says Redis sessions.
```

## 6.3 Conflict Types

Promem should detect:

```txt
decision vs decision
architecture vs architecture
TODO marked complete but still present
API removed but still documented
file reference deleted
database table renamed
dependency removed but referenced
status conflict
```

## 6.4 Command Examples

```bash
promem conflicts
promem conflicts authentication
promem resolve
promem resolve authentication --mark-superseded
```

## 6.5 Detection Pipeline

```txt
Run promem sync
↓
Parse all Markdown/frontmatter
↓
Extract decisions, APIs, files, dependencies
↓
Compare across features and time
↓
Use heuristics first
↓
Optionally use AI for deeper contradiction detection
↓
Write conflict report
```

## 6.6 Conflict Report

```txt
.promem/cache/conflicts.json
```

Example:

```json
{
  "conflicts": [
    {
      "id": "conflict_auth_sessions_001",
      "severity": "high",
      "type": "decision_vs_decision",
      "left": {
        "path": ".promem/features/authentication/decisions.md",
        "claim": "Use JWT-only stateless authentication."
      },
      "right": {
        "path": ".promem/features/sessions/architecture.md",
        "claim": "Use Redis-backed server-side sessions."
      },
      "suggested_actions": [
        "mark older decision as superseded",
        "update authentication architecture",
        "link new sessions decision"
      ]
    }
  ]
}
```

## 6.7 Resolution Workflow

```bash
promem conflicts
promem resolve conflict_auth_sessions_001
```

Promem asks:

```txt
1. Mark left as superseded
2. Mark right as superseded
3. Keep both and explain scope
4. Ignore
5. Open files manually
```

Result:

```md
---
status: superseded
superseded_by: .promem/features/sessions/architecture.md#redis-backed-sessions
---
```

## 6.8 Why This Matters

Promem should not just accumulate memory.

It should maintain memory quality.

Contradiction detection turns Promem into a trusted context layer.

---

# Feature 7 — Repo-Focused Connectors and Importers

## 7.1 Goal

Promem should import useful project knowledge from external tools, but only when it strengthens repository memory.

It should not become a general life/work memory collector.

## 7.2 Rule

Before importing anything, ask:

```txt
Does this help the repository remember why it was built this way?
```

If no, do not import it.

## 7.3 Good Import Sources

Useful sources:

```txt
GitHub PR discussions
GitHub issues
Linear tickets
Jira tickets
Slack engineering threads
Architecture Decision Records
Figma comments for design-system work
Claude/Cursor/Codex handoff outputs
terminal session summaries
meeting notes about technical decisions
```

## 7.4 Bad Import Sources

Avoid:

```txt
general browser history
personal notes unrelated to repo
random chats
full raw transcripts
company-wide everything
personal profile data
private user preferences
```

## 7.5 Importer Commands

```bash
promem import github-pr 123
promem import github-issue 88
promem import linear PROJ-42
promem import adr docs/adr/001-auth.md
promem import handoff handoff.md --feature authentication
promem import slack-thread --url <url> --feature payments
```

## 7.6 Import Pipeline

```txt
Fetch source
↓
Extract repo-relevant information
↓
Convert to structured memory JSON
↓
Render Markdown
↓
Update .promem/features/*
↓
Developer reviews git diff
↓
Developer commits
```

## 7.7 No Raw Source Storage by Default

Promem should not store full source content unless the user opts in.

Default:

```txt
store extracted project memory
store source metadata
do not store full raw thread/transcript
```

Example metadata:

```md
---
source:
  type: github_pr
  id: 123
  url: https://github.com/org/repo/pull/123
  imported_at: 2026-06-26
---
```

## 7.8 GitHub PR Import Example

Command:

```bash
promem import github-pr 123 --feature authentication
```

Output:

```txt
.promem/features/authentication/changelog.md
.promem/features/authentication/decisions.md
.promem/features/authentication/todos.md
```

Extracted memory:

```md
# PR 123 Memory

## Summary

Implemented refresh token rotation and added token family invalidation.

## Decisions

- Token reuse invalidates the entire token family.
- Refresh token metadata is stored server-side.

## Files

- src/auth/token-service.ts
- src/db/refresh-tokens.ts

## TODOs

- Add audit logging for token reuse events.
```

## 7.9 Connector Priority

Build importers in this order:

```txt
1. local handoff Markdown
2. GitHub PR
3. GitHub issue
4. ADR docs
5. Linear/Jira
6. Slack engineering thread
7. Figma/design comments
```

## 7.10 Why This Matters

A lot of project reasoning does not live in AI chats.

It lives in:

```txt
PR comments
issue threads
architecture docs
design comments
commit messages
```

Repo-focused importers let Promem collect reasoning from the places developers already work.

---

# Combined Smart Context Flow

When all 7 features work together:

```txt
User asks:
promem load checkout --smart

Promem:
1. Reads checkout memory
2. Rewrites the query/context goal
3. Searches semantically
4. Finds related features through graph
5. Removes stale/superseded memories
6. Detects unresolved contradictions
7. Reranks the best chunks
8. Builds compact LLM-ready context
```

Output:

```txt
Project overview
Checkout current state
Accepted decisions
Related payment/webhook context
Relevant APIs
Important files
Open TODOs
Warnings about stale/conflicting memory
```

This is the point where Promem stops being a Markdown writer and becomes a real context engine.

---

# Suggested Command Set for These Features

## Search

```bash
promem search "refund retries"
promem search "auth tokens" --semantic
promem search "admin theme" --smart
promem search "webhook retry" --json
```

## Load

```bash
promem load authentication
promem load payments --smart
promem load checkout --include-related
promem load auth --current-only
promem load auth --max-tokens 8000
```

## Graph

```bash
promem graph authentication
promem related payments
promem related checkout --depth 2
```

## Status

```bash
promem status
promem stale
promem review
promem mark auth --status accepted
```

## Conflicts

```bash
promem conflicts
promem conflicts authentication
promem resolve
```

## Import

```bash
promem import handoff handoff.md --feature authentication
promem import github-pr 123 --feature payments
promem import adr docs/adr/001-auth.md
```

## Sync

```bash
promem sync
promem sync --rebuild-index
promem sync --check-conflicts
```

---

# Recommended Implementation Order

## Phase A — Smart Search Foundation

Build:

```txt
chunking
keyword search
semantic search
local cache
basic filters
```

Commands:

```bash
promem sync
promem search "query" --semantic
```

## Phase B — Smart Context Loading

Build:

```txt
query rewriting
reranking
context ranking
max-token output
current-only loading
```

Commands:

```bash
promem load auth --smart
promem load auth --max-tokens 8000
```

## Phase C — Memory Graph

Build:

```txt
relationship extraction
related feature detection
graph cache
related command
include-related loading
```

Commands:

```bash
promem graph auth
promem related payments
promem load checkout --include-related
```

## Phase D — Memory Quality

Build:

```txt
status tracking
staleness checks
contradiction detection
resolve workflow
```

Commands:

```bash
promem stale
promem conflicts
promem resolve
```

## Phase E — Repo Importers

Build:

```txt
handoff importer
GitHub PR importer
GitHub issue importer
ADR importer
Linear/Jira importer
Slack thread importer
```

Commands:

```bash
promem import github-pr 123
promem import adr docs/adr/001-auth.md
```

---

# Recommended Rust Modules

```txt
crates/
  promem-core/
    memory.rs
    feature.rs
    status.rs
    config.rs

  promem-cli/
    main.rs
    commands/

  promem-store/
    markdown.rs
    frontmatter.rs
    index_json.rs
    fs.rs

  promem-search/
    chunking.rs
    keyword.rs
    semantic.rs
    rerank.rs
    query_rewrite.rs

  promem-graph/
    graph.rs
    relationships.rs
    related.rs

  promem-quality/
    stale.rs
    conflicts.rs
    resolve.rs

  promem-import/
    handoff.rs
    github.rs
    adr.rs
    linear.rs
    slack.rs

  promem-render/
    context.rs
    decisions.rs
    architecture.rs
    todos.rs
```

---

# Config Additions

```toml
[memory]
root = ".promem"

[search]
mode = "hybrid"
query_rewriting = "heuristic"
reranking = true
default_top_k = 10

[semantic]
enabled = true
provider = "local"
cache_dir = ".promem/cache"

[graph]
enabled = true
default_depth = 1

[quality]
staleness_days = 90
detect_conflicts = true
exclude_deprecated_by_default = true

[import]
store_raw_sources = false
```

---

# Frontmatter Standard

All Promem documents should support frontmatter:

```yaml
---
feature: authentication
type: decisions
status: accepted
updated_at: 2026-06-26
review_after: 2026-09-26
related:
  - users
  - sessions
references:
  files:
    - src/auth/token-service.ts
  api:
    - POST /auth/refresh
tags:
  - auth
  - security
  - jwt
---
```

Supported `type` values:

```txt
context
decisions
architecture
api
todos
changelog
handoff
adr
note
```

Supported `status` values:

```txt
accepted
proposed
experimental
deprecated
superseded
stale
conflicting
unknown
```

---

# What Not to Build

Avoid these unless Promem later becomes a hosted product:

```txt
general user memory
personal profile memory
raw transcript archive
browser-wide context capture
hosted memory API as the core
team SaaS dashboard
social/collaboration layer
```

Promem should remain:

```txt
repo memory first
Git-native first
Markdown-first
AI-tool agnostic
```

---

# Final Vision

Promem starts as:

```txt
A CLI that saves AI project context into .promem/
```

But with these seven features, it becomes:

```txt
A repo-native memory engine for AI-assisted development.
```

The difference is important.

Markdown is only the storage format.

The real product is:

```txt
structured extraction
semantic retrieval
repo-aware ranking
project graph
staleness control
contradiction detection
MCP-ready context loading
```

Promem should not remember everything.

Promem should remember what matters to the repository.
