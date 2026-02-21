<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { Plus, Edit2, Trash2, Sparkles, MessageCircle } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Switch } from '$lib/components/ui/switch';

	type AiPreset = {
		id: string;
		name: string;
		icon: string | null;
		model: string | null;
		temperature: number | null;
		systemPrompt: string | null;
		webSearch: boolean;
		createdAt: number;
		updatedAt: number;
	};

	let presets = $state<AiPreset[]>([]);
	let isLoading = $state(true);
	let showForm = $state(false);
	let editingPreset = $state<AiPreset | null>(null);

	// Form state
	let formName = $state('');
	let formSystemPrompt = $state('');
	let formModel = $state<string | null>(null);
	let formTemperature = $state(0.7);
	let formWebSearch = $state(false);
	let showModelDropdown = $state(false);

	// Available models
	let ollamaModels = $state<string[]>([]);
	let isLoadingModels = $state(false);

	async function loadPresets() {
		try {
			presets = await invoke<AiPreset[]>('list_ai_presets');
		} catch (error) {
			console.error('Failed to load AI presets:', error);
		}
	}

	async function loadOllamaModels() {
		isLoadingModels = true;
		try {
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
		await Promise.all([loadPresets(), loadOllamaModels()]);
		isLoading = false;
	});

	function openNewForm() {
		editingPreset = null;
		formName = '';
		formSystemPrompt = '';
		formModel = null;
		formTemperature = 0.7;
		formWebSearch = false;
		showForm = true;
	}

	function openEditForm(preset: AiPreset) {
		editingPreset = preset;
		formName = preset.name;
		formSystemPrompt = preset.systemPrompt || '';
		formModel = preset.model || null;
		formTemperature = preset.temperature ?? 0.7;
		formWebSearch = preset.webSearch;
		showForm = true;
	}

	function closeForm() {
		showForm = false;
		editingPreset = null;
	}

	async function savePreset() {
		try {
			if (editingPreset) {
				await invoke('update_ai_preset', {
					id: editingPreset.id,
					name: formName,
					systemPrompt: formSystemPrompt || null,
					model: formModel,
					temperature: formTemperature,
					webSearch: formWebSearch
				});
			} else {
				await invoke('create_ai_preset', {
					name: formName,
					systemPrompt: formSystemPrompt || null,
					model: formModel,
					temperature: formTemperature,
					webSearch: formWebSearch
				});
			}
			await loadPresets();
			closeForm();
		} catch (error) {
			console.error('Failed to save AI preset:', error);
		}
	}

	async function deletePreset(id: string) {
		try {
			await invoke('delete_ai_preset', { id });
			await loadPresets();
		} catch (error) {
			console.error('Failed to delete AI preset:', error);
		}
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center justify-between p-4">
		<div>
			<h2 class="text-lg font-semibold">AI Chat Presets</h2>
			<p class="text-muted-foreground text-sm">Saved configurations for different AI use cases</p>
		</div>
		{#if !showForm}
			<Button size="sm" onclick={openNewForm}>
				<Plus class="mr-2 size-4" />
				New Preset
			</Button>
		{/if}
	</div>

	{#if showForm}
		<!-- Form -->
		<div class="border-border/50 mx-4 flex-1 space-y-4 overflow-y-auto rounded-lg border p-4">
			<h3 class="font-medium">{editingPreset ? 'Edit Preset' : 'New Preset'}</h3>

			<div class="space-y-2">
				<label for="preset-name" class="text-sm font-medium">Name</label>
				<Input id="preset-name" bind:value={formName} placeholder="e.g., Code Review Assistant" />
			</div>

			<div class="space-y-2">
				<label for="system-prompt" class="text-sm font-medium">System Prompt</label>
				<textarea
					id="system-prompt"
					bind:value={formSystemPrompt}
					placeholder="You are a helpful code reviewer who focuses on best practices, performance, and security..."
					class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex min-h-[100px] w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
				></textarea>
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

			<!-- Temperature Slider -->
			<div class="space-y-2">
				<span class="text-sm font-medium">Temperature</span>
				<div class="flex items-center gap-4">
					<input
						type="range"
						min="0"
						max="1"
						step="0.1"
						bind:value={formTemperature}
						class="flex-1"
					/>
					<span class="w-12 text-right font-mono text-sm">{formTemperature.toFixed(1)}</span>
				</div>
				<p class="text-muted-foreground text-xs">
					0 = focused & deterministic, 1 = creative & varied
				</p>
			</div>

			<!-- Web Search Toggle -->
			<div class="flex items-center gap-2">
				<Switch bind:checked={formWebSearch} id="web-search" />
				<label for="web-search" class="text-sm font-medium">Enable Web Search</label>
			</div>

			<div class="flex justify-end gap-2 pt-4">
				<Button variant="outline" onclick={closeForm}>Cancel</Button>
				<Button onclick={savePreset} disabled={!formName.trim()}>
					{editingPreset ? 'Save Changes' : 'Create Preset'}
				</Button>
			</div>
		</div>
	{:else}
		<!-- Preset List -->
		<div class="flex-1 overflow-y-auto px-4 pb-4">
			{#if isLoading}
				<div class="text-muted-foreground py-8 text-center">Loading...</div>
			{:else if presets.length === 0}
				<div class="text-muted-foreground flex flex-col items-center py-12 text-center">
					<MessageCircle class="mb-4 size-12 opacity-50" />
					<p class="font-medium">No AI Presets yet</p>
					<p class="text-sm">
						Create presets for different AI use cases like Code Review, Writing, etc.
					</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each presets as preset (preset.id)}
						<div
							class="border-border/50 hover:bg-muted/50 group flex items-center gap-3 rounded-lg border p-3 transition-colors"
						>
							<div class="bg-primary/10 rounded-lg p-2">
								<Sparkles class="text-primary size-4" />
							</div>
							<div class="min-w-0 flex-1">
								<div class="truncate font-medium">{preset.name}</div>
								<div class="text-muted-foreground truncate text-sm">
									{preset.model || 'Default model'} • Temp: {preset.temperature?.toFixed(1) ??
										'0.7'}
									{#if preset.systemPrompt}• Has system prompt{/if}
								</div>
							</div>
							<div class="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
								<button
									onclick={() => openEditForm(preset)}
									class="hover:bg-accent rounded p-1.5 transition-colors"
									title="Edit"
								>
									<Edit2 class="size-4" />
								</button>
								<button
									onclick={() => deletePreset(preset.id)}
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
