# Assistant Workflows

These notes explain how AI coding assistants should save useful session memory
to Promem without storing raw transcripts.

When the user says "save this session to Promem":

1. Ask for a feature name if the user did not provide one.
2. Distill the session into structured handoff Markdown.
3. Do not include raw transcript text.
4. Save through Promem with either:

```bash
promem save <feature> --stdin
```

or:

```bash
promem import handoff /tmp/promem-handoff.md --feature <feature>
```

5. Show the user:

```bash
git diff .promem
```

The handoff Markdown should use this shape:

```md
# Feature Name

## Summary
Short durable summary.

## Current State
### Implemented
- Existing capability.

### Pending
- Planned capability.

## Decisions
### Decision title
Reason: why this decision was made.
Status: accepted
- Important tradeoff.

## TODOs
- Follow-up task.

## Open Questions
- Question to resolve later.

## Future Work
- Larger future direction.
```

