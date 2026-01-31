// src/lib/stores/editor-store.ts
import { writable, derived } from 'svelte/store';
import { Editor } from '@tiptap/core';
import type { Selection } from '@tiptap/pm/state';

interface EditorState {
	editor: Editor | null;
	content: string;
	selection: Selection | null;
	wordCount: number;
	charCount: number;
	isModified: boolean;
	lastSaved: Date | null;
}

const initialState: EditorState = {
	editor: null,
	content: '',
	selection: null,
	wordCount: 0,
	charCount: 0,
	isModified: false,
	lastSaved: null
};

function createEditorStore() {
	const { subscribe, set, update } = writable<EditorState>(initialState);

	return {
		subscribe,
		setEditor: (editor: Editor | null) => {
			update((state) => ({ ...state, editor }));
		},
		setContent: (content: string) => {
			update((state) => {
				const wordCount = state.editor?.storage.characterCount.words() || 0;
				const charCount = state.editor?.storage.characterCount.characters() || 0;

				return {
					...state,
					content,
					wordCount,
					charCount,
					isModified: true
				};
			});
		},
		setSelection: (selection: Selection | null) => {
			update((state) => ({ ...state, selection }));
		},
		markSaved: () => {
			update((state) => ({
				...state,
				isModified: false,
				lastSaved: new Date()
			}));
		},
		reset: () => set(initialState)
	};
}

export const editorStore = createEditorStore();

// Derived stores for common queries
export const isEditorActive = derived(editorStore, ($store) => $store.editor !== null);

export const editorStats = derived(editorStore, ($store) => ({
	words: $store.wordCount,
	characters: $store.charCount,
	isModified: $store.isModified,
	lastSaved: $store.lastSaved
}));

export const canUndo = derived(editorStore, ($store) => $store.editor?.can().undo() || false);

export const canRedo = derived(editorStore, ($store) => $store.editor?.can().redo() || false);

// Formatting state helpers
export const isTextFormatActive = derived(editorStore, ($store) => {
	const editor = $store.editor;
	if (!editor) return {};

	return {
		bold: editor.isActive('bold'),
		italic: editor.isActive('italic'),
		strike: editor.isActive('strike'),
		code: editor.isActive('code'),
		link: editor.isActive('link'),
		bulletList: editor.isActive('bulletList'),
		orderedList: editor.isActive('orderedList'),
		blockquote: editor.isActive('blockquote'),
		codeBlock: editor.isActive('codeBlock'),
		heading: {
			h1: editor.isActive('heading', { level: 1 }),
			h2: editor.isActive('heading', { level: 2 }),
			h3: editor.isActive('heading', { level: 3 }),
			h4: editor.isActive('heading', { level: 4 }),
			h5: editor.isActive('heading', { level: 5 }),
			h6: editor.isActive('heading', { level: 6 })
		}
	};
});
