/**
 * HTTP client for communicating with Flareup's debug API
 */

const FLAREUP_API_URL = 'http://127.0.0.1:7266';

export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
}

export interface HealthResponse {
    status: string;
    version: string;
    uptime_seconds: number;
}

export interface App {
    name: string;
    exec: string;
    icon?: string;
    comment?: string;
    desktop_file_path?: string;
}

export interface PluginInfo {
    name: string;
    title?: string;
    description?: string;
    path: string;
    icon?: string;
    enabled: boolean;
}

export interface Snippet {
    id: number;
    name: string;
    keyword: string;
    content: string;
    use_count: number;
    created_at: string;
    updated_at: string;
}

export interface Quicklink {
    id: number;
    name: string;
    link: string;
    application?: string;
    icon?: string;
    createdAt: string;
    updatedAt: string;
}

export interface FrecencyData {
    item_id: string;
    total_score: number;
    use_count: number;
    last_used: number;
}

export interface LogEntry {
    timestamp: string;
    level: string;
    target: string;
    message: string;
    fields?: Record<string, unknown>;
}

export interface LogConfig {
    level: string;
    available_levels: string[];
}

export class FlareupClient {
    private baseUrl: string;

    constructor(baseUrl: string = FLAREUP_API_URL) {
        this.baseUrl = baseUrl;
    }

    private async fetch<T>(endpoint: string): Promise<ApiResponse<T>> {
        try {
            const response = await fetch(`${this.baseUrl}${endpoint}`, {
                headers: { 'Accept': 'application/json' },
            });

            if (!response.ok) {
                return {
                    success: false,
                    error: `HTTP ${response.status}: ${response.statusText}`
                };
            }

            return await response.json() as ApiResponse<T>;
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }

    async health(): Promise<ApiResponse<HealthResponse>> {
        try {
            const response = await fetch(`${this.baseUrl}/health`);
            const data = await response.json() as HealthResponse;
            return { success: true, data };
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Connection failed'
            };
        }
    }

    async listApps(): Promise<ApiResponse<App[]>> {
        return this.fetch<App[]>('/apps');
    }

    async listPlugins(): Promise<ApiResponse<PluginInfo[]>> {
        return this.fetch<PluginInfo[]>('/plugins');
    }

    async listSnippets(): Promise<ApiResponse<Snippet[]>> {
        return this.fetch<Snippet[]>('/snippets');
    }

    async listQuicklinks(): Promise<ApiResponse<Quicklink[]>> {
        return this.fetch<Quicklink[]>('/quicklinks');
    }

    async getFrecency(): Promise<ApiResponse<FrecencyData[]>> {
        return this.fetch<FrecencyData[]>('/frecency');
    }

    async getAliases(): Promise<ApiResponse<Record<string, string>>> {
        return this.fetch<Record<string, string>>('/aliases');
    }

    async getSettings(): Promise<ApiResponse<Record<string, unknown>>> {
        return this.fetch<Record<string, unknown>>('/settings');
    }

    async getAiSettings(): Promise<ApiResponse<Record<string, unknown>>> {
        return this.fetch<Record<string, unknown>>('/ai/settings');
    }

    async getJumpMode(): Promise<ApiResponse<JumpModeData>> {
        return this.fetch<JumpModeData>('/jump-mode');
    }

    async getLogs(options?: { limit?: number; level?: string; search?: string }): Promise<ApiResponse<LogEntry[]>> {
        const params = new URLSearchParams();
        if (options?.limit) params.set('limit', options.limit.toString());
        if (options?.level) params.set('level', options.level);
        if (options?.search) params.set('search', options.search);

        const queryString = params.toString();
        const endpoint = queryString ? `/logs?${queryString}` : '/logs';
        return this.fetch<LogEntry[]>(endpoint);
    }

    async clearLogs(): Promise<ApiResponse<void>> {
        try {
            const response = await fetch(`${this.baseUrl}/logs`, {
                method: 'DELETE',
                headers: { 'Accept': 'application/json' },
            });

            if (!response.ok) {
                return {
                    success: false,
                    error: `HTTP ${response.status}: ${response.statusText}`
                };
            }

            return await response.json() as ApiResponse<void>;
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }

    async getLogConfig(): Promise<ApiResponse<LogConfig>> {
        return this.fetch<LogConfig>('/logs/config');
    }

    async setLogConfig(level: string): Promise<ApiResponse<LogConfig>> {
        try {
            const response = await fetch(`${this.baseUrl}/logs/config`, {
                method: 'POST',
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ level }),
            });

            if (!response.ok) {
                return {
                    success: false,
                    error: `HTTP ${response.status}: ${response.statusText}`
                };
            }

            return await response.json() as ApiResponse<LogConfig>;
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }
}

export interface JumpModeData {
    enabled: boolean;
    fd_available: boolean;
    editor_command: string;
    max_results: number;
    search_hidden: boolean;
}

// Singleton instance
export const flareupClient = new FlareupClient();
