# Promemo CLI Usage

Generated from `promemo --help`.

```txt
Git-native project memory for AI-assisted development

Usage: promemo <COMMAND>

Commands:
  init         Create the .promemo repository structure
  mcp          Start the Promemo MCP server over stdio
  completions  Generate shell completion scripts
  docs         Generate Markdown CLI usage documentation
  index        Manage the generated local search index
  schema       Print machine-readable schemas
  memory       Read or write memory through the stable JSON request envelope
  extract      Extract structured memory from source text using the configured provider
  save-json    Save a MemoryInput JSON document for a feature
  save         Parse and save assistant handoff Markdown for a feature
  load         Load prompt-ready context for a feature
  list         List saved feature names
  tree         Print saved memory files
  search       Search saved memory
  doctor       Validate the current Promemo repository
  snapshot     Capture git state and TODO context for a feature
  open         Open a feature directory in the OS file browser
  import       Import memory from external formats
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help

  -V, --version
          Print version
```

## Commands

### `init`

```txt
Create the .promemo repository structure

Usage: init

Options:
  -h, --help
          Print help
```

### `mcp`

```txt
Start the Promemo MCP server over stdio

Usage: mcp

Options:
  -h, --help
          Print help
```

### `completions`

```txt
Generate shell completion scripts

Usage: completions <SHELL>

Arguments:
  <SHELL>
          Shell to generate completions for
          
          [possible values: bash, elvish, fish, powershell, zsh]

Options:
  -h, --help
          Print help
```

### `docs`

```txt
Generate Markdown CLI usage documentation

Usage: docs

Options:
  -h, --help
          Print help
```

### `index`

```txt
Manage the generated local search index

Usage: index <COMMAND>

Commands:
  rebuild  Rebuild the local search index
  status   Show local search index status
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `schema`

```txt
Print machine-readable schemas

Usage: schema <COMMAND>

Commands:
  memory-input  Print the MemoryInput JSON schema
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `memory`

```txt
Read or write memory through the stable JSON request envelope

Usage: memory <COMMAND>

Commands:
  preview  Preview a memory save request without writing files
  save     Save a memory save request
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `extract`

```txt
Extract structured memory from source text using the configured provider

Usage: extract [OPTIONS] <FEATURE>

Arguments:
  <FEATURE>
          Feature name to write extracted memory under

Options:
      --from <FILE>
          Read extraction input from a file

      --stdin
          Read extraction input from stdin

      --dry-run
          Preview writes without changing files

      --json
          Print JSON output

  -h, --help
          Print help
```

### `save-json`

```txt
Save a MemoryInput JSON document for a feature

Usage: save-json [OPTIONS] <FEATURE>

Arguments:
  <FEATURE>
          Feature name to save

Options:
      --dry-run
          Preview writes without changing files

  -h, --help
          Print help
```

### `save`

```txt
Parse and save assistant handoff Markdown for a feature

Usage: save [OPTIONS] <FEATURE>

Arguments:
  <FEATURE>
          Feature name to save

Options:
      --from <FILE>
          Read handoff Markdown from a file

      --stdin
          Read handoff Markdown from stdin

      --json
          Print JSON output

      --dry-run
          Preview writes without changing files

  -h, --help
          Print help
```

### `load`

```txt
Load prompt-ready context for a feature

Usage: load [OPTIONS] <FEATURE>

Arguments:
  <FEATURE>
          Feature name to load

Options:
      --token-budget <TOKEN_BUDGET>
          Approximate maximum number of whitespace-delimited tokens to return

      --related
          Include related feature summaries from the local relationship graph

      --json
          Print JSON output

  -h, --help
          Print help
```

### `list`

```txt
List saved feature names

Usage: list [OPTIONS]

Options:
      --json
          Print JSON output

  -h, --help
          Print help
```

### `tree`

```txt
Print saved memory files

Usage: tree [OPTIONS]

Options:
      --json
          Print JSON output

  -h, --help
          Print help
```

### `search`

```txt
Search saved memory

Usage: search [OPTIONS] <QUERY>

Arguments:
  <QUERY>
          Search query

Options:
      --semantic
          Rank chunked results using local deterministic embeddings

      --hybrid
          Rank chunked results with keyword and semantic signals

      --limit <LIMIT>
          Maximum number of matches to return

      --json
          Print JSON output

  -h, --help
          Print help
```

### `doctor`

```txt
Validate the current Promemo repository

Usage: doctor [OPTIONS]

Options:
      --json
          Print JSON output

  -h, --help
          Print help
```

### `snapshot`

```txt
Capture git state and TODO context for a feature

Usage: snapshot [OPTIONS] <FEATURE>

Arguments:
  <FEATURE>
          Feature name to snapshot

Options:
      --dry-run
          Preview the snapshot without writing files

      --json
          Print JSON output

  -h, --help
          Print help
```

### `open`

```txt
Open a feature directory in the OS file browser

Usage: open <FEATURE>

Arguments:
  <FEATURE>
          Feature name to open

Options:
  -h, --help
          Print help
```

### `import`

```txt
Import memory from external formats

Usage: import <COMMAND>

Commands:
  handoff  Import assistant handoff Markdown from a file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `help`

```txt
Print this message or the help of the given subcommand(s)

Usage: help [COMMAND]...

Arguments:
  [COMMAND]...
          Print help for the subcommand(s)
```

