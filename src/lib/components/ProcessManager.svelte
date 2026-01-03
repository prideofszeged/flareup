<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { viewManager } from '$lib/viewManager.svelte';
	import { onMount } from 'svelte';

	interface ProcessInfo {
		pid: number;
		name: string;
		command: string;
		cpuPercent: number;
		memoryPercent: number;
		memoryMb: number;
		user: string;
	}

	interface PortInfo {
		port: number;
		protocol: string;
		pid: number | null;
		processName: string | null;
		localAddress: string;
		state: string;
	}

	type Tab = 'processes' | 'ports';
	type KillSignal = 'soft' | 'hard';

	let activeTab = $state<Tab>('processes');
	let searchQuery = $state('');
	let processes = $state<ProcessInfo[]>([]);
	let ports = $state<PortInfo[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let selectedIndex = $state(0);
	let sortColumn = $state<'pid' | 'name' | 'cpu' | 'memory'>('cpu');
	let sortAscending = $state(false);

	// Kill port modal state
	let showKillPortModal = $state(false);
	let killPortInput = $state('');

	async function loadProcesses() {
		isLoading = true;
		error = null;
		try {
			if (searchQuery.trim()) {
				processes = await invoke<ProcessInfo[]>('process_search', { query: searchQuery });
			} else {
				processes = await invoke<ProcessInfo[]>('process_list');
			}
			selectedIndex = 0;
		} catch (e) {
			error = String(e);
			processes = [];
		} finally {
			isLoading = false;
		}
	}

	async function loadPorts() {
		isLoading = true;
		error = null;
		try {
			ports = await invoke<PortInfo[]>('port_list_open');
			selectedIndex = 0;
		} catch (e) {
			error = String(e);
			ports = [];
		} finally {
			isLoading = false;
		}
	}

	async function killProcess(pid: number, signal: KillSignal) {
		const signalName = signal === 'hard' ? 'SIGKILL' : 'SIGTERM';
		const confirmed = confirm(`Are you sure you want to send ${signalName} to process ${pid}?`);
		if (!confirmed) return;

		try {
			await invoke('process_kill', { pid, signal });
			await invoke('show_hud', { title: `Process ${pid} killed` });
			// Reload after kill
			if (activeTab === 'processes') {
				await loadProcesses();
			} else {
				await loadPorts();
			}
		} catch (e) {
			error = String(e);
		}
	}

	async function killPortProcess(port: number, signal: KillSignal) {
		const signalName = signal === 'hard' ? 'SIGKILL' : 'SIGTERM';
		const confirmed = confirm(
			`Are you sure you want to send ${signalName} to the process on port ${port}?`
		);
		if (!confirmed) return;

		try {
			await invoke('port_kill_process', { port, signal });
			await invoke('show_hud', { title: `Process on port ${port} killed` });
			await loadPorts();
		} catch (e) {
			error = String(e);
		}
	}

	async function handleQuickKillPort() {
		const port = parseInt(killPortInput.trim(), 10);
		if (isNaN(port) || port < 1 || port > 65535) {
			error = 'Invalid port number (1-65535)';
			return;
		}

		showKillPortModal = false;
		try {
			await invoke('port_kill_process', { port, signal: 'soft' });
			await invoke('show_hud', { title: `Process on port ${port} killed` });
			killPortInput = '';
			await loadPorts();
		} catch (e) {
			error = String(e);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		const items = activeTab === 'processes' ? sortedProcesses : ports;
		const maxIndex = items.length - 1;

		switch (event.key) {
			case 'Escape':
				viewManager.showCommandPalette();
				break;
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, maxIndex);
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Tab':
				event.preventDefault();
				activeTab = activeTab === 'processes' ? 'ports' : 'processes';
				selectedIndex = 0;
				break;
			case 'Enter':
				event.preventDefault();
				if (items[selectedIndex]) {
					if (activeTab === 'processes') {
						killProcess((items[selectedIndex] as ProcessInfo).pid, 'soft');
					} else {
						const portInfo = items[selectedIndex] as PortInfo;
						if (portInfo.pid) {
							killPortProcess(portInfo.port, 'soft');
						}
					}
				}
				break;
			case 'k':
				if (event.ctrlKey) {
					showKillPortModal = true;
				}
				break;
		}
	}

	function sortBy(column: typeof sortColumn) {
		if (sortColumn === column) {
			sortAscending = !sortAscending;
		} else {
			sortColumn = column;
			sortAscending = false;
		}
	}

	const sortedProcesses = $derived.by(() => {
		const sorted = [...processes];
		sorted.sort((a, b) => {
			let cmp = 0;
			switch (sortColumn) {
				case 'pid':
					cmp = a.pid - b.pid;
					break;
				case 'name':
					cmp = a.name.localeCompare(b.name);
					break;
				case 'cpu':
					cmp = a.cpuPercent - b.cpuPercent;
					break;
				case 'memory':
					cmp = a.memoryMb - b.memoryMb;
					break;
			}
			return sortAscending ? cmp : -cmp;
		});
		return sorted;
	});

	// Debounced search - watch searchQuery changes
	let searchTimeout: ReturnType<typeof setTimeout>;
	$effect(() => {
		// Access searchQuery to establish dependency tracking
		const query = searchQuery;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			if (activeTab === 'processes') {
				loadProcesses();
			}
		}, 300);
	});

	$effect(() => {
		if (activeTab === 'ports') {
			loadPorts();
		}
	});

	onMount(() => {
		loadProcesses();
	});
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="process-manager">
	<header class="header">
		<button class="back-button" onclick={() => viewManager.showCommandPalette()}> ← Back </button>
		<h1>Process Manager</h1>
		<button
			class="kill-port-btn"
			onclick={() => (showKillPortModal = true)}
			title="Quick Kill Port (Ctrl+K)"
		>
			Kill Port
		</button>
	</header>

	<div class="tabs">
		<button
			class="tab"
			class:active={activeTab === 'processes'}
			onclick={() => (activeTab = 'processes')}
		>
			Processes
		</button>
		<button class="tab" class:active={activeTab === 'ports'} onclick={() => (activeTab = 'ports')}>
			Ports
		</button>
	</div>

	{#if activeTab === 'processes'}
		<div class="search-bar">
			<input
				type="text"
				placeholder="Search processes by name, command, or PID..."
				bind:value={searchQuery}
				autofocus
			/>
		</div>
	{/if}

	{#if error}
		<div class="error">{error}</div>
	{/if}

	{#if isLoading}
		<div class="loading">Loading...</div>
	{:else if activeTab === 'processes'}
		<div class="table-container">
			<table>
				<thead>
					<tr>
						<th class="sortable" onclick={() => sortBy('pid')}>
							PID {sortColumn === 'pid' ? (sortAscending ? '↑' : '↓') : ''}
						</th>
						<th class="sortable" onclick={() => sortBy('name')}>
							Name {sortColumn === 'name' ? (sortAscending ? '↑' : '↓') : ''}
						</th>
						<th>User</th>
						<th class="sortable" onclick={() => sortBy('cpu')}>
							CPU% {sortColumn === 'cpu' ? (sortAscending ? '↑' : '↓') : ''}
						</th>
						<th class="sortable" onclick={() => sortBy('memory')}>
							Memory {sortColumn === 'memory' ? (sortAscending ? '↑' : '↓') : ''}
						</th>
						<th>Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each sortedProcesses as proc, idx}
						<tr class:selected={idx === selectedIndex}>
							<td class="pid">{proc.pid}</td>
							<td class="name" title={proc.command}>{proc.name}</td>
							<td class="user">{proc.user}</td>
							<td class="cpu">{proc.cpuPercent.toFixed(1)}%</td>
							<td class="memory">{proc.memoryMb.toFixed(1)} MB</td>
							<td class="actions">
								<button
									class="kill-btn soft"
									onclick={() => killProcess(proc.pid, 'soft')}
									title="SIGTERM - Graceful termination"
								>
									Kill
								</button>
								<button
									class="kill-btn hard"
									onclick={() => killProcess(proc.pid, 'hard')}
									title="SIGKILL - Force kill"
								>
									Force
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="table-container">
			<table>
				<thead>
					<tr>
						<th>Port</th>
						<th>Protocol</th>
						<th>Address</th>
						<th>PID</th>
						<th>Process</th>
						<th>State</th>
						<th>Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each ports as portInfo, idx}
						<tr class:selected={idx === selectedIndex}>
							<td class="port">{portInfo.port}</td>
							<td class="protocol">{portInfo.protocol}</td>
							<td class="address">{portInfo.localAddress}</td>
							<td class="pid">{portInfo.pid ?? '-'}</td>
							<td class="name">{portInfo.processName ?? '-'}</td>
							<td class="state">{portInfo.state}</td>
							<td class="actions">
								{#if portInfo.pid}
									<button
										class="kill-btn soft"
										onclick={() => killPortProcess(portInfo.port, 'soft')}
									>
										Kill
									</button>
									<button
										class="kill-btn hard"
										onclick={() => killPortProcess(portInfo.port, 'hard')}
									>
										Force
									</button>
								{:else}
									<span class="no-pid">No PID</span>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}

	<footer class="footer">
		<span class="hint">Tab: Switch tabs | ↑↓: Navigate | Enter: Kill (soft) | Esc: Back</span>
	</footer>
</div>

{#if showKillPortModal}
	<div class="modal-overlay" onclick={() => (showKillPortModal = false)}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<h2>Kill Process on Port</h2>
			<input
				type="number"
				placeholder="Enter port number (e.g., 3000)"
				bind:value={killPortInput}
				autofocus
				onkeydown={(e) => e.key === 'Enter' && handleQuickKillPort()}
			/>
			<div class="modal-actions">
				<button class="cancel-btn" onclick={() => (showKillPortModal = false)}> Cancel </button>
				<button class="confirm-btn" onclick={handleQuickKillPort}>Kill</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.process-manager {
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
		gap: 1rem;
		padding: 0.75rem 1rem;
		border-bottom: 1px solid var(--border-color, #333);
	}

	.header h1 {
		flex: 1;
		font-size: 1.1rem;
		font-weight: 600;
		margin: 0;
	}

	.back-button {
		background: none;
		border: none;
		color: var(--text-secondary, #aaa);
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
	}

	.back-button:hover {
		background: var(--bg-hover, #2a2a4a);
	}

	.kill-port-btn {
		background: var(--accent-color, #6366f1);
		border: none;
		color: white;
		padding: 0.375rem 0.75rem;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
	}

	.kill-port-btn:hover {
		filter: brightness(1.1);
	}

	.tabs {
		display: flex;
		gap: 0;
		padding: 0 1rem;
		border-bottom: 1px solid var(--border-color, #333);
	}

	.tab {
		background: none;
		border: none;
		color: var(--text-secondary, #aaa);
		padding: 0.75rem 1rem;
		cursor: pointer;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
	}

	.tab.active {
		color: var(--accent-color, #6366f1);
		border-bottom-color: var(--accent-color, #6366f1);
	}

	.tab:hover:not(.active) {
		color: var(--text-primary, #eee);
	}

	.search-bar {
		padding: 0.75rem 1rem;
	}

	.search-bar input {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--border-color, #333);
		border-radius: 6px;
		background: var(--bg-secondary, #252542);
		color: var(--text-primary, #eee);
		font-size: 0.9rem;
	}

	.search-bar input:focus {
		outline: none;
		border-color: var(--accent-color, #6366f1);
	}

	.error {
		padding: 0.75rem 1rem;
		background: #ff4444;
		color: white;
		margin: 0.5rem 1rem;
		border-radius: 6px;
	}

	.loading {
		padding: 2rem;
		text-align: center;
		color: var(--text-secondary, #aaa);
	}

	.table-container {
		flex: 1;
		overflow: auto;
		padding: 0 1rem;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.85rem;
	}

	th,
	td {
		padding: 0.5rem 0.75rem;
		text-align: left;
		border-bottom: 1px solid var(--border-color, #333);
	}

	th {
		font-weight: 600;
		color: var(--text-secondary, #aaa);
		position: sticky;
		top: 0;
		background: var(--bg-primary, #1a1a2e);
	}

	th.sortable {
		cursor: pointer;
	}

	th.sortable:hover {
		color: var(--text-primary, #eee);
	}

	tr:hover {
		background: var(--bg-hover, #2a2a4a);
	}

	tr.selected {
		background: var(--accent-color, #6366f1) !important;
	}

	.pid {
		font-family: monospace;
		color: var(--text-secondary, #aaa);
	}

	.name {
		max-width: 200px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.user {
		color: var(--text-secondary, #aaa);
	}

	.cpu,
	.memory {
		font-family: monospace;
		text-align: right;
	}

	.port {
		font-family: monospace;
		font-weight: 600;
		color: var(--accent-color, #6366f1);
	}

	.protocol {
		text-transform: uppercase;
		font-size: 0.75rem;
	}

	.state {
		text-transform: uppercase;
		font-size: 0.75rem;
		color: var(--text-secondary, #aaa);
	}

	.actions {
		display: flex;
		gap: 0.25rem;
	}

	.kill-btn {
		padding: 0.25rem 0.5rem;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.75rem;
	}

	.kill-btn.soft {
		background: var(--warning-color, #f59e0b);
		color: black;
	}

	.kill-btn.hard {
		background: var(--danger-color, #ef4444);
		color: white;
	}

	.kill-btn:hover {
		filter: brightness(1.1);
	}

	.no-pid {
		font-size: 0.75rem;
		color: var(--text-secondary, #666);
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

	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--bg-primary, #1a1a2e);
		border: 1px solid var(--border-color, #333);
		border-radius: 12px;
		padding: 1.5rem;
		min-width: 300px;
	}

	.modal h2 {
		margin: 0 0 1rem;
		font-size: 1.1rem;
	}

	.modal input {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--border-color, #333);
		border-radius: 6px;
		background: var(--bg-secondary, #252542);
		color: var(--text-primary, #eee);
		font-size: 1rem;
		margin-bottom: 1rem;
	}

	.modal input:focus {
		outline: none;
		border-color: var(--accent-color, #6366f1);
	}

	.modal-actions {
		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}

	.cancel-btn {
		background: var(--bg-secondary, #252542);
		border: 1px solid var(--border-color, #333);
		color: var(--text-primary, #eee);
		padding: 0.5rem 1rem;
		border-radius: 6px;
		cursor: pointer;
	}

	.confirm-btn {
		background: var(--danger-color, #ef4444);
		border: none;
		color: white;
		padding: 0.5rem 1rem;
		border-radius: 6px;
		cursor: pointer;
	}

	.cancel-btn:hover,
	.confirm-btn:hover {
		filter: brightness(1.1);
	}
</style>
