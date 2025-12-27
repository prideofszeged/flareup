<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { tick, onMount } from 'svelte';
	import {
		Loader2,
		Send,
		Stars,
		MessageSquare,
		Plus,
		Trash2,
		ChevronLeft,
		ChevronRight,
		Wrench,
		Check,
		X
	} from '@lucide/svelte';
	import { focusManager } from '$lib/focus.svelte';
	import { viewManager } from '$lib/viewManager.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import starsSquareIcon from '$lib/assets/stars-square-1616x16@2x.png?inline';
	import SvelteMarked from 'svelte-marked';

	type Props = {
		onBack: () => void;
	};

	type Message = {
		role: 'user' | 'assistant' | 'tool';
		content: string;
		toolCalls?: ToolCall[]; // For assistant messages with tool calls
		toolCallId?: string; // For tool result messages
		toolName?: string; // For tool result messages
	};

	type ToolCall = {
		id: string;
		name: string;
		arguments: Record<string, unknown>;
		safety: 'safe' | 'dangerous';
		status: 'pending' | 'running' | 'success' | 'error';
		result?: string;
		error?: string;
	};

	type Conversation = {
		id: string;
		title: string;
		createdAt: number;
		updatedAt: number;
		model?: string;
		messages: Message[];
	};

	type AiPreset = {
		id: string;
		name: string;
		icon: string | null;
		model: string | null;
		temperature: number | null;
		systemPrompt: string | null;
		webSearch: boolean;
	};

	let { onBack }: Props = $props();

	let currentConversationId = $state<string | null>(null);
	let conversations = $state<Conversation[]>([]);
	let messages = $state<Message[]>([]);
	let prompt = $state('');
	let isGenerating = $state(false);
	let showSidebar = $state(true);
	let searchInputEl: HTMLInputElement | null = $state(null);
	let scrollContainer: HTMLElement | null = $state(null);

	// Presets
	let presets = $state<AiPreset[]>([]);
	let selectedPreset = $state<AiPreset | null>(null);
	let showPresetDropdown = $state(false);

	// Tool calls state
	let pendingToolCalls = $state<Map<string, ToolCall>>(new Map());

	// Model selection for tool use
	let selectedModel = $state('default');
	let showModelDropdown = $state(false);
	let availableModels = $state<{ id: string; label: string }[]>([
		{ id: 'default', label: 'Default Model' }
	]);

	$effect(() => {
		if (focusManager.activeScope === 'main-input') {
			tick().then(() => {
				searchInputEl?.focus();
			});
		}
	});

	// Auto-scroll to bottom when messages change
	$effect(() => {
		if (messages.length > 0) {
			tick().then(() => {
				if (scrollContainer) {
					scrollContainer.scrollTop = scrollContainer.scrollHeight;
				}
			});
		}
	});

	async function handleSubmit() {
		if (!prompt.trim() || isGenerating) return;

		const userMessage = prompt.trim();
		messages = [...messages, { role: 'user', content: userMessage }];
		prompt = '';
		isGenerating = true;

		// Add placeholder for assistant message
		const assistantMessageId = Date.now().toString();
		messages = [...messages, { role: 'assistant', content: '' }];
		const assistantMessageIndex = messages.length - 1;

		try {
			// Build messages array for API (excluding empty assistant placeholder)
			const historyMessages = messages.slice(0, -1).map((m) => ({
				role: m.role,
				content: m.content
			}));

			await invoke('ai_ask_stream', {
				requestId: assistantMessageId,
				prompt: userMessage, // Keep for backwards compatibility
				options: {
					model: selectedModel === 'default' ? selectedPreset?.model || 'default' : selectedModel,
					creativity: 'medium',
					enableTools: true,
					messages: historyMessages, // Send full conversation history
					systemPrompt: selectedPreset?.systemPrompt || null
				}
			});
		} catch (error) {
			console.error('AI streaming failed:', error);
			messages[assistantMessageIndex].content =
				'Failed to get response from AI. Please check your settings and API key.';
			isGenerating = false;
		}
	}

	async function loadConversations() {
		try {
			conversations = await invoke<Conversation[]>('list_conversations');
		} catch (error) {
			console.error('Failed to load conversations:', error);
		}
	}

	async function loadConversation(id: string) {
		try {
			const conversation = await invoke<Conversation | null>('get_conversation', { id });
			if (conversation) {
				currentConversationId = conversation.id;
				messages = conversation.messages;
			}
		} catch (error) {
			console.error('Failed to load conversation:', error);
		}
	}

	function newChat() {
		messages = [];
		currentConversationId = null;
		selectedPreset = null;
		searchInputEl?.focus();
	}

	async function loadModels() {
		try {
			// Try to get Ollama models
			const ollamaModels = await invoke<string[]>('get_ollama_models', { baseUrl: '' });
			availableModels = [
				{ id: 'default', label: 'Default Model' },
				...ollamaModels.map((m) => ({ id: m, label: m.replace(/:latest$/, '') }))
			];
		} catch (error) {
			console.error('Failed to load Ollama models:', error);
			// Fallback to some common models
			availableModels = [
				{ id: 'default', label: 'Default Model' },
				{ id: 'openai/gpt-4o-mini', label: 'GPT-4o Mini' },
				{ id: 'openai/gpt-4o', label: 'GPT-4o' },
				{ id: 'anthropic/claude-3-haiku', label: 'Claude 3 Haiku' }
			];
		}
	}

	async function loadPresets() {
		try {
			presets = await invoke<AiPreset[]>('list_ai_presets');
		} catch (error) {
			console.error('Failed to load presets:', error);
		}
	}

	onMount(() => {
		// Check if we're coming from Quick AI with a conversation
		if (viewManager.quickAiPrompt && viewManager.quickAiResponse) {
			// Build the full prompt that was used
			const userContent = viewManager.quickAiSelection
				? `${viewManager.quickAiPrompt}\n\nSelected text:\n${viewManager.quickAiSelection}`
				: viewManager.quickAiPrompt;

			messages = [
				{ role: 'user', content: userContent },
				{ role: 'assistant', content: viewManager.quickAiResponse }
			];

			// Clear the Quick AI state
			viewManager.quickAiPrompt = '';
			viewManager.quickAiSelection = '';
			viewManager.quickAiResponse = '';
		}

		loadConversations();
		loadPresets();
		loadModels();

		if (viewManager.initialAiPrompt) {
			prompt = viewManager.initialAiPrompt;
			viewManager.initialAiPrompt = null;
			handleSubmit();
		}

		const unlistenChunk = listen<{ request_id: string; text: string }>(
			'ai-stream-chunk',
			(event) => {
				const { text } = event.payload;
				messages[messages.length - 1].content += text;
			}
		);

		const unlistenEnd = listen<{ request_id: string; full_text: string }>(
			'ai-stream-end',
			async () => {
				isGenerating = false;
				// Auto-save after response completes
				if (messages.length > 0) {
					await saveConversation();
				}
			}
		);

		// Listen for tool call requests
		const unlistenToolCall = listen<{
			request_id: string;
			tool_call_id: string;
			tool_name: string;
			arguments: Record<string, unknown>;
			safety: string;
		}>('ai-tool-call', (event) => {
			const { tool_call_id, tool_name, arguments: args, safety } = event.payload;

			// Add to pending tool calls
			const toolCall: ToolCall = {
				id: tool_call_id,
				name: tool_name,
				arguments: args,
				safety: safety as 'safe' | 'dangerous',
				status: 'running'
			};
			pendingToolCalls = new Map(pendingToolCalls).set(tool_call_id, toolCall);

			// Add a marker message for the tool call if not already present
			if (messages.length > 0 && messages[messages.length - 1].role === 'assistant') {
				const lastMsg = messages[messages.length - 1];
				if (!lastMsg.toolCalls) {
					lastMsg.toolCalls = [];
				}
				lastMsg.toolCalls.push(toolCall);
				messages = [...messages]; // Trigger reactivity
			}
		});

		// Listen for tool results
		const unlistenToolResult = listen<{
			request_id: string;
			tool_call_id: string;
			tool_name: string;
			success: boolean;
			output: string;
			error?: string;
		}>('ai-tool-result', (event) => {
			const { tool_call_id, success, output, error } = event.payload;

			// Update the pending tool call
			const toolCall = pendingToolCalls.get(tool_call_id);
			if (toolCall) {
				toolCall.status = success ? 'success' : 'error';
				toolCall.result = output;
				toolCall.error = error;
				pendingToolCalls = new Map(pendingToolCalls);

				// Update the tool call in messages too
				for (const msg of messages) {
					if (msg.toolCalls) {
						const tc = msg.toolCalls.find((t) => t.id === tool_call_id);
						if (tc) {
							tc.status = success ? 'success' : 'error';
							tc.result = output;
							tc.error = error;
						}
					}
				}
				messages = [...messages]; // Trigger reactivity
			}
		});

		return () => {
			unlistenChunk.then((f) => f());
			unlistenEnd.then((f) => f());
			unlistenToolCall.then((f) => f());
			unlistenToolResult.then((f) => f());
		};
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	}

	async function saveConversation() {
		try {
			if (!currentConversationId) {
				// Create new conversation
				const title = messages[0]?.content.slice(0, 50) || 'New Chat';
				const conversation = await invoke<Conversation>('create_conversation', {
					title,
					model: null
				});
				currentConversationId = conversation.id;
			}
			// Update existing conversation
			await invoke('update_conversation', {
				id: currentConversationId,
				title: null,
				messages: messages
			});
		} catch (error) {
			console.error('Failed to save conversation:', error);
		}
	}

	function clearChat() {
		newChat();
	}

	async function deleteConversation(id: string) {
		try {
			await invoke('delete_conversation', { id });
			conversations = conversations.filter((c) => c.id !== id);
			if (currentConversationId === id) {
				newChat();
			}
		} catch (error) {
			console.error('Failed to delete conversation:', error);
		}
	}

	function toggleSidebar() {
		showSidebar = !showSidebar;
	}

	function formatDate(timestamp: number): string {
		const date = new Date(timestamp);
		const now = new Date();
		const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

		if (diffDays === 0) {
			return 'Today';
		} else if (diffDays === 1) {
			return 'Yesterday';
		} else if (diffDays < 7) {
			return `${diffDays} days ago`;
		} else {
			return date.toLocaleDateString();
		}
	}
</script>

<MainLayout>
	{#snippet header()}
		<Header showBackButton={true} onPopView={onBack}>
			<span class="text-lg font-medium">AI Chat</span>
			<div class="flex-1"></div>
			{#if isGenerating}
				<div class="mr-4">
					<Loader2 class="text-muted-foreground size-4 animate-spin" />
				</div>
			{/if}

			<!-- Preset Selector -->
			{#if presets.length > 0}
				<div class="relative mr-2">
					<button
						type="button"
						onclick={() => (showPresetDropdown = !showPresetDropdown)}
						class="border-input bg-background hover:bg-accent flex h-8 items-center gap-1 rounded-md border px-2 text-xs transition-colors"
						title="Select AI Preset"
					>
						<Stars class="size-3" />
						<span class="max-w-24 truncate">{selectedPreset?.name || 'No preset'}</span>
						<svg class="h-3 w-3 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 9l-7 7-7-7"
							/>
						</svg>
					</button>
					{#if showPresetDropdown}
						<div class="bg-popover absolute right-0 z-50 mt-1 w-48 rounded-md border shadow-lg">
							<button
								type="button"
								class="hover:bg-accent w-full px-3 py-2 text-left text-sm {!selectedPreset
									? 'bg-accent'
									: ''}"
								onclick={() => {
									selectedPreset = null;
									showPresetDropdown = false;
								}}
							>
								No preset
							</button>
							{#each presets as preset (preset.id)}
								<button
									type="button"
									class="hover:bg-accent w-full px-3 py-2 text-left text-sm {selectedPreset?.id ===
									preset.id
										? 'bg-accent'
										: ''}"
									onclick={() => {
										selectedPreset = preset;
										showPresetDropdown = false;
									}}
								>
									{preset.name}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}

			<!-- Model Selector -->
			<div class="relative mr-2">
				<button
					type="button"
					onclick={() => (showModelDropdown = !showModelDropdown)}
					class="border-input bg-background hover:bg-accent flex h-8 items-center gap-1 rounded-md border px-2 text-xs transition-colors"
					title="Select AI Model"
				>
					<Wrench class="size-3" />
					<span class="max-w-28 truncate"
						>{availableModels.find((m) => m.id === selectedModel)?.label || 'Model'}</span
					>
					<svg class="h-3 w-3 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M19 9l-7 7-7-7"
						/>
					</svg>
				</button>
				{#if showModelDropdown}
					<div
						class="bg-popover absolute right-0 z-50 mt-1 max-h-80 w-56 overflow-y-auto rounded-md border shadow-lg"
					>
						<div class="text-muted-foreground bg-popover sticky top-0 border-b px-3 py-1.5 text-xs">
							Available Models
						</div>
						{#each availableModels as model (model.id)}
							<button
								type="button"
								class="hover:bg-accent w-full px-3 py-2 text-left text-sm {selectedModel ===
								model.id
									? 'bg-accent'
									: ''}"
								onclick={() => {
									selectedModel = model.id;
									showModelDropdown = false;
								}}
							>
								{model.label}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</Header>
	{/snippet}

	{#snippet content()}
		<div class="flex h-full overflow-hidden">
			<!-- Sidebar -->
			{#if showSidebar}
				<div class="border-border/50 flex w-64 shrink-0 flex-col overflow-hidden border-r">
					<!-- Sidebar header -->
					<div class="border-border/50 flex items-center justify-between border-b p-3">
						<span class="text-sm font-medium">Conversations</span>
						<div class="flex gap-1">
							<button
								onclick={newChat}
								class="hover:bg-accent rounded p-1.5 transition-colors"
								title="New Chat"
							>
								<Plus class="size-4" />
							</button>
							<button
								onclick={toggleSidebar}
								class="hover:bg-accent rounded p-1.5 transition-colors"
								title="Hide Sidebar"
							>
								<ChevronLeft class="size-4" />
							</button>
						</div>
					</div>

					<!-- Conversation list -->
					<div class="flex-1 overflow-y-auto">
						{#if conversations.length === 0}
							<div class="text-muted-foreground p-4 text-center text-sm">No conversations yet</div>
						{:else}
							{#each conversations as conversation (conversation.id)}
								<div
									onclick={() => loadConversation(conversation.id)}
									onkeydown={(e) => e.key === 'Enter' && loadConversation(conversation.id)}
									role="button"
									tabindex="0"
									class="hover:bg-accent/50 group flex w-full cursor-pointer items-start gap-2 px-3 py-2.5 text-left transition-colors {currentConversationId ===
									conversation.id
										? 'bg-accent'
										: ''}"
								>
									<MessageSquare class="text-muted-foreground mt-0.5 size-4 shrink-0" />
									<div class="min-w-0 flex-1">
										<div class="truncate text-sm font-medium">{conversation.title}</div>
										<div class="text-muted-foreground text-xs">
											{formatDate(conversation.updatedAt)}
										</div>
									</div>
									<button
										onclick={(e) => {
											e.stopPropagation();
											deleteConversation(conversation.id);
										}}
										class="text-muted-foreground hover:text-destructive opacity-0 transition-opacity group-hover:opacity-100"
										title="Delete"
									>
										<Trash2 class="size-4" />
									</button>
								</div>
							{/each}
						{/if}
					</div>
				</div>
			{:else}
				<!-- Collapsed sidebar toggle -->
				<button
					onclick={toggleSidebar}
					class="border-border/50 hover:bg-accent flex shrink-0 items-center border-r px-2 transition-colors"
					title="Show Sidebar"
				>
					<ChevronRight class="text-muted-foreground size-4" />
				</button>
			{/if}

			<!-- Chat column (scroll area + input) -->
			<div class="flex min-w-0 flex-1 flex-col">
				<!-- Chat messages area -->
				<div
					bind:this={scrollContainer}
					class="flex grow flex-col gap-6 overflow-y-auto scroll-smooth p-6"
				>
					{#if messages.length === 0}
						<div class="flex h-full flex-col items-center justify-center text-center">
							<div class="bg-primary/10 mb-4 rounded-2xl p-4">
								<Stars class="text-primary size-12" />
							</div>
							<h2 class="text-2xl font-semibold">How can I help you today?</h2>
							<p class="text-muted-foreground mt-2 max-w-sm">
								Ask anything, from coding questions to general knowledge. Press Ctrl+, to configure.
							</p>
						</div>
					{:else}
						{#each messages as message, i (i)}
							<div
								class="flex flex-col gap-2 {message.role === 'user'
									? 'ml-12 items-end'
									: 'mr-12 items-start'}"
							>
								<div
									class="rounded-2xl px-4 py-3 text-sm leading-relaxed {message.role === 'user'
										? 'bg-primary text-primary-foreground shadow-sm'
										: 'bg-muted/50 border-border/50 border'}"
								>
									{#if message.role === 'assistant'}
										<div class="prose prose-sm prose-invert max-w-none">
											<SvelteMarked source={message.content} />
										</div>
									{:else}
										{message.content}
									{/if}
								</div>

								<!-- Tool calls display -->
								{#if message.toolCalls && message.toolCalls.length > 0}
									<div class="mt-2 space-y-2">
										{#each message.toolCalls as toolCall (toolCall.id)}
											<div
												class="border-border/50 bg-background/50 flex items-start gap-2 rounded-lg border p-2 text-xs"
											>
												<div class="mt-0.5">
													{#if toolCall.status === 'running'}
														<Loader2 class="text-muted-foreground size-3.5 animate-spin" />
													{:else if toolCall.status === 'success'}
														<Check class="size-3.5 text-green-500" />
													{:else if toolCall.status === 'error'}
														<X class="size-3.5 text-red-500" />
													{:else}
														<Wrench class="text-muted-foreground size-3.5" />
													{/if}
												</div>
												<div class="flex-1">
													<div class="flex items-center gap-1.5">
														<span class="font-medium">{toolCall.name}</span>
														{#if toolCall.safety === 'dangerous'}
															<span class="rounded bg-yellow-500/20 px-1 text-yellow-500">⚠️</span>
														{/if}
													</div>
													{#if toolCall.result}
														<div
															class="text-muted-foreground mt-1 max-h-24 overflow-y-auto font-mono whitespace-pre-wrap"
														>
															{toolCall.result.slice(0, 500)}{toolCall.result.length > 500
																? '...'
																: ''}
														</div>
													{:else if toolCall.error}
														<div class="text-destructive mt-1">{toolCall.error}</div>
													{/if}
												</div>
											</div>
										{/each}
									</div>
								{/if}
							</div>
						{/each}
					{/if}
				</div>

				<!-- Chat input at bottom -->
				<div class="border-border/50 bg-background shrink-0 border-t p-4">
					<div class="relative">
						<input
							type="text"
							placeholder="Ask anything..."
							bind:value={prompt}
							bind:this={searchInputEl}
							onkeydown={handleKeydown}
							disabled={isGenerating}
							class="bg-muted/50 border-input focus:ring-primary w-full rounded-xl border px-4 py-3 pr-12 text-sm transition-all focus:ring-2 focus:outline-none"
						/>
						<button
							onclick={handleSubmit}
							disabled={isGenerating || !prompt.trim()}
							class="bg-primary hover:bg-primary/90 disabled:bg-muted disabled:text-muted-foreground absolute top-1/2 right-2 -translate-y-1/2 rounded-lg p-2 text-white transition-colors"
						>
							{#if isGenerating}
								<Loader2 class="size-4 animate-spin" />
							{:else}
								<Send class="size-4" />
							{/if}
						</button>
					</div>
				</div>
			</div>
		</div>
	{/snippet}

	{#snippet footer()}
		<ActionBar
			actions={[
				{
					title: 'Send',
					shortcut: { key: 'Enter', modifiers: [] },
					handler: handleSubmit
				},
				{
					title: 'New Chat',
					shortcut: { key: 'n', modifiers: ['ctrl'] },
					handler: newChat
				},
				{
					title: 'Configure AI',
					shortcut: { key: ',', modifiers: ['ctrl'] },
					handler: () => viewManager.showSettings()
				},
				{
					title: 'Clear Chat',
					shortcut: { key: 'l', modifiers: ['ctrl'] },
					handler: clearChat
				}
			]}
			icon={starsSquareIcon}
			title="AI Chat"
		/>
	{/snippet}
</MainLayout>

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
