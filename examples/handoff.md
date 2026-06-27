# Product Direction

## Summary
Promemo is a Git-native project memory CLI. v1 focuses on assistant-distilled
handoff workflows that do not require users to manually write JSON.

## Current State
### Implemented
- v0 local Markdown memory works.
- Upward repo discovery works from subdirectories.

### Pending
- MCP automation.
- AI provider extraction.

## Decisions
### Use structured handoff Markdown before MCP
Reason: it works across Codex, Claude Code, Cursor, and VS Code without requiring assistant-specific APIs.
Status: accepted
- Requires assistants to produce predictable sections.
- Avoids raw transcript storage.

## TODOs
- Add assistant usage docs.
- Improve snapshot dry-run output.

## Open Questions
- Should v1 include optional git staging, or leave that for later?

## Future Work
- Add MCP save-memory support after the local handoff path is stable.
