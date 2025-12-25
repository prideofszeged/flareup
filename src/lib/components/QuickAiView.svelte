<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { tick, onMount, onDestroy } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { Loader2, Copy, MessageSquare, RefreshCw, Stars } from '@lucide/svelte';
	import SvelteMarked from 'svelte-marked';
	import { viewManager } from '$lib/viewManager.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import type { ActionDefinition } from './nodes/shared/actions';
	import starsSquareIcon from '$lib/assets/stars-square-1616x16@2x.png?inline';

	type Props = {
		prompt: string;
		selection?: string;
		onClose: () => void;
	};

	let { prompt, selection = '', onClose }: Props = $props();

	let response = $state('');
	let isGenerating = $state(true);
	let error = $state<string | null>(null);
	let scrollContainer: HTMLElement | null = $state(null);
	let requestId = $state('');
	let unlistenChunk: (() => void) | null = null;
	let unlistenEnd: (() => void) | null = null;

	// Combine prompt with selection if available
	const fullPrompt = $derived(selection ? `${prompt}\n\nSelected text:\n${selection}` : prompt);

	async function startStream() {
		response = '';
		error = null;
		isGenerating = true;
		requestId = Date.now().toString();
		console.log('[QuickAI] Starting stream with requestId:', requestId, 'prompt:', fullPrompt);

		try {
			await invoke('ai_ask_stream', {
				requestId,
				prompt: fullPrompt,
				options: {
					model: 'default',
					creativity: 'medium'
				}
			});
			console.log('[QuickAI] ai_ask_stream invoke completed');
		} catch (e) {
			console.error('[QuickAI] ai_ask_stream error:', e);
			error = String(e);
			isGenerating = false;
		}
	}

	async function copyResponse() {
		if (response) {
			await writeText(response);
			await invoke('show_hud', { title: 'Copied to clipboard' });
		}
	}

	async function openInChat() {
		// Save the conversation state for AI Chat to pick up
		// Keep the prompt/selection and save the response
		viewManager.quickAiResponse = response;
		// Navigate directly to AI chat
		viewManager.showAiChat();
	}

	async function regenerate() {
		await startStream();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onClose();
		} else if (e.key === 'Enter' && !isGenerating) {
			e.preventDefault();
			copyResponse();
		} else if (e.key === 'o' && e.ctrlKey) {
			e.preventDefault();
			openInChat();
		} else if (e.key === 'r' && e.ctrlKey) {
			e.preventDefault();
			regenerate();
		}
	}

	// Auto-scroll to bottom when response updates
	$effect(() => {
		if (response && scrollContainer) {
			tick().then(() => {
				if (scrollContainer) {
					scrollContainer.scrollTop = scrollContainer.scrollHeight;
				}
			});
		}
	});

	onMount(async () => {
		// Set up event listeners for streaming
		console.log('[QuickAI] Setting up event listeners');
		const chunkUnlisten = await listen<{ requestId: string; text: string }>(
			'ai-stream-chunk',
			(event) => {
				console.log(
					'[QuickAI] Received chunk:',
					event.payload.requestId,
					event.payload.text.substring(0, 50)
				);
				if (event.payload.requestId === requestId || requestId === '') {
					response += event.payload.text;
				}
			}
		);
		unlistenChunk = chunkUnlisten;

		const endUnlisten = await listen<{ requestId: string; fullText: string }>(
			'ai-stream-end',
			(event) => {
				console.log('[QuickAI] Stream ended:', event.payload.requestId);
				isGenerating = false;
			}
		);
		unlistenEnd = endUnlisten;

		// Start the AI stream
		console.log('[QuickAI] Calling startStream');
		startStream();
	});

	onDestroy(() => {
		unlistenChunk?.();
		unlistenEnd?.();
	});

	const actions: ActionDefinition[] = $derived.by(() => {
		const result: ActionDefinition[] = [];

		if (!isGenerating && response) {
			result.push({
				title: 'Copy Response',
				shortcut: { key: 'Enter', modifiers: [] },
				handler: copyResponse
			});
		}

		result.push({
			title: 'Open in AI Chat',
			shortcut: { key: 'o', modifiers: ['ctrl'] },
			handler: openInChat
		});

		if (!isGenerating) {
			result.push({
				title: 'Regenerate',
				shortcut: { key: 'r', modifiers: ['ctrl'] },
				handler: regenerate
			});
		}

		result.push({
			title: 'Close',
			shortcut: { key: 'Escape', modifiers: [] },
			handler: onClose
		});

		return result;
	});
</script>

<svelte:window onkeydown={handleKeydown} />
<div
	role="application"
	tabindex="0"
	class="bg-background text-foreground flex h-screen flex-col outline-none"
	style="
		box-shadow: 
			inset 0 0 0 2px rgba(255, 255, 255, 0.6),
			0 0 0 1px rgba(255, 255, 255, 0.5),
			0 20px 60px rgba(0, 0, 0, 0.9),
			0 10px 30px rgba(0, 0, 0, 0.7);
		border-radius: 8px;
	"
>
	<!-- Header showing the prompt -->
	<div class="border-border/50 flex items-center gap-2 border-b px-4 py-3">
		<div class="bg-primary/10 rounded-lg p-1.5">
			<Stars class="text-primary size-4" />
		</div>
		<div class="min-w-0 flex-1">
			<div class="truncate text-sm font-medium">Quick AI</div>
			<div class="text-muted-foreground truncate text-xs">{prompt}</div>
		</div>
		{#if isGenerating}
			<Loader2 class="text-muted-foreground size-4 animate-spin" />
		{/if}
	</div>

	<!-- Response area -->
	<div bind:this={scrollContainer} class="flex-1 overflow-y-auto p-4">
		{#if error}
			<div class="text-destructive text-sm">
				{error}
			</div>
		{:else if response}
			<div class="prose prose-sm prose-invert max-w-none">
				<SvelteMarked source={response} />
			</div>
		{:else if isGenerating}
			<div class="text-muted-foreground flex items-center gap-2 text-sm">
				<Loader2 class="size-4 animate-spin" />
				Thinking...
			</div>
		{/if}
	</div>

	<!-- Action bar -->
	<ActionBar {actions} icon={starsSquareIcon} title="Quick AI" />
</div>

<style>
	:global(.prose pre) {
		background-color: rgba(0, 0, 0, 0.2);
		padding: 1rem;
		border-radius: 0.5rem;
		overflow-x: auto;
	}
	:global(.prose code) {
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
			monospace;
		font-size: 0.9em;
	}
	:global(.prose p:first-child) {
		margin-top: 0;
	}
	:global(.prose p:last-child) {
		margin-bottom: 0;
	}
</style>
