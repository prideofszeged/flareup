<script lang="ts">
	import AiSettingsView from '../AiSettingsView.svelte';
	import AiCommandsSettings from './AiCommandsSettings.svelte';
	import AiPresetsSettings from './AiPresetsSettings.svelte';
	import { Settings, Wand2, Stars, Wrench } from '@lucide/svelte';

	type AiSection = 'general' | 'commands' | 'presets' | 'tools';
	let activeSection = $state<AiSection>('general');

	const sections = [
		{ id: 'general', label: 'General', icon: Settings },
		{ id: 'commands', label: 'AI Commands', icon: Wand2 },
		{ id: 'presets', label: 'Presets', icon: Stars }
	] as const;
</script>

<div class="flex h-full">
	<!-- Sidebar -->
	<div class="border-border/50 w-48 shrink-0 border-r p-3">
		<h2 class="text-muted-foreground mb-3 text-xs font-semibold tracking-wider uppercase">
			AI Settings
		</h2>
		<nav class="space-y-1">
			{#each sections as section}
				{@const IconComponent = section.icon}
				<button
					type="button"
					onclick={() => (activeSection = section.id)}
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors
						{activeSection === section.id
						? 'bg-accent text-accent-foreground'
						: 'hover:bg-accent/50 text-muted-foreground'}"
				>
					<IconComponent class="size-4" />
					{section.label}
				</button>
			{/each}
		</nav>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto">
		{#if activeSection === 'general'}
			<AiSettingsView />
		{:else if activeSection === 'commands'}
			<AiCommandsSettings />
		{:else if activeSection === 'presets'}
			<AiPresetsSettings />
		{/if}
	</div>
</div>
