import { invoke } from '@tauri-apps/api/core';

class AliasesStore {
	aliases = $state<Record<string, string>>({});
	isLoaded = $state(false);

	constructor() {
		this.loadAliases();
	}

	async loadAliases() {
		try {
			const result = await invoke<Record<string, string>>('get_aliases');
			console.log('[AliasesStore] Loaded aliases from backend:', result);
			this.aliases = result;
			this.isLoaded = true;
		} catch (error) {
			console.error('[AliasesStore] Failed to load aliases:', error);
			this.isLoaded = true; // Mark as loaded even on error so we don't block
		}
	}

	async setAlias(alias: string, commandId: string) {
		console.log('[AliasesStore] Setting alias:', alias, '->', commandId);
		try {
			await invoke('set_alias', { alias, commandId });
			this.aliases = { ...this.aliases, [alias]: commandId };
			console.log('[AliasesStore] Alias set successfully. Current aliases:', this.aliases);
		} catch (error) {
			console.error('[AliasesStore] Failed to set alias:', error);
			throw error;
		}
	}

	async removeAlias(alias: string) {
		console.log('[AliasesStore] Removing alias:', alias);
		try {
			await invoke('remove_alias', { alias });
			const newAliases = { ...this.aliases };
			delete newAliases[alias];
			this.aliases = newAliases;
			console.log('[AliasesStore] Alias removed. Current aliases:', this.aliases);
		} catch (error) {
			console.error('[AliasesStore] Failed to remove alias:', error);
			throw error;
		}
	}

	getCommandId(alias: string): string | undefined {
		const result = this.aliases?.[alias];
		console.log('[AliasesStore] getCommandId for', alias, '->', result);
		return result;
	}
}

export const aliasesStore = new AliasesStore();
