<!-- src/lib/editor/toolbar/ToolbarButton.svelte -->
<script lang="ts">
	const { children, active = false, disabled = false, title = '', onclick } = $props();
</script>

<button
	{onclick}
	class={'p-2 rounded hover:bg-gray-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed' +
		(active ? 'bg-blue-100 text-blue-700' : 'bg-gray-200')}
	class:active
	{disabled}
	{title}
	type="button"
>
	{@render children()}
</button>
