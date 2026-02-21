<script lang="ts">
	import { aliasesStore } from '$lib/aliases.svelte';
	import { onMount } from 'svelte';
	import Icon from '$lib/components/Icon.svelte';

	let editingAlias: string | null = $state(null);
	let editValue = $state('');

	// Get aliases as array for display
	const aliasEntries = $derived(
		Object.entries(aliasesStore.aliases).map(([alias, commandId]) => ({
			alias,
			commandId,
			// Extract display name from commandId
			displayName: getDisplayName(commandId)
		}))
	);

	function getDisplayName(commandId: string): string {
		// Try to extract a readable name from the command ID
		if (commandId.startsWith('ai-command-')) {
			return `AI Command: ${commandId.replace('ai-command-', '')}`;
		}
		if (commandId.startsWith('quicklink-')) {
			return `Quicklink: ${commandId.replace('quicklink-', '')}`;
		}
		// For apps, the ID is the exec path - get the last part
		const parts = commandId.split('/');
		return parts[parts.length - 1] || commandId;
	}

	function startEditing(alias: string) {
		editingAlias = alias;
		editValue = alias;
	}

	async function saveEdit(oldAlias: string, commandId: string) {
		if (!editValue.trim() || editValue === oldAlias) {
			cancelEdit();
			return;
		}

		try {
			// Remove old alias and set new one
			await aliasesStore.removeAlias(oldAlias);
			await aliasesStore.setAlias(editValue.trim(), commandId);
			cancelEdit();
		} catch (e) {
			console.error('Failed to update alias:', e);
		}
	}

	function cancelEdit() {
		editingAlias = null;
		editValue = '';
	}

	async function removeAlias(alias: string) {
		try {
			await aliasesStore.removeAlias(alias);
		} catch (e) {
			console.error('Failed to remove alias:', e);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!editingAlias) return;

		if (event.key === 'Enter') {
			event.preventDefault();
			const entry = aliasEntries.find((e) => e.alias === editingAlias);
			if (entry) {
				saveEdit(editingAlias, entry.commandId);
			}
		} else if (event.key === 'Escape') {
			event.preventDefault();
			cancelEdit();
		}
	}

	onMount(() => {
		// Ensure aliases are loaded
		if (!aliasesStore.isLoaded) {
			aliasesStore.loadAliases();
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="aliases-settings">
	<div class="mb-6">
		<h2 class="text-foreground text-xl font-semibold">Command Aliases</h2>
		<p class="text-muted-foreground mt-1 text-sm">
			Manage keyboard shortcuts for quick command access. Type an alias in the command palette to
			jump directly to the command.
		</p>
	</div>

	{#if aliasEntries.length === 0}
		<div
			class="bg-secondary/50 border-border flex flex-col items-center justify-center rounded-lg border border-dashed p-8"
		>
			<Icon icon="command-16" class="text-muted-foreground mb-3 size-8 opacity-50" />
			<p class="text-muted-foreground text-center text-sm">
				No aliases configured yet.
				<br />
				<span class="text-xs opacity-75">
					Select a command in the palette and use "Assign Alias" to create one.
				</span>
			</p>
		</div>
	{:else}
		<div class="alias-list">
			{#each aliasEntries as entry (entry.alias)}
				<div
					class="bg-background border-border hover:border-muted-foreground flex items-center justify-between gap-4 rounded-lg border p-4 transition-all hover:shadow-md"
				>
					<div class="min-w-0 flex-1">
						<div class="text-foreground mb-1 truncate text-sm font-medium">
							{entry.displayName}
						</div>
						<div class="text-muted-foreground truncate font-mono text-xs opacity-70">
							{entry.commandId}
						</div>
					</div>

					<div class="flex items-center gap-2">
						{#if editingAlias === entry.alias}
							<input
								type="text"
								bind:value={editValue}
								class="bg-background border-primary w-24 rounded border-2 px-2 py-1 font-mono text-sm focus:outline-none"
								autofocus
							/>
							<button
								class="hover:bg-primary/90 bg-primary rounded px-3 py-1.5 text-sm text-white transition-colors"
								onclick={() => saveEdit(entry.alias, entry.commandId)}
							>
								Save
							</button>
							<button
								class="bg-secondary hover:bg-secondary/80 border-border rounded border px-3 py-1.5 text-sm transition-colors"
								onclick={cancelEdit}
							>
								Cancel
							</button>
						{:else}
							<span
								class="bg-emerald-500/20 text-emerald-400 border-emerald-500/50 rounded border px-2 py-1 font-mono text-sm font-medium"
							>
								{entry.alias}
							</span>
							<button
								class="text-muted-foreground hover:text-foreground p-1 transition-colors"
								onclick={() => startEditing(entry.alias)}
								title="Edit alias"
							>
								<Icon icon="pencil-16" class="size-4" />
							</button>
							<button
								class="text-muted-foreground hover:text-destructive p-1 transition-colors"
								onclick={() => removeAlias(entry.alias)}
								title="Remove alias"
							>
								<Icon icon="trash-16" class="size-4" />
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.aliases-settings {
		padding: 20px;
		max-width: 800px;
		margin: 0 auto;
	}

	.alias-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
</style>
