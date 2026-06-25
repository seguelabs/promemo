# Codex

When the user asks Codex to save the current session to Promem:

1. Distill only durable project memory.
2. Write structured handoff Markdown to a temporary file, or pipe it directly.
3. Run:

```bash
promem import handoff /tmp/promem-handoff.md --feature <feature>
```

or:

```bash
promem save <feature> --stdin
```

4. Show:

```bash
git diff .promem
```

Do not store the raw conversation transcript.

