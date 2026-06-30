# Promemo

Git-native project memory for AI-assisted development.

## Usage

Run without a global install:

```bash
npx promemo --version
npx promemo init
npx promemo mcp
```

Or install globally:

```bash
npm install -g promemo
promemo --version
```

The npm package includes bundled native binaries, so users do not need GitHub
authentication during install.

Supported platforms:

- macOS Apple Silicon: `darwin-arm64`
- Linux x64: `linux-x64`
- Windows x64: `win32-x64`

## MCP

After installing globally, configure an MCP client with:

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

Project: <https://github.com/seguelabs/promemo>
