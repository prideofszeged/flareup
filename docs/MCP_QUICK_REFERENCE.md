# MCP Development Quick Reference

## 🚀 Quick Start Commands

```bash
# Health check
just mcp-health

# Create new feature
just mcp-new-feature <name>

# Query state
just mcp-query "/<endpoint>"

# View logs
just mcp-logs 20
```

## 📋 Feature Development Checklist

When adding a new feature:

- [ ] Implement backend logic
- [ ] Add debug endpoint to `src-tauri/src/debug_api.rs`
- [ ] Add MCP tool to `packages/mcp-server/src/index.ts`
- [ ] Add client method to `packages/mcp-server/src/client.ts`
- [ ] Update `test-mcp-server.sh`
- [ ] Test: `just mcp-query "/<endpoint>"`
- [ ] Validate: `just mcp-health`
- [ ] Ask Claude to verify via MCP
- [ ] Commit

## 🔧 MCP Tools Available

| Tool | What It Does |
|------|--------------|
| `flareup_health` | Server status |
| `flareup_list_apps` | Installed apps |
| `flareup_list_plugins` | Plugins |
| `flareup_list_snippets` | Snippets |
| `flareup_list_quicklinks` | Quicklinks |
| `flareup_get_frecency` | Usage stats |
| `flareup_get_aliases` | Aliases |
| `flareup_get_settings` | Settings |
| `flareup_get_ai_settings` | AI config |
| `flareup_get_logs` | Logs |
| `flareup_get_log_config` | Log config |
| `flareup_set_log_level` | Set log level |
| `flareup_clear_logs` | Clear logs |

## 🎯 Common Workflows

### Debug a Feature
```bash
just mcp-query "/<endpoint>"
just mcp-logs 50 | grep feature_name
```

### Add New Feature
```bash
just mcp-new-feature my_feature
# Follow generated checklist in /tmp/my_feature_checklist.md
```

### Validate Before Commit
```bash
just mcp-health
# Should show all ✓ passing
```

### AI-Assisted Testing
Ask Claude:
```
"Use flareup_get_<feature> to verify it works"
"Check flareup_get_logs for any errors related to X"
"Use flareup_list_plugins and confirm Y is loaded"
```

## 📁 Key Files

- **Workflow**: `.agent/workflows/mcp-driven-development.md`
- **Tests**: `test-mcp-server.sh`
- **Scaffold**: `scripts/new-mcp-feature.sh`
- **MCP Server**: `packages/mcp-server/src/index.ts`
- **Debug API**: `src-tauri/src/debug_api.rs`

## 💡 Remember

**The Meta Loop:**
Code → Add MCP Tool → Ask Claude → Validate → Done

**Traditional:** 2-5 min per iteration
**MCP-Driven:** 10-30 sec per iteration

**= 10x faster! 🚀**
