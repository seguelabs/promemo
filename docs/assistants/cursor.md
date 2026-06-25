# Cursor

When the user asks Cursor to save memory to Promem:

1. Create structured handoff Markdown from the useful session outcome.
2. Save it with:

```bash
promem save <feature> --stdin
```

or:

```bash
promem import handoff /tmp/promem-handoff.md --feature <feature>
```

3. Show:

```bash
git diff .promem
```

Cursor rules should keep Promem memory distilled and feature-specific.

