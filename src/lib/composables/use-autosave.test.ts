// src/lib/composables/use-autosave.test.ts

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { writable } from 'svelte/store';

// ------------------------------------------------------------------
// Mocks
// ------------------------------------------------------------------

// We mock both stores so this test has no external dependencies.
// editorStore is mocked as a writable so we can trigger subscription updates.

const mockEditorState = writable({
	isModified: false,
	content: ''
});

vi.mock('$lib/stores/editor-store', () => ({
	editorStore: {
		subscribe: (fn: Parameters<typeof mockEditorState.subscribe>[0]) =>
			mockEditorState.subscribe(fn)
	}
}));

const mockSave = vi.fn().mockResolvedValue(undefined);
const mockDocState = writable<{ current: { id: number } | null; isSaving: boolean }>({
	current: { id: 1 },
	isSaving: false
});

vi.mock('$lib/stores/document-store', () => ({
	documentStore: {
		save: (...args: unknown[]) => mockSave(...args),
		subscribe: (fn: Parameters<typeof mockDocState.subscribe>[0]) => mockDocState.subscribe(fn)
	}
}));

// Stub onDestroy - in tests there's no Svelte component context.
// We capture the cleanup callback so we can call it manually.
let capturedOnDestroy: (() => void) | null = null;
vi.mock('svelte', () => ({
	onDestroy: (fn: () => void) => {
		capturedOnDestroy = fn;
	}
}));

import { useAutosave } from './use-autosave';

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

describe('useAutosave', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		mockSave.mockClear();
		capturedOnDestroy = null;
		mockEditorState.set({ isModified: false, content: '' });
		mockDocState.set({ current: { id: 1 }, isSaving: false });
	});

	afterEach(() => {
		vi.useRealTimers();
		capturedOnDestroy?.();
	});

	it('does not save immediately when isModified becomes true', () => {
		useAutosave(1500);
		mockEditorState.set({ isModified: true, content: 'hello' });

		expect(mockSave).not.toHaveBeenCalled();
	});

	it('saves after the debounce delay', async () => {
		useAutosave(1500);
		mockEditorState.set({ isModified: true, content: 'hello' });

		vi.advanceTimersByTime(1500);
		await Promise.resolve(); // flush async save

		expect(mockSave).toHaveBeenCalledOnce();
		expect(mockSave).toHaveBeenCalledWith('hello');
	});

	it('debounces - only saves once after rapid edits', async () => {
		useAutosave(1500);

		mockEditorState.set({ isModified: true, content: 'h' });
		vi.advanceTimersByTime(500);
		mockEditorState.set({ isModified: true, content: 'he' });
		vi.advanceTimersByTime(500);
		mockEditorState.set({ isModified: true, content: 'hel' });
		vi.advanceTimersByTime(1500);
		await Promise.resolve();

		expect(mockSave).toHaveBeenCalledOnce();
		expect(mockSave).toHaveBeenCalledWith('hel');
	});

	it('does not save when no document is loaded', async () => {
		mockDocState.set({ current: null, isSaving: false });

		useAutosave(1500);
		mockEditorState.set({ isModified: true, content: 'hello' });

		vi.advanceTimersByTime(1500);
		await Promise.resolve();

		expect(mockSave).not.toHaveBeenCalled();
	});

	it('does not save when a save is already in flight', async () => {
		mockDocState.set({ current: { id: 1 }, isSaving: true });

		useAutosave(1500);
		mockEditorState.set({ isModified: true, content: 'hello' });

		vi.advanceTimersByTime(1500);
		await Promise.resolve();

		expect(mockSave).not.toHaveBeenCalled();
	});

	it('does not trigger when isModified is false', async () => {
		useAutosave(1500);
		mockEditorState.set({ isModified: false, content: 'hello' });

		vi.advanceTimersByTime(1500);
		await Promise.resolve();

		expect(mockSave).not.toHaveBeenCalled();
	});

	it('cleans up timer on destroy', async () => {
		useAutosave(1500);
		mockEditorState.set({ isModified: true, content: 'hello' });

		// Destroy before debounce fires
		capturedOnDestroy?.();
		vi.advanceTimersByTime(1500);
		await Promise.resolve();

		expect(mockSave).not.toHaveBeenCalled();
	});
});
