# Claude Code

When the user asks Claude Code to save a session to Promem:

1. Summarize the conversation into structured handoff Markdown.
2. Use the feature name provided by the user.
3. Run:

```bash
promem import handoff /tmp/promem-handoff.md --feature <feature>
```

4. Ask the user to review:

```bash
git diff .promem
```

The handoff should capture decisions, TODOs, open questions, and future work,
not raw chat logs.

