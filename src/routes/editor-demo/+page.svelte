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

	// Reactive class string for Save Button
	let saveButtonClass = $derived.by(() => {
		let baseClasses = 'px-4 py-2 rounded font-medium transition-all';
		if (saveStatus === 'saved') {
			baseClasses += 'bg-green-600 text-white';
		} else if (saveStatus === 'saving') {
			baseClasses += 'bg-gray-400 text-white cursor-wait';
		} else {
			baseClasses += 'bg-blue-600 text-white hover:bg-blue-700';
		}
		return baseClasses;
	});
</script>

<div class="flex flex-col h-screen bg-gray-50">
	<header class="flex items-center justify-between px-6 py-4 bg-white border-b border-gray-300">
		<h1 class="text-2xl font-bold text-gray-800">Scriptoria Editor Demo</h1>
		<div class="flex items-center gap-4">
			<div class="text-sm text-gray-600 flex items-center gap-2">
				<span>{$editorStats.words} words</span>
				<span class="text-gray-400">•</span>
				<span>{$editorStats.characters} characters</span>
			</div>
			<button class={saveButtonClass} onclick={saveDocument}>
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

	<div class="flex flex-col flex-1 overflow-hidden bg-white shadow-lg mx-6 my-4 rounded-lg">
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

	<footer class="px-6 py-3 bg-white border-t border-gray-300">
		<p class="text-sm text-gray-600">
			Tip: Use <kbd>Ctrl+B</kbd> for bold, <kbd>Ctrl+I</kbd> for italics, and <kbd>Ctrl+K</kbd> for links
		</p>
	</footer>
</div>

<style>
	kbd {
		padding: 0.25rem 0.5rem;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		font-size: 1.25rem;
		line-height: 1.4;
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
			monospace;
	}
</style>
