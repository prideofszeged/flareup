<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { viewManager } from '$lib/viewManager.svelte';
	import { onMount } from 'svelte';

	interface WindowInfo {
		pid: number;
		windowId: string;
		title: string;
		className: string;
	}

	let windows = $state<WindowInfo[]>([]);
	let filteredWindows = $state<WindowInfo[]>([]);
	let searchQuery = $state('');
	let selectedIndex = $state(0);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	async function loadWindows() {
		isLoading = true;
		error = null;
		try {
			windows = await invoke<WindowInfo[]>('list_windows');
			filterWindows();
		} catch (e) {
			error = String(e);
			windows = [];
			filteredWindows = [];
		} finally {
			isLoading = false;
		}
	}

	function filterWindows() {
		const query = searchQuery.toLowerCase();
		if (!query) {
			filteredWindows = windows;
		} else {
			filteredWindows = windows.filter(
				(w) => w.title.toLowerCase().includes(query) || w.className.toLowerCase().includes(query)
			);
		}
		selectedIndex = 0;
	}

	async function killWindow(window: WindowInfo, force: boolean = false) {
		try {
			await invoke('process_kill', {
				pid: window.pid,
				signal: force ? 'hard' : 'soft'
			});
			await invoke('show_hud', { title: `Killed: ${window.title}` });
			// Go back to command palette after killing
			viewManager.showCommandPalette();
		} catch (e) {
			error = String(e);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		switch (event.key) {
			case 'Escape':
				viewManager.showCommandPalette();
				break;
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, filteredWindows.length - 1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				event.preventDefault();
				if (filteredWindows[selectedIndex]) {
					killWindow(filteredWindows[selectedIndex], event.shiftKey);
				}
				break;
		}
	}

	// Filter when search changes
	$effect(() => {
		const _ = searchQuery; // track dependency
		filterWindows();
	});

	onMount(() => {
		loadWindows();
	});
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="quick-kill">
	<header class="header">
		<h1>🔴 Quick Kill</h1>
		<span class="subtitle">Force quit applications</span>
	</header>

	<div class="search-bar">
		<input type="text" placeholder="Search applications..." bind:value={searchQuery} />
	</div>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	{#if isLoading}
		<div class="loading">Loading applications...</div>
	{:else if filteredWindows.length === 0}
		<div class="empty">No applications found</div>
	{:else}
		<div class="window-list" role="list">
			{#each filteredWindows as window, idx}
				<div
					class="window-item"
					class:selected={idx === selectedIndex}
					role="button"
					tabindex="0"
					onclick={() => killWindow(window)}
					onkeydown={(event) => {
						if (event.key === 'Enter' || event.key === ' ') {
							event.preventDefault();
							killWindow(window);
						}
					}}
				>
					<div class="window-info">
						<span class="window-title">{window.title}</span>
						{#if window.className}
							<span class="window-class">{window.className}</span>
						{/if}
					</div>
					<div class="window-actions">
						<button
							class="kill-btn"
							onclick={(e) => {
								e.stopPropagation();
								killWindow(window, false);
							}}
							title="SIGTERM - Graceful quit"
						>
							Quit
						</button>
						<button
							class="force-btn"
							onclick={(e) => {
								e.stopPropagation();
								killWindow(window, true);
							}}
							title="SIGKILL - Force kill"
						>
							Force
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	<footer class="footer">
		<span class="hint">↑↓ Navigate • Enter: Quit • Shift+Enter: Force Kill • Esc: Cancel</span>
	</footer>
</div>

<style>
	.quick-kill {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--bg-primary, #1a1a2e);
		color: var(--text-primary, #eee);
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
	}

	.header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 1rem;
		border-bottom: 1px solid var(--border-color, #333);
	}

	.header h1 {
		font-size: 1.1rem;
		font-weight: 600;
		margin: 0;
	}

	.subtitle {
		color: var(--text-secondary, #888);
		font-size: 0.85rem;
	}

	.search-bar {
		padding: 0.75rem 1rem;
	}

	.search-bar input {
		width: 100%;
		padding: 0.65rem 1rem;
		border: 1px solid var(--border-color, #333);
		border-radius: 8px;
		background: var(--bg-secondary, #252542);
		color: var(--text-primary, #eee);
		font-size: 1rem;
	}

	.search-bar input:focus {
		outline: none;
		border-color: var(--danger-color, #ef4444);
		box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.2);
	}

	.error {
		padding: 0.75rem 1rem;
		background: #ff4444;
		color: white;
		margin: 0.5rem 1rem;
		border-radius: 6px;
	}

	.loading,
	.empty {
		padding: 2rem;
		text-align: center;
		color: var(--text-secondary, #aaa);
	}

	.window-list {
		flex: 1;
		overflow-y: auto;
		list-style: none;
		margin: 0;
		padding: 0.5rem;
	}

	.window-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		border-radius: 8px;
		cursor: pointer;
		transition: background 0.1s;
	}

	.window-item:hover {
		background: var(--bg-hover, #2a2a4a);
	}

	.window-item.selected {
		background: var(--danger-color, #ef4444);
	}

	.window-info {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		min-width: 0;
		flex: 1;
	}

	.window-title {
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.window-class {
		font-size: 0.75rem;
		color: var(--text-secondary, #888);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.window-item.selected .window-class {
		color: rgba(255, 255, 255, 0.7);
	}

	.window-actions {
		display: flex;
		gap: 0.5rem;
		margin-left: 1rem;
	}

	.kill-btn,
	.force-btn {
		padding: 0.35rem 0.75rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.8rem;
		font-weight: 500;
	}

	.kill-btn {
		background: var(--warning-color, #f59e0b);
		color: black;
	}

	.force-btn {
		background: var(--danger-color, #ef4444);
		color: white;
	}

	.kill-btn:hover,
	.force-btn:hover {
		filter: brightness(1.1);
	}

	.footer {
		padding: 0.5rem 1rem;
		border-top: 1px solid var(--border-color, #333);
		background: var(--bg-secondary, #252542);
	}

	.hint {
		font-size: 0.75rem;
		color: var(--text-secondary, #888);
	}
</style>
