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

<div class="flex-1 border border-gray-300 rounded-lg overflow-auto bg-white">
	<div bind:this={editorElement} class="editor-content focus:outline-none"></div>
</div>

<style>
	.editor-content :global(.ProseMirror:focus) {
		outline: none;
	}

	/* Placeholder styling */
	.editor-content :global(.ProseMirror p.is-editor-empty:first-child::before) {
		content: attr(data-placeholder);
		color: #99a1af;
		float: left;
		height: 0;
		pointer-events: none;
	}

	/* Selection styling */
	.editor-content :global(.ProseMirror-selectednode) {
		outline: 2px solid #3b82f6;
	}

	/* List styling */
	.editor-content :global(.ProseMirror ul),
	.editor-content :global(.ProseMirror ol) {
		padding-left: 1.5rem;
	}

	/* Link styling */
	.editor-content :global(.ProseMirror a) {
		color: #155dfc;
		text-decoration: underline;
	}

	/* Link styling while hovering */
	.editor-content :global(.ProseMirror a):hover {
		color: #193cb8;
	}

	/* Code block styling */
	.editor-content :global(.ProseMirror pre) {
		background-color: #f3f4f6;
		border-radius: 0.25rem;
		padding: 1rem;
		font-family:
			system-ui,
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			Roboto,
			Oxygen,
			Ubuntu,
			Cantarell,
			'Open Sans',
			'Helvetica Neue',
			sans-serif;
		font-size: 0.875rem;
		overflow: auto;
	}

	.editor-content :global(.ProseMirror code) {
		background-color: #f3f4f6;
		border-radius: 0.25rem;
		padding: 0 0.25rem;
		font-family:
			system-ui,
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			Roboto,
			Oxygen,
			Ubuntu,
			Cantarell,
			'Open Sans',
			'Helvetica Neue',
			sans-serif;
		font-size: 0.875rem;
	}

	/* Blockquote stylign */
	.editor-content :global(.ProseMirror blockquote) {
		border-left: 4px solid #d1dfdc;
		padding-left: 1rem;
		font-style: italic;
		color: #364153;
	}

	/* Heading styling */
	.editor-content :global(.ProseMirror h1) {
		font-size: 2.25rem;
		line-height: 2.5rem;
		font-weight: 700;
		margin-top: 1.5rem;
		margin-bottom: 1rem;
	}

	.editor-content :global(.ProseMirror h2) {
		font-size: 1.875rem;
		line-height: 2.25rem;
		font-weight: 700;
		margin-top: 1.25rem;
		margin-bottom: 0.75rem;
	}

	.editor-content :global(.ProseMirror h3) {
		font-size: 1.5rem;
		line-height: 2rem;
		font-weight: 700;
		margin-top: 1rem;
		margin-bottom: 0.5rem;
	}

	.editor-content :global(.ProseMirror h4) {
		font-size: 1.25rem;
		line-height: 1.75rem;
		font-weight: 700;
		margin-top: 0.75rem;
		margin-bottom: 0.5rem;
	}

	.editor-content :global(.ProseMirror h5) {
		font-size: 1.125rem;
		line-height: 1.75rem;
		font-weight: 700;
		margin-top: 0.5rem;
		margin-bottom: 0.25rem;
	}

	.editor-content :global(.ProseMirror h6) {
		font-size: 1rem;
		line-height: 1.5rem;
		font-weight: 700;
		margin-top: 0.5rem;
		margin-bottom: 0.25rem;
	}
</style>
