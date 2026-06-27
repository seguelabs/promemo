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
