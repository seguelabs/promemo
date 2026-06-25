# Generic Assistants

Any assistant can work with Promem if it can produce structured Markdown and run
local commands.

Recommended flow:

```txt
User: Save this session to Promem under authentication.
Assistant: Distills the session into handoff Markdown.
Assistant: Runs promem save authentication --stdin.
Assistant: Shows git diff .promem.
```

Rules:

- do not store raw transcripts
- keep memory feature-specific
- prefer accepted decisions and concrete TODOs
- include open questions when uncertainty remains
- let the user review Git diffs before committing

