<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { viewManager } from '$lib/viewManager.svelte';

	let searchQuery = $state('');
	let results = $state<string[]>([]);
	let selectedIndex = $state(0);
	let isSearching = $state(false);
	let error = $state<string | null>(null);
	let isFdAvailable = $state(true);
	let searchInput: HTMLInputElement;
	let debounceTimer: number;

	onMount(async () => {
		// Focus the input when component mounts
		searchInput?.focus();

		// Check if fd is available
		try {
			isFdAvailable = await invoke<boolean>('jump_mode_is_available');
			if (!isFdAvailable) {
				error = 'fd command not found. Please install fd-find to use this feature.';
			}
		} catch (e) {
			console.error('Failed to check fd availability:', e);
			error = 'Failed to check if file search is available';
		}
	});

	// Debounced search function
	function handleSearchInput(value: string) {
		searchQuery = value;
		selectedIndex = 0;

		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		if (!value.trim()) {
			results = [];
			error = null;
			return;
		}

		debounceTimer = setTimeout(async () => {
			await performSearch(value);
		}, 200) as unknown as number;
	}

	async function performSearch(query: string) {
		if (!isFdAvailable || !query.trim()) {
			return;
		}

		isSearching = true;
		error = null;

		try {
			const searchResults = await invoke<string[]>('jump_mode_search', { query });
			results = searchResults;

			if (searchResults.length === 0) {
				error = 'No files found';
			}
		} catch (e) {
			error = e as string;
			results = [];
		} finally {
			isSearching = false;
		}
	}

	async function openFile(path: string) {
		try {
			await invoke('jump_mode_open', { path });
			// Close the jump mode view after opening file
			viewManager.showCommandPalette();
		} catch (e) {
			error = `Failed to open file: ${e}`;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		switch (e.key) {
			case 'Escape':
				viewManager.showCommandPalette();
				break;

			case 'ArrowDown':
				e.preventDefault();
				if (results.length > 0) {
					selectedIndex = (selectedIndex + 1) % results.length;
					scrollToSelected();
				}
				break;

			case 'ArrowUp':
				e.preventDefault();
				if (results.length > 0) {
					selectedIndex = selectedIndex === 0 ? results.length - 1 : selectedIndex - 1;
					scrollToSelected();
				}
				break;

			case 'Enter':
				e.preventDefault();
				if (results.length > 0 && selectedIndex < results.length) {
					openFile(results[selectedIndex]);
				}
				break;
		}
	}

	function scrollToSelected() {
		const selectedElement = document.querySelector('.result-item.selected');
		selectedElement?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
	}

	// Highlight matching characters in path
	function highlightMatch(path: string, query: string): string {
		if (!query) return path;

		const queryChars = query.toLowerCase().split('');
		const pathLower = path.toLowerCase();
		let result = '';
		let queryIndex = 0;

		for (let i = 0; i < path.length; i++) {
			if (queryIndex < queryChars.length && pathLower[i] === queryChars[queryIndex]) {
				result += `<span class="highlight">${path[i]}</span>`;
				queryIndex++;
			} else {
				result += path[i];
			}
		}

		return result;
	}
</script>

<div class="jump-mode-container" role="dialog" aria-label="Jump to file">
	<div class="search-header">
		<div class="search-icon">🔍</div>
		<input
			bind:this={searchInput}
			type="text"
			class="search-input"
			placeholder="Jump to file... (type path fragments like 'src lib com')"
			value={searchQuery}
			oninput={(e) => handleSearchInput(e.currentTarget.value)}
			onkeydown={handleKeydown}
			aria-label="File search input"
		/>
		{#if isSearching}
			<div class="loading-spinner" aria-label="Searching...">⏳</div>
		{/if}
	</div>

	<div class="results-container" role="listbox">
		{#if error}
			<div class="error-message" role="alert">
				<span class="error-icon">⚠️</span>
				{error}
			</div>
		{:else if !searchQuery.trim()}
			<div class="empty-state">
				<div class="empty-icon">📂</div>
				<div class="empty-text">Start typing to search for files...</div>
				<div class="empty-hint">
					Tip: Type fragments of the path. For example, "src lib com" matches "src/lib/components"
				</div>
			</div>
		{:else if results.length === 0 && !isSearching}
			<div class="empty-state">
				<div class="empty-icon">🔍</div>
				<div class="empty-text">No files found matching "{searchQuery}"</div>
			</div>
		{:else}
			<div class="results-list">
				{#each results as result, index}
					<button
						class="result-item {index === selectedIndex ? 'selected' : ''}"
						onclick={() => openFile(result)}
						onmouseenter={() => (selectedIndex = index)}
						role="option"
						aria-selected={index === selectedIndex}
					>
						<span class="file-icon">📄</span>
						<span class="file-path">
							{@html highlightMatch(result, searchQuery)}
						</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<div class="footer">
		<div class="footer-shortcuts">
			<kbd>↑↓</kbd> Navigate
			<kbd>Enter</kbd> Open
			<kbd>Esc</kbd> Close
		</div>
		<div class="footer-info">
			{#if results.length > 0}
				{results.length} file{results.length === 1 ? '' : 's'} found
			{/if}
		</div>
	</div>
</div>

<style>
	.jump-mode-container {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--background);
		color: var(--foreground);
	}

	.search-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 1.5rem;
		border-bottom: 1px solid var(--border);
		background: var(--background-elevated);
	}

	.search-icon {
		font-size: 1.5rem;
		opacity: 0.6;
	}

	.search-input {
		flex: 1;
		padding: 0.75rem 1rem;
		font-size: 1.1rem;
		background: var(--input-background);
		border: 2px solid var(--border);
		border-radius: 8px;
		color: var(--foreground);
		transition: border-color 0.2s;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--accent);
	}

	.search-input::placeholder {
		color: var(--foreground-muted);
		opacity: 0.6;
	}

	.loading-spinner {
		font-size: 1.2rem;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.results-container {
		flex: 1;
		overflow-y: auto;
		padding: 0.5rem;
	}

	.results-list {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.result-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1rem;
		background: transparent;
		border: none;
		border-radius: 6px;
		color: var(--foreground);
		cursor: pointer;
		transition: all 0.15s;
		text-align: left;
		width: 100%;
	}

	.result-item:hover,
	.result-item.selected {
		background: var(--accent-transparent);
	}

	.result-item.selected {
		border-left: 3px solid var(--accent);
	}

	.file-icon {
		font-size: 1.2rem;
		flex-shrink: 0;
	}

	.file-path {
		flex: 1;
		font-family: 'Courier New', monospace;
		font-size: 0.95rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.file-path :global(.highlight) {
		color: var(--accent);
		font-weight: 600;
		background: var(--accent-transparent);
		padding: 0 2px;
		border-radius: 2px;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: 3rem;
		text-align: center;
		opacity: 0.7;
	}

	.empty-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
	}

	.empty-text {
		font-size: 1.2rem;
		margin-bottom: 0.5rem;
	}

	.empty-hint {
		font-size: 0.9rem;
		color: var(--foreground-muted);
		max-width: 500px;
	}

	.error-message {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 1rem;
		margin: 1rem;
		background: var(--error-background, rgba(220, 38, 38, 0.1));
		border: 1px solid var(--error-border, rgba(220, 38, 38, 0.3));
		border-radius: 8px;
		color: var(--error-foreground, #ef4444);
	}

	.error-icon {
		font-size: 1.5rem;
	}

	.footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1.5rem;
		border-top: 1px solid var(--border);
		background: var(--background-elevated);
		font-size: 0.85rem;
		color: var(--foreground-muted);
	}

	.footer-shortcuts {
		display: flex;
		gap: 1rem;
	}

	kbd {
		padding: 0.2rem 0.5rem;
		background: var(--kbd-background, rgba(255, 255, 255, 0.1));
		border: 1px solid var(--border);
		border-radius: 4px;
		font-family: monospace;
		font-size: 0.8rem;
		margin-left: 0.25rem;
	}

	/* Scrollbar styling */
	.results-container::-webkit-scrollbar {
		width: 8px;
	}

	.results-container::-webkit-scrollbar-track {
		background: transparent;
	}

	.results-container::-webkit-scrollbar-thumb {
		background: var(--scrollbar-thumb, rgba(255, 255, 255, 0.2));
		border-radius: 4px;
	}

	.results-container::-webkit-scrollbar-thumb:hover {
		background: var(--scrollbar-thumb-hover, rgba(255, 255, 255, 0.3));
	}
</style>
