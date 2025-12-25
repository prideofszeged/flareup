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
		List,
		Trash2,
		ChevronLeft,
		ChevronRight
	} from '@lucide/svelte';
	import { focusManager } from '$lib/focus.svelte';
	import { viewManager } from '$lib/viewManager.svelte';
	import HeaderInput from './HeaderInput.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import starsSquareIcon from '$lib/assets/stars-square-1616x16@2x.png?inline';
	import SvelteMarked from 'svelte-marked';
	import { Button } from './ui/button';

	type Props = {
		onBack: () => void;
	};

	type Message = {
		role: 'user' | 'assistant';
		content: string;
	};

	type Conversation = {
		id: string;
		title: string;
		createdAt: number;
		updatedAt: number;
		model?: string;
		messages: Message[];
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
			await invoke('ai_ask_stream', {
				requestId: assistantMessageId,
				prompt: userMessage,
				options: {
					model: 'default',
					creativity: 'medium'
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
		searchInputEl?.focus();
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

		return () => {
			unlistenChunk.then((f) => f());
			unlistenEnd.then((f) => f());
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
			<HeaderInput
				placeholder="Ask anything..."
				bind:value={prompt}
				bind:ref={searchInputEl}
				autofocus
				class="!pl-2.5"
				onkeydown={handleKeydown}
			/>
			{#if isGenerating}
				<div class="mr-4">
					<Loader2 class="text-muted-foreground size-4 animate-spin" />
				</div>
			{/if}
		</Header>
	{/snippet}

	{#snippet content()}
		<div class="flex h-full">
			<!-- Sidebar -->
			{#if showSidebar}
				<div class="border-border/50 flex w-64 shrink-0 flex-col border-r">
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
							{#each conversations as conversation}
								<!-- svelte-ignore a11y_no_static_element_interactions -->
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

			<!-- Chat area -->
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
					{#each messages as message}
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
						</div>
					{/each}
				{/if}
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
