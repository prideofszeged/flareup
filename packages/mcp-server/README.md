# @flareup/mcp-server

MCP (Model Context Protocol) server for Flareup - enables AI assistants like Claude to interact with Flareup.

## What is This?

This MCP server exposes Flareup's internal state through 13 tools that AI assistants can use to:
- Check if Flareup is running
- List installed apps, plugins, snippets, quicklinks
- Query usage statistics (frecency data)
- View application settings and configuration
- Access and filter application logs
- Monitor system health

## Quick Start

### Running the Server

```bash
# From the Flareup root directory
npx tsx packages/mcp-server/src/index.ts

# Or use pnpm from the mcp-server directory
cd packages/mcp-server
pnpm dev
```

### Using with Claude Desktop

Add to your Claude Desktop config (`~/.config/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "flareup": {
      "command": "npx",
      "args": ["tsx", "packages/mcp-server/src/index.ts"],
      "cwd": "/path/to/flareup",
      "env": {}
    }
  }
}
```

### Testing

```bash
# From Flareup root
./test-mcp-server.sh
```

## Available Tools

| Tool | Description |
|------|-------------|
| `flareup_health` | Check Flareup status and version |
| `flareup_list_apps` | List all installed applications (192 apps) |
| `flareup_list_plugins` | List discovered plugins/extensions (57 plugins) |
| `flareup_list_snippets` | List text expansion snippets |
| `flareup_list_quicklinks` | List URL/file shortcuts |
| `flareup_get_frecency` | Get usage frequency data |
| `flareup_get_aliases` | Get command aliases |
| `flareup_get_settings` | Get application settings |
| `flareup_get_ai_settings` | Get AI configuration |
| `flareup_get_logs` | Get recent logs (with filtering) |
| `flareup_clear_logs` | Clear the log buffer |
| `flareup_get_log_config` | Get log configuration |
| `flareup_set_log_level` | Set minimum log capture level |

## Architecture

```
MCP Client (Claude Desktop)
    ↓
MCP Protocol (stdio)
    ↓
MCP Server (this package)
    ↓
Flareup Client (HTTP wrapper)
    ↓
Flareup Debug API (:7266)
    ↓
Flareup Backend (Rust/Tauri)
```

## Example Usage

Once connected to an MCP client, you can ask:

```
"Use flareup_health to check if Flareup is running"
"Use flareup_list_plugins to see what plugins are installed"
"Use flareup_get_logs to show recent errors"
"Use flareup_get_frecency to see most-used items"
```

## Development

### Adding a New Tool

1. **Define the tool** in `TOOLS` array:
```typescript
{
    name: 'flareup_my_tool',
    description: 'What this tool does',
    inputSchema: {
        type: 'object',
        properties: {
            // parameters
        },
        required: [],
    },
}
```

2. **Add client method** in `client.ts`:
```typescript
async getMyData(): Promise<ApiResponse<MyData>> {
    return this.fetch<MyData>('/my-endpoint');
}
```

3. **Add handler** in `executeTool()`:
```typescript
case 'flareup_my_tool': {
    const result = await flareupClient.getMyData();
    if (result.success && result.data) {
        return JSON.stringify(result.data, null, 2);
    }
    return `Error: ${result.error}`;
}
```

4. **Update test script**: Add to `test-mcp-server.sh`

5. **Test**: `./test-mcp-server.sh`

### Adding a New Debug Endpoint

See `.agent/workflows/mcp-driven-development.md` for the complete workflow.

Short version:
```bash
just mcp-new-feature <feature_name>
# Follow the generated checklist
```

## Requirements

- Node.js 18+
- Flareup running with debug API enabled (port 7266)
- `@modelcontextprotocol/sdk` package

## License

Same as Flareup main project.

## See Also

- [MCP Development Workflow](../../.agent/workflows/mcp-driven-development.md)
- [MCP Quick Reference](../../docs/MCP_QUICK_REFERENCE.md)
- [Debug API Source](../../src-tauri/src/debug_api.rs)
