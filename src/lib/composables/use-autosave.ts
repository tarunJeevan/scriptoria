// src/lib/composables/use-autosave.ts
//
// Svelte composable that watches editorStore.isModified and triggers documentStore.save() after a debounce delay.
// Designed to be called once from the document page's <script> block.
//
// Usage:
//   import { useAutosave } from '$lib/composables/use-autosave';
//   useAutosave(); // Call at component initialisation, not inside onMount

import { documentStore } from '$lib/stores/document-store';
import { editorStore } from '$lib/stores/editor-store';
import { onDestroy } from 'svelte';
import { get } from 'svelte/store';

/**
 * Watches editorStore.isModified and saves the current document content to the backend after `delayMs` of inactivity.
 *
 * Guards:
 * - Skips save if no document is loaded (documentStore.current is null)
 * - Skips save if a save is already in flight (documentStore.isSaving)
 * - Resets debounce timer on every content change
 * - Cleans up timer and store subscription on component destroy
 *
 * @param delayMs - Debounce delay in milliseconds (default: 1500)
 */
export function useAutosave(delayMs = 1500): void {
	let timer: ReturnType<typeof setTimeout> | null = null;

	const unsubscribe = editorStore.subscribe(($editor) => {
		// Only react to dirty state transitions
		if (!$editor.isModified) return;

		// Reset debounce on every content change
		if (timer) clearTimeout(timer);

		timer = setTimeout(async () => {
			const docState = get(documentStore);

			// Guard: no document loaded
			if (!docState.current) return;

			// Guard: save already in flight - the next isModified transition (triggered by editorStore.markSaved() after the in-flight save completes) will be false, so we won't loop.
			if (docState.isSaving) return;

			await documentStore.save($editor.content);
		}, delayMs);
	});

	onDestroy(() => {
		if (timer) clearTimeout(timer);
		unsubscribe();
	});
}
