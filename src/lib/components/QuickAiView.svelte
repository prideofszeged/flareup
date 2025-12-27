<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { tick, onMount, onDestroy } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { Loader2, Stars, Send } from '@lucide/svelte';
	import SvelteMarked from 'svelte-marked';
	import { viewManager } from '$lib/viewManager.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import type { ActionDefinition } from './nodes/shared/actions';
	import starsSquareIcon from '$lib/assets/stars-square-1616x16@2x.png?inline';
	import { Input } from './ui/input';

	type Props = {
		prompt: string;
		selection?: string;
		onClose: () => void;
	};

	type Message = {
		role: 'user' | 'assistant';
		content: string;
	};

	let { prompt, selection = '', onClose }: Props = $props();

	let messages = $state<Message[]>([]);
	let followUpPrompt = $state('');
	let isGenerating = $state(false);
	let error = $state<string | null>(null);
	let scrollContainer: HTMLElement | null = $state(null);
	let followUpInputEl: HTMLInputElement | null = $state(null);
	let requestId = $state('');
	let unlistenChunk: (() => void) | null = null;
	let unlistenEnd: (() => void) | null = null;

	// Build the full prompt for context
	function buildContextPrompt(userMessage: string): string {
		// For follow-ups, include conversation history
		if (messages.length === 0) {
			// First message - include selection if available
			return selection ? `${userMessage}\n\nSelected text:\n${selection}` : userMessage;
		}

		// Build context from previous messages
		let context = messages
			.map((m) => (m.role === 'user' ? `User: ${m.content}` : `Assistant: ${m.content}`))
			.join('\n\n');

		return `${context}\n\nUser: ${userMessage}`;
	}

	async function askQuestion(userMessage: string) {
		if (!userMessage.trim()) return;
		if (isGenerating) return;

		// Add user message to history
		messages = [...messages, { role: 'user', content: userMessage }];

		// Add placeholder for assistant
		messages = [...messages, { role: 'assistant', content: '' }];

		error = null;
		isGenerating = true;
		requestId = Date.now().toString();

		const contextPrompt = buildContextPrompt(userMessage);

		try {
			await invoke('ai_ask_stream', {
				requestId,
				prompt: contextPrompt,
				options: {
					model: 'default',
					creativity: 'medium'
				}
			});
		} catch (e) {
			console.error('[QuickAI] ai_ask_stream error:', e);
			error = String(e);
			isGenerating = false;
		}
	}

	async function handleFollowUp() {
		if (!followUpPrompt.trim() || isGenerating) return;
		const question = followUpPrompt.trim();
		followUpPrompt = '';
		await askQuestion(question);
	}

	async function copyAllResponses() {
		const allContent = messages
			.filter((m) => m.role === 'assistant')
			.map((m) => m.content)
			.join('\n\n---\n\n');

		if (allContent) {
			await writeText(allContent);
			await invoke('show_hud', { title: 'Copied to clipboard' });
		}
	}

	async function openInChat() {
		// Save the full conversation for AI Chat
		viewManager.quickAiPrompt = prompt;
		viewManager.quickAiSelection = selection;
		// Combine all assistant responses
		viewManager.quickAiResponse = messages
			.filter((m) => m.role === 'assistant')
			.map((m) => m.content)
			.join('\n\n');
		viewManager.showAiChat();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onClose();
		} else if (e.key === 'o' && e.ctrlKey) {
			e.preventDefault();
			openInChat();
		}
	}

	function handleInputKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleFollowUp();
		}
	}

	// Auto-scroll to bottom when messages change
	$effect(() => {
		if (messages.length > 0 && scrollContainer) {
			tick().then(() => {
				if (scrollContainer) {
					scrollContainer.scrollTop = scrollContainer.scrollHeight;
				}
			});
		}
	});

	// Focus follow-up input when generation completes
	$effect(() => {
		if (!isGenerating && followUpInputEl) {
			tick().then(() => followUpInputEl?.focus());
		}
	});

	onMount(async () => {
		// Set up event listeners for streaming
		const chunkUnlisten = await listen<{ requestId: string; text: string }>(
			'ai-stream-chunk',
			(event) => {
				if (event.payload.requestId === requestId || requestId === '') {
					// Update the last assistant message
					if (messages.length > 0 && messages[messages.length - 1].role === 'assistant') {
						messages[messages.length - 1].content += event.payload.text;
					}
				}
			}
		);
		unlistenChunk = chunkUnlisten;

		const endUnlisten = await listen<{ requestId: string; fullText: string }>(
			'ai-stream-end',
			() => {
				isGenerating = false;
			}
		);
		unlistenEnd = endUnlisten;

		// Start the initial stream with the original prompt
		askQuestion(prompt);
	});

	onDestroy(() => {
		unlistenChunk?.();
		unlistenEnd?.();
	});

	const actions: ActionDefinition[] = $derived.by(() => {
		const result: ActionDefinition[] = [];

		if (!isGenerating && messages.some((m) => m.role === 'assistant' && m.content)) {
			result.push({
				title: 'Copy All Responses',
				shortcut: { key: 'c', modifiers: ['ctrl', 'shift'] },
				handler: copyAllResponses
			});
		}

		result.push({
			title: 'Open in AI Chat',
			shortcut: { key: 'o', modifiers: ['ctrl'] },
			handler: openInChat
		});

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
	role="dialog"
	aria-modal="true"
	aria-label="Quick AI assistant"
	tabindex="-1"
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
	<!-- Header -->
	<div class="border-border/50 flex items-center gap-2 border-b px-4 py-3">
		<div class="bg-primary/10 rounded-lg p-1.5">
			<Stars class="text-primary size-4" />
		</div>
		<div class="min-w-0 flex-1">
			<div class="truncate text-sm font-medium">Quick AI</div>
			<div class="text-muted-foreground truncate text-xs">
				{messages.length > 2 ? `${Math.floor(messages.length / 2)} messages` : prompt}
			</div>
		</div>
		{#if isGenerating}
			<Loader2 class="text-muted-foreground size-4 animate-spin" />
		{/if}
	</div>

	<!-- Messages area -->
	<div bind:this={scrollContainer} class="flex-1 space-y-4 overflow-y-auto p-4">
		{#if error}
			<div class="text-destructive text-sm">
				{error}
			</div>
		{:else}
			{#each messages as message, i (i)}
				<div class="flex flex-col gap-1 {message.role === 'user' ? 'items-end' : 'items-start'}">
					<div
						class="max-w-[85%] rounded-2xl px-4 py-2.5 text-sm {message.role === 'user'
							? 'bg-primary text-primary-foreground'
							: 'bg-muted/50 border-border/50 border'}"
					>
						{#if message.role === 'assistant'}
							{#if message.content}
								<div class="prose prose-sm prose-invert max-w-none">
									<SvelteMarked source={message.content} />
								</div>
							{:else if isGenerating}
								<div class="text-muted-foreground flex items-center gap-2">
									<Loader2 class="size-3 animate-spin" />
									<span>Thinking...</span>
								</div>
							{/if}
						{:else}
							{message.content}
						{/if}
					</div>
				</div>
			{/each}
		{/if}
	</div>

	<!-- Follow-up input -->
	<div class="border-border/50 border-t px-4 py-3">
		<div class="flex items-center gap-2">
			<Input
				bind:ref={followUpInputEl}
				bind:value={followUpPrompt}
				placeholder={isGenerating ? 'Wait for response...' : 'Ask a follow-up question...'}
				disabled={isGenerating}
				class="flex-1"
				onkeydown={handleInputKeydown}
			/>
			<button
				onclick={handleFollowUp}
				disabled={isGenerating || !followUpPrompt.trim()}
				class="bg-primary text-primary-foreground hover:bg-primary/90 disabled:bg-muted disabled:text-muted-foreground rounded-lg p-2 transition-colors"
			>
				<Send class="size-4" />
			</button>
		</div>
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
