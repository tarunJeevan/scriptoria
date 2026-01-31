<script lang="ts">
	import Editor from '$lib/editor/Editor.svelte';
	import Toolbar from '$lib/editor/toolbar/Toolbar.svelte';
	import { editorStats } from '$lib/stores/editor-store';

	let editor: Editor;
	let lastSavedContent = '';
	let saveStatus: 'saved' | 'saving' | 'unsaved' = $state('saved');

	const initialContent = '<h1>Welcome to the Scriptoria Editor</h1>';

	function handleUpdate(content: string) {
		if (content !== lastSavedContent) {
			saveStatus = 'unsaved';
		}
	}

	function saveDocument() {
		saveStatus = 'saving';
		const content = editor?.getContent();

		// Simulate save delay
		setTimeout(() => {
			lastSavedContent = content;
			saveStatus = 'saved';
		}, 500);
	}

	// Autosave every 5 secs if modified
	let autoSaveInterval: number;
	$effect(() => {
		if (saveStatus === 'unsaved') {
			clearInterval(autoSaveInterval);
			autoSaveInterval = window.setInterval(() => {
				saveDocument();
			}, 5000);
		}
	});
</script>

<div class="editor-page">
	<header class="editor-header">
		<h1 class="editor-title">Scriptoria Editor Demo</h1>
		<div class="editor-actions">
			<div class="editor-stats">
				<span>{$editorStats.words} words</span>
				<span class="separator">•</span>
				<span>{$editorStats.characters} characters</span>
			</div>
			<button
				class="save-button"
				class:saved={saveStatus === 'saved'}
				class:saving={saveStatus === 'saving'}
				onclick={saveDocument}
			>
				{#if saveStatus === 'saved'}
					✓ Saved
				{:else if saveStatus === 'saving'}
					Saving...
				{:else}
					Save
				{/if}
			</button>
		</div>
	</header>

	<div class="editor-wrapper">
		<Toolbar />
		<Editor
			bind:this={editor}
			content={initialContent}
			placeholder="Start writing..."
			onUpdate={handleUpdate}
			autofocus
			editable
		/>
	</div>

	<footer class="editor-footer">
		<p class="hint">
			Tip: Use <kbd>Ctrl+B</kbd> for bold, <kbd>Ctrl+I</kbd> for italics, and <kbd>Ctrl+K</kbd> for links
		</p>
	</footer>
</div>

<style lang="postcss">
	@reference "tailwindcss";

	.editor-page {
		@apply flex flex-col h-screen bg-gray-50;
	}

	.editor-header {
		@apply flex items-center justify-between px-6 py-4 bg-white border-b border-gray-300;
	}

	.editor-title {
		@apply text-2xl font-bold text-gray-800;
	}

	.editor-actions {
		@apply flex items-center gap-4;
	}

	.editor-stats {
		@apply text-sm text-gray-600 flex items-center gap-2;
	}

	.separator {
		@apply text-gray-400;
	}

	.save-button {
		@apply px-4 py-2 rounded font-medium transition-all;
	}

	.save-button:not(.saved):not(.saving) {
		@apply bg-blue-600 text-white hover:bg-blue-700;
	}

	.save-button.saving {
		@apply bg-gray-400 text-white cursor-wait;
	}

	.save-button.saved {
		@apply bg-green-600 text-white;
	}

	.editor-wrapper {
		@apply flex flex-col flex-1 overflow-hidden bg-white shadow-lg mx-6 my-4 rounded-lg;
	}

	.editor-wrapper :global(.editor-container) {
		@apply flex-1 overflow-auto;
	}

	.editor-footer {
		@apply px-6 py-3 bg-white border-t border-gray-300;
	}

	.hint {
		@apply text-sm text-gray-600;
	}

	kbd {
		@apply px-2 py-1 bg-gray-200 rounded text-xl font-mono;
	}
</style>
