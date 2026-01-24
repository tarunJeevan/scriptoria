<!-- src/lib/editor/toolbar/ToolbarButton.svelte -->
<script lang="ts">
	const { children, active = false, disabled = false, title = '', onclick } = $props();
</script>

<button {onclick} class="toolbar-button" class:active {disabled} {title} type="button">
	{@render children()}
</button>

<style lang="postcss">
	@reference "tailwindcss";

	.toolbar-button {
		@apply p-2 rounded hover:bg-gray-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.toolbar-button.active {
		@apply bg-blue-100 text-blue-700;
	}

	.toolbar-button:not(.active):not(:disabled):hover {
		@apply bg-gray-200;
	}
</style>
