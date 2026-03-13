// src/;ib/stores/document-store.test.ts

import { describe, it, expect, beforeEach, vi, type MockedFunction } from 'vitest';
import { get } from 'svelte/store';

// ------------------------------------------------------------------
// Mock Tauri IPC - must be hoisted before store imports
// ------------------------------------------------------------------

vi.mock('$lib/types/tauri-types', () => ({
	documentCommands: {
		read: vi.fn(),
		update: vi.fn()
	}
}));

vi.mock('$lib/stores/editor-store', () => ({
	editorStore: {
		markSaved: vi.fn(),
		subscribe: vi.fn(() => () => {})
	}
}));

import {
	documentStore,
	isSaving,
	saveError,
	lastSaved,
	isDocumentLoaded
} from '$lib/stores//document-store';
import { documentCommands } from '$lib/types/tauri-types';
import { editorStore } from '$lib/stores/editor-store';

// ------------------------------------------------------------------
// Fixtures
// ------------------------------------------------------------------

const mockDoc = {
	id: 1,
	project_id: 1,
	title: 'Test Document',
	content: JSON.stringify({
		type: 'doc',
		content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Hello world' }] }]
	}),
	word_count: 2,
	char_count: 11,
	created_at: '2025-01-01T00:00:00Z',
	updated_at: '20215-01-01T00:00:00Z',
	last_edited_at: null,
	doc_type: 'document',
	entity_type: null,
	parent_id: null,
	display_order: 0,
	deleted: false,
	metadata: { tags: [] }
};

const updatedDoc = {
	...mockDoc,
	content: JSON.stringify({ type: 'doc', content: [] }),
	updated_at: '2025-01-02T00:00:00Z'
};

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

describe('documentStore', () => {
	beforeEach(() => {
		documentStore.unload();
		vi.clearAllMocks();
	});

	// Initial state

	it('initialises with null current document', () => {
		const state = get(documentStore);
		expect(state.current).toBeNull();
		expect(state.isSaving).toBe(false);
		expect(state.saveError).toBeNull();
		expect(state.lastSaved).toBeNull();
	});

	it('isDocumentLoaded is false initially', () => {
		expect(get(isDocumentLoaded)).toBe(false);
	});

	// load()

	it('load() sets current document on success', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);

		await documentStore.load(1);

		const state = get(documentStore);
		expect(state.current).toEqual(mockDoc);
		expect(state.isSaving).toBe(false);
		expect(state.saveError).toBeNull();
		expect(get(isDocumentLoaded)).toBe(true);
	});

	it('load() returns the loaded document', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);

		const result = await documentStore.load(1);
		expect(result).toEqual(mockDoc);
	});

	it('load() re-throws in IPC failure', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockRejectedValue(
			new Error('IPC error')
		);

		await expect(documentStore.load(1)).rejects.toThrow('IPC error');
	});

	// save()

	it('save() updates current document and clears dirty flag on success', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);
		(documentCommands.update as MockedFunction<typeof documentCommands.update>).mockResolvedValue(
			updatedDoc
		);

		await documentStore.load(1);
		await documentStore.save(updatedDoc.content);

		const state = get(documentStore);
		expect(state.current).toEqual(updatedDoc);
		expect(state.isSaving).toBe(false);
		expect(state.lastSaved).toBeInstanceOf(Date);
		expect(editorStore.markSaved).toHaveBeenCalledOnce();
	});

	it('save() sets saveError on IPC failure', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);
		(documentCommands.update as MockedFunction<typeof documentCommands.update>).mockRejectedValue(
			new Error('Write failed')
		);

		await documentStore.load(1);
		await documentStore.save('content');

		const state = get(documentStore);
		expect(state.saveError).toBe('Write failed');
		expect(state.isSaving).toBe(false);
		expect(editorStore.markSaved).not.toHaveBeenCalled();
	});

	it('save() is a no-op when no document is loaded', async () => {
		await documentStore.save('content');
		expect(documentCommands.update).not.toHaveBeenCalled();
	});

	it('save() is a no-op when a save is already in flight', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);

		// Make update hang so isSaving stays true
		let resolveUpdate!: (v: typeof updatedDoc) => void;
		(documentCommands.update as MockedFunction<typeof documentCommands.update>).mockReturnValue(
			new Promise((r) => {
				resolveUpdate = r;
			})
		);

		await documentStore.load(1);

		// Fire first save - hangs
		const first = documentStore.save('content');

		// Second save while first is in flight - should no-op
		await documentStore.save('content');

		expect(documentCommands.update).toHaveBeenCalledTimes(1);

		// Clean up
		resolveUpdate(updatedDoc);
		await first;
	});

	// clearError()

	it('clearError() nulls out saveError', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);
		(documentCommands.update as MockedFunction<typeof documentCommands.update>).mockRejectedValue(
			new Error('fail')
		);

		await documentStore.load(1);
		await documentStore.save('content');

		expect(get(saveError)).not.toBeNull();

		documentStore.clearError();
		expect(get(saveError)).toBeNull();
	});

	// unload()

	it('unload() resets all state', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);
		await documentStore.load(1);

		documentStore.unload();

		const state = get(documentStore);
		expect(state.current).toBeNull();
		expect(state.isSaving).toBe(false);
		expect(state.saveError).toBeNull();
		expect(state.lastSaved).toBeNull();
	});

	// Derived stores

	it('isSaving derived store reflects in-flight state', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);

		let resolveUpdate!: (v: typeof updatedDoc) => void;
		(documentCommands.update as MockedFunction<typeof documentCommands.update>).mockReturnValue(
			new Promise((r) => {
				resolveUpdate = r;
			})
		);

		await documentStore.load(1);
		expect(get(isSaving)).toBe(false);

		const savePromise = documentStore.save('content');
		expect(get(isSaving)).toBe(true);

		resolveUpdate(updatedDoc);
		await savePromise;
		expect(get(isSaving)).toBe(false);
	});

	it('lastSaved updates after successful save', async () => {
		(documentCommands.read as MockedFunction<typeof documentCommands.read>).mockResolvedValue(
			mockDoc
		);
		(documentCommands.update as MockedFunction<typeof documentCommands.update>).mockResolvedValue(
			updatedDoc
		);

		expect(get(lastSaved)).toBeNull();

		await documentStore.load(1);
		await documentStore.save('content');

		expect(get(lastSaved)).toBeInstanceOf(Date);
	});
});
