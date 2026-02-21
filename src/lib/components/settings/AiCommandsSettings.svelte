<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { Plus, Edit2, Trash2, Sparkles } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	type OutputAction = 'quick_ai' | 'open_chat' | 'copy' | 'paste';

	type AiCommand = {
		id: string;
		name: string;
		icon: string | null;
		promptTemplate: string;
		model: string | null;
		creativity: string | null;
		outputAction: OutputAction;
		hotkey: string | null;
		createdAt: number;
		updatedAt: number;
	};

	type PlaceholderInfo = {
		name: string;
		description: string;
	};

	let commands = $state<AiCommand[]>([]);
	let placeholders = $state<PlaceholderInfo[]>([]);
	let isLoading = $state(true);
	let showForm = $state(false);
	let editingCommand = $state<AiCommand | null>(null);

	// Form state
	let formName = $state('');
	let formPromptTemplate = $state('');
	let formOutputAction = $state<OutputAction>('quick_ai');
	let formCreativity = $state('medium');
	let formModel = $state<string | null>(null);
	let showOutputActionDropdown = $state(false);
	let showCreativityDropdown = $state(false);
	let showModelDropdown = $state(false);

	// Available models
	let ollamaModels = $state<string[]>([]);
	let isLoadingModels = $state(false);

	const outputActionOptions = [
		{ value: 'quick_ai', label: 'Quick AI' },
		{ value: 'open_chat', label: 'Open in AI Chat' },
		{ value: 'copy', label: 'Copy to Clipboard' },
		{ value: 'paste', label: 'Paste in Place' }
	];

	const creativityOptions = [
		{ value: 'none', label: 'None (0.0)' },
		{ value: 'low', label: 'Low (0.4)' },
		{ value: 'medium', label: 'Medium (0.7)' },
		{ value: 'high', label: 'High (1.0)' }
	];

	async function loadCommands() {
		try {
			commands = await invoke<AiCommand[]>('list_ai_commands');
		} catch (error) {
			console.error('Failed to load AI commands:', error);
		}
	}

	async function loadPlaceholders() {
		try {
			placeholders = await invoke<PlaceholderInfo[]>('get_available_placeholders');
		} catch (error) {
			console.error('Failed to load placeholders:', error);
		}
	}

	async function loadOllamaModels() {
		isLoadingModels = true;
		try {
			// Get AI settings to find Ollama base URL
			const settings = await invoke<{ baseUrl?: string }>('get_ai_settings');
			const baseUrl = settings.baseUrl || 'http://localhost:11434/v1';
			ollamaModels = await invoke<string[]>('get_ollama_models', { baseUrl });
		} catch (error) {
			console.error('Failed to load Ollama models:', error);
			ollamaModels = [];
		} finally {
			isLoadingModels = false;
		}
	}

	onMount(async () => {
		await Promise.all([loadCommands(), loadPlaceholders(), loadOllamaModels()]);
		isLoading = false;
	});

	function openNewForm() {
		editingCommand = null;
		formName = '';
		formPromptTemplate = '';
		formOutputAction = 'quick_ai';
		formCreativity = 'medium';
		formModel = null;
		showForm = true;
	}

	function openEditForm(command: AiCommand) {
		editingCommand = command;
		formName = command.name;
		formPromptTemplate = command.promptTemplate;
		formOutputAction = command.outputAction;
		formCreativity = command.creativity || 'medium';
		formModel = command.model || null;
		showForm = true;
	}

	function closeForm() {
		showForm = false;
		editingCommand = null;
	}

	async function saveCommand() {
		try {
			if (editingCommand) {
				await invoke('update_ai_command', {
					id: editingCommand.id,
					name: formName,
					promptTemplate: formPromptTemplate,
					outputAction: formOutputAction,
					creativity: formCreativity,
					model: formModel
				});
			} else {
				await invoke('create_ai_command', {
					name: formName,
					promptTemplate: formPromptTemplate,
					outputAction: formOutputAction,
					creativity: formCreativity,
					model: formModel
				});
			}
			await loadCommands();
			closeForm();
		} catch (error) {
			console.error('Failed to save AI command:', error);
		}
	}

	async function deleteCommand(id: string) {
		try {
			await invoke('delete_ai_command', { id });
			await loadCommands();
		} catch (error) {
			console.error('Failed to delete AI command:', error);
		}
	}

	function insertPlaceholder(placeholder: string) {
		formPromptTemplate += placeholder;
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center justify-between p-4">
		<div>
			<h2 class="text-lg font-semibold">AI Commands</h2>
			<p class="text-muted-foreground text-sm">
				Custom prompts with placeholders that appear in the command palette
			</p>
		</div>
		{#if !showForm}
			<Button size="sm" onclick={openNewForm}>
				<Plus class="mr-2 size-4" />
				New Command
			</Button>
		{/if}
	</div>

	{#if showForm}
		<!-- Form -->
		<div class="border-border/50 mx-4 flex-1 space-y-4 rounded-lg border p-4">
			<h3 class="font-medium">{editingCommand ? 'Edit Command' : 'New Command'}</h3>

			<div class="space-y-2">
				<label for="command-name" class="text-sm font-medium">Name</label>
				<Input id="command-name" bind:value={formName} placeholder="e.g., Explain Code" />
			</div>

			<div class="space-y-2">
				<label for="command-prompt" class="text-sm font-medium">Prompt Template</label>
				<textarea
					id="command-prompt"
					bind:value={formPromptTemplate}
					placeholder="Explain this code in simple terms: &#123;selection&#125;"
					class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex min-h-[120px] w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
				></textarea>
				<div class="flex flex-wrap gap-2">
					{#each placeholders as placeholder (placeholder.name)}
						<button
							onclick={() => insertPlaceholder(placeholder.name)}
							class="bg-muted hover:bg-muted/80 rounded-md px-2 py-1 text-xs transition-colors"
							title={placeholder.description}
						>
							{placeholder.name}
						</button>
					{/each}
				</div>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<span class="text-sm font-medium">Output Action</span>
					<div class="relative">
						<button
							type="button"
							onclick={() => (showOutputActionDropdown = !showOutputActionDropdown)}
							class="border-input bg-background flex h-10 w-full items-center justify-between rounded-md border px-3 py-2 text-sm"
						>
							{outputActionOptions.find((o) => o.value === formOutputAction)?.label || 'Select...'}
							<svg class="h-4 w-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M19 9l-7 7-7-7"
								/>
							</svg>
						</button>
						{#if showOutputActionDropdown}
							<div class="bg-popover absolute z-50 mt-1 w-full rounded-md border shadow-lg">
								{#each outputActionOptions as option (option.value)}
									<button
										type="button"
										class="hover:bg-accent w-full px-3 py-2 text-left text-sm {formOutputAction ===
										option.value
											? 'bg-accent'
											: ''}"
										onclick={() => {
											formOutputAction = option.value as OutputAction;
											showOutputActionDropdown = false;
										}}
									>
										{option.label}
									</button>
								{/each}
							</div>
						{/if}
					</div>
				</div>

				<div class="space-y-2">
					<span class="text-sm font-medium">Creativity</span>
					<div class="relative">
						<button
							type="button"
							onclick={() => (showCreativityDropdown = !showCreativityDropdown)}
							class="border-input bg-background flex h-10 w-full items-center justify-between rounded-md border px-3 py-2 text-sm"
						>
							{creativityOptions.find((o) => o.value === formCreativity)?.label || 'Select...'}
							<svg class="h-4 w-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M19 9l-7 7-7-7"
								/>
							</svg>
						</button>
						{#if showCreativityDropdown}
							<div class="bg-popover absolute z-50 mt-1 w-full rounded-md border shadow-lg">
								{#each creativityOptions as option (option.value)}
									<button
										type="button"
										class="hover:bg-accent w-full px-3 py-2 text-left text-sm {formCreativity ===
										option.value
											? 'bg-accent'
											: ''}"
										onclick={() => {
											formCreativity = option.value;
											showCreativityDropdown = false;
										}}
									>
										{option.label}
									</button>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>

			<!-- Model Picker -->
			<div class="space-y-2">
				<span class="text-sm font-medium">Model (Optional)</span>
				<div class="relative">
					<button
						type="button"
						onclick={() => (showModelDropdown = !showModelDropdown)}
						class="border-input bg-background flex h-10 w-full items-center justify-between rounded-md border px-3 py-2 text-sm"
					>
						{formModel || 'Use default model'}
						<svg class="h-4 w-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
							class="bg-popover absolute z-50 mt-1 max-h-48 w-full overflow-y-auto rounded-md border shadow-lg"
						>
							<button
								type="button"
								class="hover:bg-accent w-full px-3 py-2 text-left text-sm {!formModel
									? 'bg-accent'
									: ''}"
								onclick={() => {
									formModel = null;
									showModelDropdown = false;
								}}
							>
								Use default model
							</button>
							{#if isLoadingModels}
								<div class="text-muted-foreground px-3 py-2 text-sm">Loading models...</div>
							{:else if ollamaModels.length === 0}
								<div class="text-muted-foreground px-3 py-2 text-sm">No Ollama models found</div>
							{:else}
								{#each ollamaModels as model (model)}
									<button
										type="button"
										class="hover:bg-accent w-full px-3 py-2 text-left text-sm {formModel === model
											? 'bg-accent'
											: ''}"
										onclick={() => {
											formModel = model;
											showModelDropdown = false;
										}}
									>
										{model}
									</button>
								{/each}
							{/if}
						</div>
					{/if}
				</div>
			</div>

			<div class="flex justify-end gap-2 pt-4">
				<Button variant="outline" onclick={closeForm}>Cancel</Button>
				<Button onclick={saveCommand} disabled={!formName.trim() || !formPromptTemplate.trim()}>
					{editingCommand ? 'Save Changes' : 'Create Command'}
				</Button>
			</div>
		</div>
	{:else}
		<!-- Command List -->
		<div class="flex-1 overflow-y-auto px-4 pb-4">
			{#if isLoading}
				<div class="text-muted-foreground py-8 text-center">Loading...</div>
			{:else if commands.length === 0}
				<div class="text-muted-foreground flex flex-col items-center py-12 text-center">
					<Sparkles class="mb-4 size-12 opacity-50" />
					<p class="font-medium">No AI Commands yet</p>
					<p class="text-sm">Create custom prompts that appear in the command palette</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each commands as command (command.id)}
						<div
							class="border-border/50 hover:bg-muted/50 group flex items-center gap-3 rounded-lg border p-3 transition-colors"
						>
							<div class="bg-primary/10 rounded-lg p-2">
								<Sparkles class="text-primary size-4" />
							</div>
							<div class="min-w-0 flex-1">
								<div class="truncate font-medium">{command.name}</div>
								<div class="text-muted-foreground truncate text-sm">{command.promptTemplate}</div>
							</div>
							<div class="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
								<button
									onclick={() => openEditForm(command)}
									class="hover:bg-accent rounded p-1.5 transition-colors"
									title="Edit"
								>
									<Edit2 class="size-4" />
								</button>
								<button
									onclick={() => deleteCommand(command.id)}
									class="text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded p-1.5 transition-colors"
									title="Delete"
								>
									<Trash2 class="size-4" />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
