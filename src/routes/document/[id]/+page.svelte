<!-- src/routes/document/[id]/+page.svelte -->
<!--
Document page - orchestrates document load/save and editor lifecycle.

Responsibilities:
- Load document on mount via documentStore.load(id)
- Pass loaded content to Editor via setContentSilent() (no dirty flag)
- Mount useAutosave() compoasble for debounced background saves
- Guard navigation away when editorStore.isModified is true
- Render a non-intrusive status bar reflecting save state
- Unload document on destroy to reset store state

This component only composes documentStore and editorStore and contains no business logic of its own.
-->
<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { beforeNavigate, goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Editor from '$lib/editor/Editor.svelte';
	import { documentStore, isSaving, saveError, lastSaved } from '$lib/stores/document-store';
	import { editorStore } from '$lib/stores/editor-store';
	import { useAutosave } from '$lib/composables/use-autosave';
	import type { PageData } from './$types';
	import Toolbar from '$lib/editor/toolbar/Toolbar.svelte';

	// Route param

	// document IDs are integers in the backend schema
	// const documentId = Number($page.params.id);
	const { data } = $props<{ data: PageData }>();
	const documentId = $derived(data.documentId);

	// Editor ref - used to call setContentSilent() after load
	let editorRef: ReturnType<typeof Editor> | null = $state(null);

	// Load state
	let isLoading = $state(true);
	let loadError: string | null = $state(null);
	let loadedContent: string | null = $state(null);

	// Lifecycle
	onMount(async () => {
		try {
			const doc = await documentStore.load(documentId);
			loadedContent = doc.content;
		} catch (err) {
			loadError = err instanceof Error ? err.message : 'Failed to load document.';
		} finally {
			isLoading = false;
			// Wait for Editor to finish mounting and editorRef to populate
			await tick();
			if (loadedContent) {
				// Inject content silently - `emitUpdateL false` prevents dirty flag on load.
				// The editor initialises with `content = ''` so this is always a valid call.
				editorRef?.setContentSilent(loadedContent);
			}
		}
	});

	onDestroy(() => {
		documentStore.unload();
	});

	// Autosave
	// Must be called at component initialisation (not inside onMount) so that onDestroy cleanup is registered correctly by Svelte.
	useAutosave(1500);

	// Dirty navigation guard

	beforeNavigate(({ cancel }) => {
		if ($editorStore.isModified) {
			const confirmed = confirm('You have unsaved changes that will be lost. Leave anyway?');
			if (!confirmed) cancel();
		}
	});

	// Status bar helpers

	function formatTime(date: Date): string {
		return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
	}
</script>

<!-- ========================================================================== -->
<!-- Main layout -->
<!-- ========================================================================== -->

{#if loadError}
	<div class="flex flex-col items-center justify-center h-full gap-3 text-red-600">
		<p class="font-medium">Failed to load document</p>
		<p class="text-sm text-gray-500">{loadError}</p>
	</div>
{:else if isLoading}
	<div class="flex items-center justify-center h-full text-gray-400 text-sm">
		Loading document...
	</div>
{:else}
	<div class="flex flex-col h-full">
		<!-- Back button -->
		<button
			class="text-xl text-gray-700 hover:text-black transition-colors"
			onclick={() => goto(resolve('/', {}))}
		>
			← Back
		</button>

		<!-- Editor fills available space -->
		<div class="flex-1 overflow-auto">
			<Toolbar />
			<Editor bind:this={editorRef} content="" editable={true} />
		</div>

		<!-- Status bar -->
		<div
			class="flex items-center gap-4 px-4 py-1.5 border-t border-gray-200 bg-gray-50 text-xs text-gray-500 select-none"
		>
			{#if $isSaving}
				<span class="flex items-center gap-1.5">
					<!-- Spinner -->
					<svg class="animate-spin h-3 w-3 text-gray-400" fill="none" viewBox="0 0 24 24">
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						/>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
					</svg>
					Saving...
				</span>
			{:else if $saveError}
				<span class="flex items-center gap-2 text-red-500">
					{$saveError}
					<button class="underline hover:text-red-700" onclick={() => documentStore.clearError()}>
						Dismiss
					</button>
				</span>
			{:else if $lastSaved}
				<span>Saved at {formatTime($lastSaved)}</span>
			{:else if $editorStore.isModified}
				<span>Unsaved changes</span>
			{:else}
				<span>All changes saved</span>
			{/if}

			<!-- Word count from editorStore -->
			<span class="ml-auto">
				{$editorStore.wordCount.toLocaleString()}
				{$editorStore.wordCount === 1 ? 'word' : 'words'}
			</span>
		</div>
	</div>
{/if}
