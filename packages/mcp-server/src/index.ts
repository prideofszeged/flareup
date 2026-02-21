#!/usr/bin/env node
/**
 * Flareup MCP Server
 * 
 * Enables AI assistants like Claude to interact with Flareup via MCP protocol.
 * Wraps the Flareup HTTP debug API with MCP tools.
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
    CallToolRequestSchema,
    ListToolsRequestSchema,
    Tool,
} from '@modelcontextprotocol/sdk/types.js';
import { flareupClient } from './client.js';

// Define all Flareup MCP tools
const TOOLS: Tool[] = [
    {
        name: 'flareup_health',
        description: 'Check if Flareup is running and get status information including version and uptime.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_list_apps',
        description: 'List all installed applications that Flareup can launch. Returns app name, exec command, icon, and description.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_list_plugins',
        description: 'List all discovered plugins/extensions installed in Flareup. Returns plugin name, title, description, and enabled status.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_list_snippets',
        description: 'List all text snippets configured in Flareup. Snippets are text expansions triggered by keywords.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_list_quicklinks',
        description: 'List all quicklinks configured in Flareup. Quicklinks are shortcuts to URLs or files.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_get_frecency',
        description: 'Get usage frequency data for Flareup items. Shows which items are used most often.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_get_aliases',
        description: 'Get all command aliases configured in Flareup. Aliases map short names to command IDs.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_get_settings',
        description: 'Get current Flareup application settings including theme, window behavior, and performance options.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_get_ai_settings',
        description: 'Get Flareup AI configuration including provider, model associations, and tool settings.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_get_logs',
        description: 'Get recent application logs from Flareup. Useful for debugging and monitoring. Returns logs in reverse chronological order (newest first).',
        inputSchema: {
            type: 'object',
            properties: {
                limit: {
                    type: 'number',
                    description: 'Maximum number of log entries to return (default: 100, max: 1000)',
                },
                level: {
                    type: 'string',
                    enum: ['error', 'warn', 'info', 'debug', 'trace'],
                    description: 'Filter logs by level (e.g., "error", "warn", "info")',
                },
                search: {
                    type: 'string',
                    description: 'Search term to filter logs by message or target',
                },
            },
            required: [],
        },
    },
    {
        name: 'flareup_clear_logs',
        description: 'Clear the log buffer. Use sparingly - mainly for testing or after reviewing a large number of logs.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_get_log_config',
        description: 'Get the current log capture configuration, including the minimum log level being captured.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
    {
        name: 'flareup_set_log_level',
        description: 'Set the minimum log level to capture. Use "trace" or "debug" for verbose logging, "info" for normal operation, or "error" for minimal logging.',
        inputSchema: {
            type: 'object',
            properties: {
                level: {
                    type: 'string',
                    enum: ['trace', 'debug', 'info', 'warn', 'error'],
                    description: 'The minimum log level to capture',
                },
            },
            required: ['level'],
        },
    },
    {
        name: 'flareup_get_jump_mode',
        description: 'Get jump mode configuration and status. Shows if fd (file finder) is available, configured editor, search settings, and whether the feature is enabled.',
        inputSchema: {
            type: 'object',
            properties: {},
            required: [],
        },
    },
];

// Tool execution handlers
async function executeTool(name: string, _args: Record<string, unknown>): Promise<string> {
    switch (name) {
        case 'flareup_health': {
            const result = await flareupClient.health();
            if (result.success && result.data) {
                return `Flareup is running!\nVersion: ${result.data.version}\nUptime: ${result.data.uptime_seconds} seconds`;
            }
            return `Flareup is not running or not reachable: ${result.error}`;
        }

        case 'flareup_list_apps': {
            const result = await flareupClient.listApps();
            if (result.success && result.data) {
                const apps = result.data.slice(0, 50); // Limit to first 50
                const appList = apps.map(a => `- ${a.name}${a.comment ? `: ${a.comment}` : ''}`).join('\n');
                return `Found ${result.data.length} installed apps:\n${appList}${result.data.length > 50 ? '\n... and more' : ''}`;
            }
            return `Error listing apps: ${result.error}`;
        }

        case 'flareup_list_plugins': {
            const result = await flareupClient.listPlugins();
            if (result.success && result.data) {
                if (result.data.length === 0) {
                    return 'No plugins installed.';
                }
                const pluginList = result.data.map(p =>
                    `- ${p.title || p.name}${p.enabled ? '' : ' (disabled)'}${p.description ? `: ${p.description}` : ''}`
                ).join('\n');
                return `Found ${result.data.length} plugins:\n${pluginList}`;
            }
            return `Error listing plugins: ${result.error}`;
        }

        case 'flareup_list_snippets': {
            const result = await flareupClient.listSnippets();
            if (result.success && result.data) {
                if (result.data.length === 0) {
                    return 'No snippets configured.';
                }
                const snippetList = result.data.map(s =>
                    `- "${s.keyword}" → ${s.name} (used ${s.use_count} times)`
                ).join('\n');
                return `Found ${result.data.length} snippets:\n${snippetList}`;
            }
            return `Error listing snippets: ${result.error}`;
        }

        case 'flareup_list_quicklinks': {
            const result = await flareupClient.listQuicklinks();
            if (result.success && result.data) {
                if (result.data.length === 0) {
                    return 'No quicklinks configured.';
                }
                const linkList = result.data.map(q => `- ${q.name}: ${q.link}`).join('\n');
                return `Found ${result.data.length} quicklinks:\n${linkList}`;
            }
            return `Error listing quicklinks: ${result.error}`;
        }

        case 'flareup_get_frecency': {
            const result = await flareupClient.getFrecency();
            if (result.success && result.data) {
                if (result.data.length === 0) {
                    return 'No frecency data yet - no items have been used.';
                }
                const sorted = [...result.data].sort((a, b) => b.total_score - a.total_score).slice(0, 20);
                const frecencyList = sorted.map(f =>
                    `- ${f.item_id}: score ${f.total_score.toFixed(2)}, used ${f.use_count} times`
                ).join('\n');
                return `Top ${sorted.length} items by frecency:\n${frecencyList}`;
            }
            return `Error getting frecency: ${result.error}`;
        }

        case 'flareup_get_aliases': {
            const result = await flareupClient.getAliases();
            if (result.success && result.data) {
                const aliases = Object.entries(result.data);
                if (aliases.length === 0) {
                    return 'No aliases configured.';
                }
                const aliasList = aliases.map(([alias, cmd]) => `- "${alias}" → ${cmd}`).join('\n');
                return `Found ${aliases.length} aliases:\n${aliasList}`;
            }
            return `Error getting aliases: ${result.error}`;
        }

        case 'flareup_get_settings': {
            const result = await flareupClient.getSettings();
            if (result.success && result.data) {
                return JSON.stringify(result.data, null, 2);
            }
            return `Error getting settings: ${result.error}`;
        }

        case 'flareup_get_ai_settings': {
            const result = await flareupClient.getAiSettings();
            if (result.success && result.data) {
                return JSON.stringify(result.data, null, 2);
            }
            return `Error getting AI settings: ${result.error}`;
        }

        case 'flareup_get_logs': {
            const limit = typeof _args.limit === 'number' ? _args.limit : undefined;
            const level = typeof _args.level === 'string' ? _args.level : undefined;
            const search = typeof _args.search === 'string' ? _args.search : undefined;

            const result = await flareupClient.getLogs({ limit, level, search });
            if (result.success && result.data) {
                if (result.data.length === 0) {
                    return 'No log entries found.';
                }
                const logList = result.data.map(entry => {
                    const fields = entry.fields ? ` ${JSON.stringify(entry.fields)}` : '';
                    return `[${entry.timestamp}] ${entry.level.toUpperCase()} ${entry.target}: ${entry.message}${fields}`;
                }).join('\n');
                return `Found ${result.data.length} log entries:\n${logList}`;
            }
            return `Error getting logs: ${result.error}`;
        }

        case 'flareup_clear_logs': {
            const result = await flareupClient.clearLogs();
            if (result.success) {
                return 'Log buffer cleared successfully.';
            }
            return `Error clearing logs: ${result.error}`;
        }

        case 'flareup_get_log_config': {
            const result = await flareupClient.getLogConfig();
            if (result.success && result.data) {
                return `Current log capture level: ${result.data.level}\nAvailable levels: ${result.data.available_levels.join(', ')}`;
            }
            return `Error getting log config: ${result.error}`;
        }

        case 'flareup_set_log_level': {
            const level = typeof _args.level === 'string' ? _args.level : 'info';
            const result = await flareupClient.setLogConfig(level);
            if (result.success && result.data) {
                return `Log capture level set to: ${result.data.level}`;
            }
            return `Error setting log level: ${result.error}`;
        }

        case 'flareup_get_jump_mode': {
            const result = await flareupClient.getJumpMode();
            if (result.success && result.data) {
                return `Jump Mode Status:
- Enabled: ${result.data.enabled}
- fd Available: ${result.data.fd_available}
- Editor: ${result.data.editor_command}
- Max Results: ${result.data.max_results}
- Search Hidden Files: ${result.data.search_hidden}`;
            }
            return `Error getting jump_mode: ${result.error}`;
        }

        default:
            return `Unknown tool: ${name}`;
    }
}

// Main server setup
async function main() {
    const server = new Server(
        {
            name: 'flareup-mcp-server',
            version: '0.1.0',
        },
        {
            capabilities: {
                tools: {},
            },
        }
    );

    // Handle tool listing
    server.setRequestHandler(ListToolsRequestSchema, async () => ({
        tools: TOOLS,
    }));

    // Handle tool execution
    server.setRequestHandler(CallToolRequestSchema, async (request) => {
        const { name, arguments: args = {} } = request.params;

        try {
            const result = await executeTool(name, args as Record<string, unknown>);
            return {
                content: [{ type: 'text', text: result }],
            };
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            return {
                content: [{ type: 'text', text: `Error executing ${name}: ${errorMessage}` }],
                isError: true,
            };
        }
    });

    // Start server with stdio transport
    const transport = new StdioServerTransport();
    await server.connect(transport);

    // Log to stderr so it doesn't interfere with MCP protocol on stdout
    console.error('Flareup MCP server started');
}

main().catch((error) => {
    console.error('Fatal error:', error);
    process.exit(1);
});
