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
			this.aliases = result && typeof result === 'object' ? result : {};
			this.isLoaded = true;
		} catch (error) {
			console.error('[AliasesStore] Failed to load aliases:', error);
			this.aliases = {};
			this.isLoaded = true; // Mark as loaded even on error so we don't block
		}
	}

	async setAlias(alias: string, commandId: string) {
		try {
			await invoke('set_alias', { alias, commandId });
			this.aliases = { ...this.aliases, [alias]: commandId };
		} catch (error) {
			console.error('[AliasesStore] Failed to set alias:', error);
			throw error;
		}
	}

	async removeAlias(alias: string) {
		try {
			await invoke('remove_alias', { alias });
			const newAliases = { ...this.aliases };
			delete newAliases[alias];
			this.aliases = newAliases;
		} catch (error) {
			console.error('[AliasesStore] Failed to remove alias:', error);
			throw error;
		}
	}

	getCommandId(alias: string): string | undefined {
		return this.aliases?.[alias];
	}
}

export const aliasesStore = new AliasesStore();
