<!-- src/lib/editor/Editor.svelte -->
<script lang="ts">
	// imports
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Link from '@tiptap/extension-link';
	import Placeholder from '@tiptap/extension-placeholder';
	import CharacterCount from '@tiptap/extension-character-count';
	import Typography from '@tiptap/extension-typography';
	import { editorStore } from '$lib/stores/editor-store';

	interface EditorProps {
		content: string;
		editable: boolean;
		placeholder: string;
		onUpdate: ((content: string) => void) | undefined;
		autofocus: boolean;
	}

	// Props
	const {
		content = '',
		editable = true,
		placeholder = 'Start writing...',
		onUpdate = undefined,
		autofocus = false
	}: EditorProps = $props();

	let editor: Editor | null = $state(null);
	let editorElement: HTMLDivElement | null = $state(null);

	// Instantiation and cleanup
	onMount(() => {
		editor = new Editor({
			element: editorElement,
			extensions: [
				StarterKit.configure({
					// heading levels 1-6 are default but are customizable here
					heading: {
						levels: [1, 2, 3, 4, 5, 6]
					},
					// Default unordered list attributes. Customizable here
					bulletList: {
						keepMarks: true,
						keepAttributes: false
					},
					// Code block attributes. Customizable here
					codeBlock: {
						// Allows indentation via Tab within code block
						enableTabIndentation: true,
						tabSize: 4
					},
					// Default ordered list attributes. Customizable here
					orderedList: {
						keepMarks: true,
						keepAttributes: false
					},
					dropcursor: {
						color: 'red'
					}
				}),
				Link.configure({
					openOnClick: false,
					linkOnPaste: true,
					HTMLAttributes: {
						// Customize link appeance here
						class: 'text-blue-600 underline hover:text-blue-800'
					}
				}),
				Placeholder.configure({
					placeholder
				}),
				CharacterCount,
				Typography
			],
			content,
			editable,
			autofocus,
			onUpdate: ({ editor }) => {
				const html = editor.getHTML();
				editorStore.setContent(html);

				if (onUpdate) {
					onUpdate(html);
				}
			},
			onSelectionUpdate: ({ editor }) => {
				editorStore.setSelection(editor.state.selection);
			},
			editorProps: {
				attributes: {
					class:
						'prose prose-sm sm:prose lg:prose-lg xl:prose-xl focus:outline-none min-h-[500px] max-w-none p-4'
				}
			}
		});

		// Store editor instance globally for toolbar access
		editorStore.setEditor(editor);
	});

	onDestroy(() => {
		if (editor) {
			editor.destroy();
			editorStore.setEditor(null);
		}
	});

	// Export methods for parent components
	export function getContent(): string {
		return editor?.getHTML() || '';
	}

	export function setContent(newContent: string) {
		editor?.commands.setContent(newContent);
	}

	export function focus() {
		editor?.commands.focus();
	}

	export function getWordCount(): number {
		return editor?.storage.characterCount.words() || 0;
	}

	export function getCharCount(): number {
		return editor?.storage.characterCount.characters() || 0;
	}
</script>

<div class="editor-container">
	<div bind:this={editorElement} class="editor-content"></div>
</div>

<style lang="postcss">
	@reference "tailwindcss";

	.editor-container {
		@apply border border-gray-300 rounded-lg overflow-hidden bg-white;
	}

	.editor-content :global(.ProseMirror) {
		@apply focus:outline-none;
	}

	/* Placeholder styling */
	.editor-content :global(.ProseMirror p.is-editor-empty:first-child::before) {
		@apply text-gray-400;
		content: attr(data-placeholder);
		float: left;
		height: 0;
		pointer-events: none;
	}

	/* Selection styling */
	.editor-content :global(.ProseMirror-selectednode) {
		@apply outline-2 outline-blue-500;
	}

	/* List styling */
	.editor-content :global(.ProseMirror ul),
	.editor-content :global(.ProseMirror ol) {
		@apply pl-6;
	}

	/* Link styling */
	.editor-content :global(.ProseMirror a) {
		@apply text-blue-600 underline hover:text-blue-800;
	}

	/* Code block styling */
	.editor-content :global(.ProseMirror pre) {
		@apply bg-gray-100 rounded p-4 font-mono text-sm;
	}

	.editor-content :global(.ProseMirror code) {
		@apply bg-gray-100 rounded px-1 font-mono text-sm;
	}

	/* Blockquote stylign */
	.editor-content :global(.ProseMirror blockquote) {
		@apply border-l-4 border-gray-300 pl-4 italic text-gray-700;
	}

	/* Heading styling */
	.editor-content :global(.ProseMirror h1) {
		@apply text-4xl font-bold mt-6 mb-4;
	}

	.editor-content :global(.ProseMirror h2) {
		@apply text-3xl font-bold mt-5 mb-3;
	}

	.editor-content :global(.ProseMirror h3) {
		@apply text-2xl font-bold mt-4 mb-2;
	}

	.editor-content :global(.ProseMirror h4) {
		@apply text-xl font-bold mt-3 mb-2;
	}

	.editor-content :global(.ProseMirror h5) {
		@apply text-lg font-bold mt-2 mb-1;
	}

	.editor-content :global(.ProseMirror h6) {
		@apply text-base font-bold mt-2 mb-1;
	}
</style>
