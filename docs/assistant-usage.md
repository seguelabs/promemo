# Assistant Usage

Promemo works best when an assistant distills durable project memory instead of
saving raw transcript text.

Use this pattern at the end of an AI-assisted development session:

```txt
Summarize the durable project memory from this session as Promemo handoff
Markdown. Include only reviewed facts: summary, current state, decisions,
tradeoffs, files, TODOs, open questions, future work, and useful follow-up
prompts. Do not include raw transcript text.
```

Then save the handoff:

```bash
promemo save <feature> --from handoff.md
```

Or pipe it directly:

```bash
cat handoff.md | promemo save <feature> --stdin
```

The explicit import form is equivalent:

```bash
promemo import handoff handoff.md --feature <feature>
```

## MCP Client Configuration

Promemo is a local stdio MCP server:

```bash
promemo mcp
```

You do not need a separate Promemo server per AI model provider. OpenAI,
Anthropic, Gemini, local models, and hosted models can all use the same Promemo
MCP server when the host app supports MCP. What changes is the config shape used
by each host app.

Install Promemo first:

```bash
npm install -g promemo@latest
promemo --version
```

If the host app cannot find `promemo`, get the absolute path:

```bash
which promemo
```

Then use that path as the `command` value.

### Cursor

Add Promemo to Cursor's MCP config:

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

### VS Code

Open `MCP: Open User Configuration`, then add:

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

### Claude Desktop

Add Promemo to the Claude Desktop MCP config:

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

### Claude Code

Claude Code can add a stdio MCP server from the command line:

```bash
claude mcp add promemo -- promemo mcp
```

If `promemo` is not on `PATH`, use the absolute binary path:

```bash
claude mcp add promemo -- /absolute/path/to/promemo mcp
```

### Cline and Roo Code

Cline and Roo Code use the common `mcpServers` shape:

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

### Windsurf and Antigravity

Many MCP-capable editors use the same `mcpServers` shape:

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

If your host app is based on VS Code's MCP user configuration, use the VS Code
`servers` shape instead.

### Generic Stdio MCP Host

For tools that ask only for the stdio command:

```json
{
  "command": "promemo",
  "args": ["mcp"]
}
```

### Absolute Path Example

If global npm binaries are not visible to the host app:

```json
{
  "mcpServers": {
    "promemo": {
      "command": "/Users/bk/.npm-global/bin/promemo",
      "args": ["mcp"]
    }
  }
}
```

Replace the path with the result from `which promemo` on your machine.

### Provider API Keys

Basic Promemo MCP tools do not require provider API keys. Provider keys are only
needed for commands that ask Promemo to call an LLM provider, such as extraction
from messy notes. Configure those through `.promemo/config.toml` and the
environment variable named in that config.

## Handoff Template

```md
# Feature Name

## Summary
One or two paragraphs of durable context.

## Current State
### Implemented
- What is already working.

### Pending
- What remains unresolved.

## Decisions
### Decision title
Status: accepted
Reason: Why this choice was made.
- Tradeoff or cost.

## Files
- `path/to/file.rs`: Why this file matters.

## TODOs
- [normal] Next task.

## Open Questions
- Question that still needs a decision.

## Future Work
- Follow-up that is useful but not immediate.

## Prompts
### Useful follow-up prompt
Prompt text to reuse later.
```

## Codex

Ask Codex to produce the handoff Markdown, review it, then save it:

```txt
Create a Promemo handoff for this session using the project facts we validated.
Use the Promemo handoff template. Keep it concise and do not include raw
transcript text.
```

```bash
promemo save <feature> --from handoff.md --json
```

## Claude Code

Ask Claude Code for a file-oriented handoff:

```txt
Write a Promemo handoff Markdown file for this work. Include decisions,
tradeoffs, touched files, TODOs, and open questions. Do not store the raw
conversation.
```

Then save it with:

```bash
promemo import handoff handoff.md --feature <feature> --json
```

## Cursor

Ask Cursor to summarize the changed files and decisions:

```txt
Create Promemo handoff Markdown for the current feature. Focus on what future
developers and assistants need to know before editing this area again.
```

Save from the repository root or any subdirectory:

```bash
promemo save <feature> --from handoff.md
```

## Generic Assistants

Any assistant can participate as long as it can produce structured Markdown.
The assistant does not need direct filesystem access.

1. Ask for Promemo handoff Markdown.
2. Review and edit the handoff.
3. Save it with `promemo save` or `promemo import handoff`.
4. Review `git diff .promemo`.
5. Commit the memory with the related code or as a separate documentation
   change.
