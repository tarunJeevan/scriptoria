import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { editorStore, editorStats, isEditorActive } from './editor-store';

describe('Editor Store', () => {
	beforeEach(() => {
		editorStore.reset();
	});

	it('should initialize with null editor', () => {
		const state = get(editorStore);
		expect(state.editor).toBeNull();
		expect(get(isEditorActive)).toBe(false);
	});

	it('should update content and mark as modified', () => {
		editorStore.setContent('New content');
		const state = get(editorStore);

		expect(state.content).toBe('New content');
		expect(state.isModified).toBe(true);
	});

	it('should mark as saved and update timestamp', () => {
		editorStore.setContent('Content');
		editorStore.markSaved();
		const state = get(editorStore);

		expect(state.isModified).toBe(false);
		expect(state.lastSaved).toBeInstanceOf(Date);
	});

	it('should provide editor stats', () => {
		editorStore.setContent('Test content');
		const stats = get(editorStats);

		expect(stats).toHaveProperty('words');
		expect(stats).toHaveProperty('characters');
		expect(stats).toHaveProperty('isModified');
		expect(stats).toHaveProperty('lastSaved');
	});
});
