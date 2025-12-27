import type { PluginInfo } from '@flare/protocol';
import { invoke } from '@tauri-apps/api/core';
import Fuse from 'fuse.js';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { Quicklink } from '$lib/quicklinks.svelte';
import { frecencyStore } from './frecency.svelte';
import { viewManager } from './viewManager.svelte';
import type { App } from './apps.svelte';
import { aliasesStore } from './aliases.svelte';

export type UnifiedItem = {
	type: 'calculator' | 'plugin' | 'app' | 'quicklink' | 'ai-command';
	id: string;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Union type with discriminated access
	data: any;
	score: number;
	alias?: string;
};

export type AiCommand = {
	id: string;
	name: string;
	icon: string | null;
	promptTemplate: string;
	model: string | null;
	creativity: string | null;
	outputAction: string;
	hotkey: string | null;
	createdAt: number;
	updatedAt: number;
};

type UseCommandPaletteItemsArgs = {
	searchText: () => string;
	plugins: () => PluginInfo[];
	installedApps: () => App[];
	quicklinks: () => Quicklink[];
	frecencyData: () => { itemId: string; useCount: number; lastUsedAt: number }[];
	selectedQuicklinkForArgument: () => Quicklink | null;
};

let cachedAiCommands: AiCommand[] = [];
let aiCommandsLoaded = false;

async function loadAiCommands(): Promise<AiCommand[]> {
	if (aiCommandsLoaded) return cachedAiCommands;
	try {
		cachedAiCommands = await invoke<AiCommand[]>('list_ai_commands');
		aiCommandsLoaded = true;
	} catch (error) {
		console.error('Failed to load AI commands:', error);
		cachedAiCommands = [];
	}
	return cachedAiCommands;
}

// Refresh AI commands cache
export function refreshAiCommandsCache() {
	aiCommandsLoaded = false;
}

export function useCommandPaletteItems({
	searchText,
	plugins,
	installedApps,
	quicklinks,
	frecencyData,
	selectedQuicklinkForArgument
}: UseCommandPaletteItemsArgs) {
	const allSearchableItems = $derived.by(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Union type with discriminated access
		const items: { type: 'plugin' | 'app' | 'quicklink' | 'ai-command'; id: string; data: any }[] =
			[];
		items.push(...plugins().map((p) => ({ type: 'plugin', id: p.pluginPath, data: p }) as const));
		items.push(...installedApps().map((a) => ({ type: 'app', id: a.exec, data: a }) as const));
		items.push(
			...quicklinks().map((q) => ({ type: 'quicklink', id: `quicklink-${q.id}`, data: q }) as const)
		);
		// Add AI commands from cache
		items.push(
			...cachedAiCommands.map(
				(c) => ({ type: 'ai-command', id: `ai-command-${c.id}`, data: c }) as const
			)
		);
		return items;
	});

	// Load AI commands on first render
	$effect(() => {
		loadAiCommands();
	});

	const fuse = $derived(
		new Fuse(allSearchableItems, {
			keys: [
				'data.title',
				'data.pluginTitle',
				'data.description',
				'data.name',
				'data.comment',
				'data.link',
				'data.promptTemplate'
			],
			threshold: 0.4,
			includeScore: true
		})
	);

	let calculatorResult = $state<{ value: string; type: string } | null>(null);
	let calculationId = 0;

	$effect(() => {
		const term = searchText();
		calculationId++;
		const currentCalculationId = calculationId;

		if (!term.trim() || selectedQuicklinkForArgument()) {
			calculatorResult = null;
			return;
		}

		(async () => {
			try {
				const resultJson = await invoke<string>('calculate_soulver', { expression: term.trim() });

				if (currentCalculationId !== calculationId) {
					return; // Stale request
				}

				const result = JSON.parse(resultJson) as {
					value: string;
					type: string;
					error?: string;
				};

				if (result.error) {
					console.error('Soulver error:', result.error);
					calculatorResult = null;
					return;
				}

				if (result.type === 'none' || !result.value) {
					calculatorResult = null;
					return;
				}

				if (result.value === term.trim()) {
					calculatorResult = null;
					return;
				}

				calculatorResult = { value: result.value, type: result.type };
			} catch (e) {
				if (currentCalculationId !== calculationId) {
					return; // Stale request
				}
				console.error('Soulver invocation failed:', e);
				calculatorResult = null;
			}
		})();
	});

	const displayItems = $derived.by(() => {
		let items: (UnifiedItem & { fuseScore?: number; alias?: string })[] = [];
		const term = searchText();
		const aliases = aliasesStore.aliases;

		// Debug: log current aliases state (stringify to see actual content)
		if (term.trim()) {
			console.log(
				'[CommandPalette] Searching with term:',
				term,
				'Available aliases:',
				JSON.stringify(aliases)
			);
		}

		// Build reverse lookup: command_id -> alias
		const commandToAlias = new Map<string, string>();
		for (const [alias, commandId] of Object.entries(aliases)) {
			commandToAlias.set(commandId, alias);
		}

		if (term.trim()) {
			items = fuse.search(term).map((result) => ({
				...result.item,
				score: 0,
				fuseScore: result.score,
				alias: commandToAlias.get(result.item.id)
			}));

			// Check if search term matches an alias exactly or partially
			const termLower = term.trim().toLowerCase();
			const aliasMatch = Object.entries(aliases).find(
				([alias]) => alias.toLowerCase() === termLower || alias.toLowerCase().startsWith(termLower)
			);

			console.log('[CommandPalette] Alias match for term:', termLower, '->', aliasMatch);

			if (aliasMatch) {
				const [matchedAlias, commandId] = aliasMatch;
				// Find the item in allSearchableItems by command ID
				console.log('[CommandPalette] Looking for item with ID:', commandId);
				console.log(
					'[CommandPalette] Available item IDs:',
					allSearchableItems.map((i) => i.id).slice(0, 10),
					'...'
				);
				const matchedItem = allSearchableItems.find((item) => item.id === commandId);
				console.log('[CommandPalette] Matched item:', matchedItem);
				if (matchedItem) {
					// Remove if already in results (to avoid duplicates)
					items = items.filter((item) => item.id !== commandId);
					// Add at the beginning with high score
					items.unshift({
						...matchedItem,
						score: 10000, // Very high score to appear first
						fuseScore: 0,
						alias: matchedAlias
					});
				}
			}
		} else {
			// No search term - show all items with their aliases
			console.log('[CommandPalette] No search term. commandToAlias map:', [
				...commandToAlias.entries()
			]);
			items = allSearchableItems.map((item) => ({
				...item,
				score: 0,
				fuseScore: 1,
				alias: commandToAlias.get(item.id)
			}));
		}

		const frecencyMap = new Map(frecencyData().map((item) => [item.itemId, item]));
		const gravity = 1.8;

		items.forEach((item) => {
			const frecency = frecencyMap.get(item.id);
			let frecencyScore = 0;
			if (frecency) {
				// Backend stores timestamp in nanoseconds, convert to seconds
				const lastUsedSeconds = frecency.lastUsedAt / 1_000_000_000;
				const nowSeconds = Date.now() / 1000;
				const ageInHours = Math.max(1, (nowSeconds - lastUsedSeconds) / 3600);
				frecencyScore = (frecency.useCount * 1000) / Math.pow(ageInHours + 2, gravity);
			}
			const textScore = item.fuseScore !== undefined ? 1 - item.fuseScore * 100 : 0;
			// Only add frecency/text score if not already boosted by alias match
			if (item.score < 10000) {
				item.score = frecencyScore + textScore;
			}
		});

		items.sort((a, b) => b.score - a.score);

		const calcRes = calculatorResult;
		if (calcRes) {
			items.unshift({
				type: 'calculator',
				id: 'calculator',
				data: {
					value: term,
					result: calcRes.value,
					resultType: calcRes.type
				},
				score: 9999
			});
		}

		// Deduplicate by ID, but prefer items that have an alias attached
		const seenIds = new Map<string, (typeof items)[0]>();
		for (const item of items) {
			const existing = seenIds.get(item.id);
			if (!existing) {
				seenIds.set(item.id, item);
			} else if (item.alias && !existing.alias) {
				// Prefer the item with alias
				seenIds.set(item.id, item);
			}
			// Otherwise keep the existing (first seen)
		}

		const result = [...seenIds.values()];

		// Debug: log items with aliases
		const itemsWithAlias = result.filter((i) => i.alias);
		if (itemsWithAlias.length > 0) {
			console.log(
				'[CommandPalette] Items with aliases:',
				itemsWithAlias.map((i) => ({ id: i.id, alias: i.alias, type: i.type }))
			);
		}

		return result;
	});

	return () => ({
		displayItems
	});
}

type UseCommandPaletteActionsArgs = {
	selectedItem: () => UnifiedItem | undefined;
	onRunPlugin: (plugin: PluginInfo) => void;
	onExecuteAiCommand: (command: AiCommand) => void;
	resetState: () => void;
	focusArgumentInput: () => void;
};

export function useCommandPaletteActions({
	selectedItem,
	onRunPlugin,
	onExecuteAiCommand,
	resetState,
	focusArgumentInput
}: UseCommandPaletteActionsArgs) {
	async function executeQuicklink(quicklink: Quicklink, argument?: string) {
		const finalLink = argument
			? quicklink.link.replace(/\{argument\}/g, encodeURIComponent(argument))
			: quicklink.link.replace(/\{argument\}/g, '');
		await invoke('execute_quicklink', {
			link: finalLink,
			application: quicklink.application
		});
		resetState();
	}

	async function handleEnter() {
		const item = selectedItem();
		if (!item) return;

		await frecencyStore.recordUsage(item.id);

		switch (item.type) {
			case 'calculator': {
				writeText(item.data.result);
				break;
			}
			case 'plugin': {
				onRunPlugin(item.data as PluginInfo);
				break;
			}
			case 'app': {
				if (item.data.exec) {
					invoke('launch_app', { exec: item.data.exec }).catch(console.error);
				}
				break;
			}
			case 'quicklink': {
				const quicklink = item.data as Quicklink;
				if (quicklink.link.includes('{argument}')) {
					focusArgumentInput();
				} else {
					executeQuicklink(quicklink);
				}
				break;
			}
			case 'ai-command': {
				onExecuteAiCommand(item.data as AiCommand);
				break;
			}
		}
	}

	async function handleResetRanking() {
		const item = selectedItem();
		if (item) {
			await frecencyStore.deleteEntry(item.id);
		}
	}

	function handleCopyDeeplink() {
		const item = selectedItem();
		if (item?.type !== 'plugin') return;
		const plugin = item.data as PluginInfo;
		const authorOrOwner =
			plugin.owner === 'raycast'
				? 'raycast'
				: typeof plugin.author === 'string'
					? plugin.author
					: (plugin.author?.name ?? 'unknown');

		const deeplink = `raycast://extensions/${authorOrOwner}/${plugin.pluginName}/${plugin.commandName}`;
		writeText(deeplink);
	}

	function handleConfigureCommand() {
		const item = selectedItem();
		if (item?.type !== 'plugin') return;
		viewManager.showSettings(item.data.pluginName);
	}

	function handleCopyAppName() {
		const item = selectedItem();
		if (item?.type !== 'app') return;
		writeText(item.data.name);
	}

	function handleCopyAppPath() {
		const item = selectedItem();
		if (item?.type !== 'app') return;
		writeText(item.data.exec);
	}

	async function handleHideApp() {
		const item = selectedItem();
		if (item?.type !== 'app') return;
		await frecencyStore.hideItem(item.id);
	}

	async function handleSetAlias(alias: string) {
		const item = selectedItem();
		if (!item) return;
		await aliasesStore.setAlias(alias, item.id);
	}

	return {
		executeQuicklink,
		handleEnter,
		handleResetRanking,
		handleCopyDeeplink,
		handleConfigureCommand,
		handleCopyAppName,
		handleCopyAppPath,
		handleHideApp,
		handleSetAlias
	};
}
