<script lang="ts">
	/* eslint-disable @typescript-eslint/no-unused-vars -- Props match common node interface */
	import type { UINode } from '$lib/types';

	type Props = {
		nodeId: number;
		uiTree: Map<number, UINode>;
		onDispatch: (instanceId: number, handlerName: string, args: unknown[]) => void;
	};

	let { nodeId, uiTree, onDispatch }: Props = $props();
</script>

<hr class="border-gray-200 dark:border-gray-700" />
