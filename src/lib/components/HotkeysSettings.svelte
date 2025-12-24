<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import type { PluginInfo } from '@flare/protocol';
	import { onMount } from 'svelte';

	interface HotkeyConfig {
		commandId: string;
		hotkey: string;
		modifiers: number;
		key: string;
	}

	export let plugins: PluginInfo[];
	export let onBack: () => void;

	let hotkeys: HotkeyConfig[] = [];
	let recordingFor: string | null = null;
	let conflictWarning: string | null = null;
	let pressedModifiers = 0;
	let pressedKey = '';

	onMount(async () => {
		await loadHotkeys();
	});

	async function loadHotkeys() {
		try {
			hotkeys = await invoke<HotkeyConfig[]>('get_hotkey_config');
		} catch (e) {
			console.error('Failed to load hotkeys:', e);
		}
	}

	function getHotkey(commandId: string): HotkeyConfig | undefined {
		return hotkeys.find((h) => h.commandId === commandId);
	}

	function startRecording(commandId: string) {
		recordingFor = commandId;
		conflictWarning = null;
		pressedModifiers = 0;
		pressedKey = '';
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!recordingFor) return;

		event.preventDefault();
		event.stopPropagation();

		// Calculate modifiers bitmask
		pressedModifiers = 0;
		if (event.ctrlKey) pressedModifiers |= 1;
		if (event.altKey) pressedModifiers |= 2;
		if (event.shiftKey) pressedModifiers |= 4;
		if (event.metaKey) pressedModifiers |= 8;

		// Get key code
		const key = event.code;

		// Ignore modifier-only presses
		if (
			key === 'Control' ||
			key === 'Alt' ||
			key === 'Shift' ||
			key === 'Meta' ||
			key === 'ControlLeft' ||
			key === 'ControlRight' ||
			key === 'AltLeft' ||
			key === 'AltRight' ||
			key === 'ShiftLeft' ||
			key === 'ShiftRight' ||
			key === 'MetaLeft' ||
			key === 'MetaRight'
		) {
			return;
		}

		// Require at least one modifier
		if (pressedModifiers === 0) {
			conflictWarning = 'Hotkeys must include at least one modifier (Ctrl, Alt, Shift, or Super)';
			return;
		}

		pressedKey = key;
	}

	async function saveHotkey() {
		if (!recordingFor || !pressedKey) return;

		try {
			// Check for conflicts
			const conflict = await invoke<string | null>('check_hotkey_conflict', {
				modifiers: pressedModifiers,
				key: pressedKey
			});

			if (conflict && conflict !== recordingFor) {
				const conflictPlugin = plugins.find((p) => p.pluginPath === conflict);
				conflictWarning = `Already assigned to: ${conflictPlugin?.title || conflict}`;
				return;
			}

			// Save the hotkey
			await invoke('set_command_hotkey', {
				commandId: recordingFor,
				modifiers: pressedModifiers,
				key: pressedKey
			});

			// Reload hotkeys
			await loadHotkeys();

			// Clear recording state
			recordingFor = null;
			conflictWarning = null;
			pressedModifiers = 0;
			pressedKey = '';
		} catch (e) {
			console.error('Failed to save hotkey:', e);
			conflictWarning = `Error: ${e}`;
		}
	}

	function cancelRecording() {
		recordingFor = null;
		conflictWarning = null;
		pressedModifiers = 0;
		pressedKey = '';
	}

	async function removeHotkey(commandId: string) {
		try {
			await invoke('remove_command_hotkey', { commandId });
			await loadHotkeys();
		} catch (e) {
			console.error('Failed to remove hotkey:', e);
		}
	}

	async function resetToDefaults() {
		if (confirm('Reset all hotkeys to defaults? This will remove your custom configurations.')) {
			try {
				await invoke('reset_hotkeys_to_defaults');
				await loadHotkeys();
			} catch (e) {
				console.error('Failed to reset hotkeys:', e);
			}
		}
	}

	function formatModifiers(mods: number): string[] {
		const parts: string[] = [];
		if (mods & 8) parts.push('Super');
		if (mods & 1) parts.push('Ctrl');
		if (mods & 2) parts.push('Alt');
		if (mods & 4) parts.push('Shift');
		return parts;
	}

	function formatKey(key: string): string {
		if (key.startsWith('Key')) return key.slice(3); // "KeyV" -> "V"
		if (key.startsWith('Digit')) return key.slice(5); // "Digit5" -> "5"
		if (key === 'ArrowLeft') return '←';
		if (key === 'ArrowRight') return '→';
		if (key === 'ArrowUp') return '↑';
		if (key === 'ArrowDown') return '↓';
		if (key === 'Space') return 'Space';
		if (key === 'Enter') return 'Enter';
		if (key === 'Minus') return '-';
		if (key === 'Equal') return '=';
		return key;
	}

	function formatCurrentRecording(): string {
		if (!pressedKey) return 'Press a key combination...';
		const mods = formatModifiers(pressedModifiers);
		const key = formatKey(pressedKey);
		return [...mods, key].join('+');
	}
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="hotkeys-settings">
	<div class="header">
		<button class="back-button" on:click={onBack}>← Back</button>
		<h1>Hotkey Configuration</h1>
		<button class="reset-button" on:click={resetToDefaults}>Reset to Defaults</button>
	</div>

	<div class="hotkey-list">
		{#each plugins as plugin}
			{@const hotkey = getHotkey(plugin.pluginPath)}
			<div class="hotkey-row">
				<div class="command-info">
					<div class="command-title">{plugin.title}</div>
					<div class="command-desc">{plugin.description}</div>
					<div class="command-path">{plugin.pluginPath}</div>
				</div>

				<div class="hotkey-input">
					{#if recordingFor === plugin.pluginPath}
						<div class="recording-box">
							<div class="recording-display">{formatCurrentRecording()}</div>
							<div class="recording-buttons">
								<button class="save-btn" on:click={saveHotkey} disabled={!pressedKey}>
									Save
								</button>
								<button class="cancel-btn" on:click={cancelRecording}>Cancel</button>
							</div>
						</div>
					{:else if hotkey}
						<div class="hotkey-display">
							<span class="hotkey-text">{hotkey.hotkey}</span>
							<button class="remove-btn" on:click={() => removeHotkey(plugin.pluginPath)}>
								×
							</button>
						</div>
					{:else}
						<button class="set-btn" on:click={() => startRecording(plugin.pluginPath)}>
							Set Hotkey
						</button>
					{/if}
				</div>
			</div>
		{/each}
	</div>

	{#if conflictWarning}
		<div class="warning-box">{conflictWarning}</div>
	{/if}
</div>

<style>
	.hotkeys-settings {
		padding: 20px;
		max-width: 1000px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 30px;
		gap: 20px;
	}

	.header h1 {
		flex: 1;
		margin: 0;
		font-size: 24px;
		font-weight: 600;
	}

	.back-button,
	.reset-button {
		padding: 8px 16px;
		border-radius: 6px;
		border: 1px solid var(--border-color, #ddd);
		background: var(--bg-secondary, #f5f5f5);
		cursor: pointer;
		font-size: 14px;
		transition: all 0.2s;
	}

	.back-button:hover,
	.reset-button:hover {
		background: var(--bg-hover, #e0e0e0);
	}

	.hotkey-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.hotkey-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		border-radius: 8px;
		border: 1px solid var(--border-color, #ddd);
		background: var(--bg-card, white);
		transition: all 0.2s;
	}

	.hotkey-row:hover {
		border-color: var(--border-hover, #999);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.command-info {
		flex: 1;
		min-width: 0;
	}

	.command-title {
		font-size: 16px;
		font-weight: 500;
		margin-bottom: 4px;
	}

	.command-desc {
		font-size: 13px;
		color: var(--text-secondary, #666);
		margin-bottom: 4px;
	}

	.command-path {
		font-size: 11px;
		color: var(--text-tertiary, #999);
		font-family: monospace;
	}

	.hotkey-input {
		min-width: 200px;
		display: flex;
		justify-content: flex-end;
	}

	.hotkey-display {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 6px;
		background: var(--bg-secondary, #f5f5f5);
		border: 1px solid var(--border-color, #ddd);
	}

	.hotkey-text {
		font-family: monospace;
		font-size: 14px;
		font-weight: 500;
	}

	.remove-btn {
		padding: 2px 8px;
		border: none;
		background: transparent;
		cursor: pointer;
		font-size: 20px;
		line-height: 1;
		color: var(--text-secondary, #666);
		transition: color 0.2s;
	}

	.remove-btn:hover {
		color: var(--danger, #d32f2f);
	}

	.set-btn {
		padding: 8px 16px;
		border-radius: 6px;
		border: 1px solid var(--primary, #007aff);
		background: transparent;
		color: var(--primary, #007aff);
		cursor: pointer;
		font-size: 14px;
		transition: all 0.2s;
	}

	.set-btn:hover {
		background: var(--primary, #007aff);
		color: white;
	}

	.recording-box {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		border-radius: 6px;
		background: var(--bg-highlight, #fff9e6);
		border: 2px solid var(--primary, #007aff);
		min-width: 250px;
	}

	.recording-display {
		font-family: monospace;
		font-size: 14px;
		font-weight: 500;
		text-align: center;
		padding: 8px;
		border-radius: 4px;
		background: white;
		border: 1px solid var(--border-color, #ddd);
		min-height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.recording-buttons {
		display: flex;
		gap: 8px;
	}

	.save-btn,
	.cancel-btn {
		flex: 1;
		padding: 6px 12px;
		border-radius: 4px;
		border: 1px solid;
		cursor: pointer;
		font-size: 13px;
		transition: all 0.2s;
	}

	.save-btn {
		border-color: var(--success, #4caf50);
		background: var(--success, #4caf50);
		color: white;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--success-dark, #45a049);
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.cancel-btn {
		border-color: var(--border-color, #ddd);
		background: var(--bg-secondary, #f5f5f5);
		color: var(--text-primary, #333);
	}

	.cancel-btn:hover {
		background: var(--bg-hover, #e0e0e0);
	}

	.warning-box {
		margin-top: 20px;
		padding: 12px 16px;
		border-radius: 6px;
		background: var(--warning-bg, #fff3cd);
		border: 1px solid var(--warning, #ffc107);
		color: var(--warning-text, #856404);
		font-size: 14px;
	}
</style>
