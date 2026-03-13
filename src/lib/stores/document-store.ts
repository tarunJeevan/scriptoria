// src/lib/stores/document-store.ts
//
// Owns backend-facing document state: which document is loaded, save lifecycle, and IPC calls via documentCommands.
// Intentionally decoupled from editorStore - the page component composes them together.
//
// Responsibility boundary:
//   documentStore → current DecryptedDocument, isSaving, saveError, lastSaved
//   editorStore   → Editor instance, selection, formatting state, isModified
//
// The only coupling point: documentStore.save() calls editorStore.markSaved() on success to clear the dirty flag.

import {
	documentCommands,
	type DecryptedDocument,
	type UpdateDocumentParams
} from '$lib/types/tauri-types';
import { derived, writable } from 'svelte/store';
import { editorStore } from './editor-store';

// -----------------------------------------------------------------------------
// State shape
// -----------------------------------------------------------------------------

interface DocumentStoreState {
	// The currently loaded document or NULL if none is open
	current: DecryptedDocument | null;
	// True while a save IPC call is in flight. Guards against concurrent saves
	isSaving: boolean;
	// Last error message from a failed save or NULL if no error
	saveError: string | null;
	// Timestamp of the last successful backend-confirmed save
	lastSaved: Date | null;
}

const initialState: DocumentStoreState = {
	current: null,
	isSaving: false,
	saveError: null,
	lastSaved: null
};

// -----------------------------------------------------------------------------
// Store factory
// -----------------------------------------------------------------------------

function createDocumentStore() {
	let _state: DocumentStoreState = initialState;
	const { subscribe, set, update } = writable<DocumentStoreState>(initialState);

	// Maintain a synchronous snapshot for use inside async functions
	subscribe((s) => {
		_state = s;
	});

	return {
		subscribe,

		/**
		 * Loads a document by ID from the backend and sets it as current.
		 * Resets all save lifecycle state. Does NOT set editor content - the page component owns that step via Editor.setContentSilent().
		 *
		 * @throws Re-throws IPC errors so the caller can handle loading failres.
		 */
		async load(documentId: number): Promise<DecryptedDocument> {
			const doc = await documentCommands.read(documentId);
			set({
				current: doc,
				isSaving: false,
				saveError: null,
				lastSaved: null
			});
			return doc;
		},

		/**
		 * Saves the provided content string to the backend for the current document.
		 * No-ops if no document is loaded or a save is already in flight.
		 * Calls editorStore.markSaved() on success.
		 *
		 * @param content - Tiptap JSON string from editor.getJSON()
		 * @param params  - Optional additional fields to update (title, metadata, etc.)
		 */
		async save(content: string, params: Omit<UpdateDocumentParams, 'content'> = {}): Promise<void> {
			// const state = get({ subscribe });

			if (!_state.current || _state.isSaving) return;

			update((s) => ({ ...s, isSaving: true, saveError: null }));

			try {
				const updated = await documentCommands.update(_state.current.id, {
					...params,
					content
				});

				update((s) => ({
					...s,
					current: updated,
					isSaving: false,
					lastSaved: new Date()
				}));

				// Clear dirty flag in editorStore - the only cross-store coupling point.
				editorStore.markSaved();
			} catch (err) {
				update((s) => ({
					...s,
					isSaving: false,
					saveError: err instanceof Error ? err.message : 'Save failed. Please try again.'
				}));
			}
		},

		/**
		 * Clears the current save error. Call from the status bar dismiss button.
		 */
		clearError(): void {
			update((s) => ({ ...s, saveError: null }));
		},

		/**
		 * Unloads the current document and resets all state.
		 * Call on route teardown (onDestroy in the page component).
		 */
		unload(): void {
			set(initialState);
		}
	};
}

export const documentStore = createDocumentStore();

// -----------------------------------------------------------------------------
// Serived stores
// -----------------------------------------------------------------------------

// True when a document is loaded and a save is in flight.
export const isSaving = derived(documentStore, ($s) => $s.isSaving);

// Current save error message or NULL.
export const saveError = derived(documentStore, ($s) => $s.saveError);

// Timestamp of the last backend-confirmed save.
export const lastSaved = derived(documentStore, ($s) => $s.lastSaved);

// True when a document is currently loaded.
export const isDocumentLoaded = derived(documentStore, ($s) => $s.current !== null);
