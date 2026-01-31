<!-- src/lib/editor/toolbar/Toolbar.svelte -->
<script lang="ts">
	import { editorStore, isTextFormatActive, canUndo, canRedo } from '$lib/stores/editor-store';
	import { tick } from 'svelte';
	import ToolbarButton from './ToolbarButton.svelte';
	import ToolbarDivider from './ToolbarDivider.svelte';

	let showLinkDialog = $state(false);
	let linkUrl = $state('');

	// Focus management
	let inputElement: HTMLInputElement | null = $state(null);
	let prevActiveElement: HTMLElement | null = $state(null);

	// When dialog opens, focus input after DOM update
	$effect(() => {
		if (showLinkDialog) {
			prevActiveElement = document.activeElement as HTMLElement | null;
			tick().then(() => {
				// Focus the first focusable control inside the dialog
				inputElement?.focus();
			});
		}
	});

	function toggleBold() {
		$editorStore.editor?.chain().focus().toggleBold().run();
	}

	function toggleItalic() {
		$editorStore.editor?.chain().focus().toggleItalic().run();
	}

	function toggleStrike() {
		$editorStore.editor?.chain().focus().toggleStrike().run();
	}

	function toggleCode() {
		$editorStore.editor?.chain().focus().toggleCode().run();
	}

	function toggleBulletList() {
		$editorStore.editor?.chain().focus().toggleBulletList().run();
	}

	function toggleOrderedList() {
		$editorStore.editor?.chain().focus().toggleOrderedList().run();
	}

	function toggleBlockquote() {
		$editorStore.editor?.chain().focus().toggleBlockquote().run();
	}

	function toggleCodeBlock() {
		$editorStore.editor?.chain().focus().toggleCodeBlock().run();
	}

	function setHeadingLevel(level: 1 | 2 | 3 | 4 | 5 | 6) {
		$editorStore.editor?.chain().focus().toggleHeading({ level }).run();
	}

	function setParagraph() {
		$editorStore.editor?.chain().focus().setParagraph().run();
	}

	function undo() {
		$editorStore.editor?.chain().focus().undo().run();
	}

	function redo() {
		$editorStore.editor?.chain().focus().redo().run();
	}

	function openLinkDialog() {
		const previousUrl = $editorStore.editor?.getAttributes('link').href || '';
		linkUrl = previousUrl;
		showLinkDialog = true;
	}

	function setLink() {
		if (linkUrl === '') {
			$editorStore.editor?.chain().focus().extendMarkRange('link').unsetLink().run();
		} else {
			$editorStore.editor?.chain().focus().extendMarkRange('link').setLink({ href: linkUrl }).run();
		}
		closeDialog();
		linkUrl = '';
	}

	function closeDialog() {
		showLinkDialog = false;
		// Restore focus to the previous focused element
		prevActiveElement?.focus();
	}

	function removeLink() {
		$editorStore.editor?.chain().focus().unsetLink().run();
		closeDialog();
	}
</script>

<div class="toolbar">
	<!-- Undo/Redo -->
	<ToolbarButton onclick={undo} disabled={!$canUndo} title="Undo (Ctrl+z)">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M3 7v6h6" />
			<path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />
		</svg>
	</ToolbarButton>

	<ToolbarButton onclick={redo} disabled={!$canRedo} title="Redo (Ctrl+y)">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M21 7v6h-6" />
			<path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7" />
		</svg>
	</ToolbarButton>

	<ToolbarDivider />

	<!-- Heading Dropdown -->
	<select
		class="heading-select"
		onchange={(e) => {
			const value = e.currentTarget.value;
			if (value === 'p') {
				setParagraph();
			} else {
				const level = parseInt(value.replace('h', '')) as 1 | 2 | 3 | 4 | 5 | 6;
				setHeadingLevel(level);
			}
		}}
		value={$isTextFormatActive.heading?.h1
			? 'h1'
			: $isTextFormatActive.heading?.h2
				? 'h2'
				: $isTextFormatActive.heading?.h3
					? 'h3'
					: $isTextFormatActive.heading?.h4
						? 'h4'
						: $isTextFormatActive.heading?.h5
							? 'h5'
							: $isTextFormatActive.heading?.h6
								? 'h6'
								: 'p'}
	>
		<option value="p">Paragraph</option>
		<option value="h1">Heading 1</option>
		<option value="h2">Heading 2</option>
		<option value="h3">Heading 3</option>
		<option value="h4">Heading 4</option>
		<option value="h5">Heading 5</option>
		<option value="h6">Heading 6</option>
	</select>

	<ToolbarDivider />

	<!-- Text Formatting -->
	<ToolbarButton onclick={toggleBold} active={$isTextFormatActive.bold} title="Bold (Ctrl+b)">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
			<path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
		</svg>
	</ToolbarButton>

	<ToolbarButton onclick={toggleItalic} active={$isTextFormatActive.italic} title="Italic (Ctrl+i)">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<line x1="19" y1="4" x2="10" y2="4" />
			<line x1="14" y1="20" x2="5" y2="20" />
			<line x1="15" y1="4" x2="9" y2="20" />
		</svg>
	</ToolbarButton>

	<ToolbarButton onclick={toggleStrike} active={$isTextFormatActive.strike} title="Strikethrough">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M16 4H9a3 3 0 0 0-2.83 4" />
			<path d="M14 12a4 4 0 0 1 0 8H6" />
			<line x1="4" y1="12" x2="20" y2="12" />
		</svg>
	</ToolbarButton>

	<ToolbarButton onclick={toggleCode} active={$isTextFormatActive.code} title="Inline Code">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<polyline points="16 18 22 12 16 6" />
			<polyline points="8 6 2 12 8 18" />
		</svg>
	</ToolbarButton>

	<ToolbarDivider />

	<!-- Lists -->
	<ToolbarButton
		onclick={toggleBulletList}
		active={$isTextFormatActive.bulletList}
		title="Bullet List"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<line x1="8" y1="6" x2="21" y2="6" />
			<line x1="8" y1="12" x2="21" y2="12" />
			<line x1="8" y1="18" x2="21" y2="18" />
			<line x1="3" y1="6" x2="3.01" y2="6" />
			<line x1="3" y1="12" x2="3.01" y2="12" />
			<line x1="3" y1="18" x2="3.01" y2="18" />
		</svg>
	</ToolbarButton>

	<ToolbarButton
		onclick={toggleOrderedList}
		active={$isTextFormatActive.orderedList}
		title="Numbered List"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<line x1="10" y1="6" x2="21" y2="6" />
			<line x1="10" y1="12" x2="21" y2="12" />
			<line x1="10" y1="18" x2="21" y2="18" />
			<path d="M4 6h1v4" />
			<path d="M4 10h2" />
			<path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" />
		</svg>
	</ToolbarButton>

	<ToolbarDivider />

	<!-- Block Types -->
	<ToolbarButton
		onclick={toggleBlockquote}
		active={$isTextFormatActive.blockquote}
		title="Blockquote"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path
				d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"
			/>
			<path
				d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3c0 1 0 1 1 1z"
			/>
		</svg>
	</ToolbarButton>

	<ToolbarButton
		onclick={toggleCodeBlock}
		active={$isTextFormatActive.codeBlock}
		title="Code Block"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<rect x="2" y="3" width="20" height="14" rx="2" />
			<path d="m8 9-3 3 3 3" />
			<path d="m16 9 3 3-3 3" />
			<line x1="2" y1="21" x2="22" y2="21" />
		</svg>
	</ToolbarButton>

	<ToolbarDivider />

	<!-- Link -->
	<ToolbarButton
		onclick={openLinkDialog}
		active={$isTextFormatActive.link}
		title="Insert Link (Ctrl+k)"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
			<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
		</svg>
	</ToolbarButton>

	<!-- Link Dialog -->
	{#if showLinkDialog}
		<div class="dialog-overlay" onclick={() => closeDialog()} role="presentation">
			<div class="dialog" onclick={(e) => e.stopPropagation()} role="presentation">
				<h3 class="dialog-title">Insert Link</h3>
				<input
					type="url"
					bind:value={linkUrl}
					bind:this={inputElement}
					placeholder="https://example.com"
					class="dialog-input"
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							setLink();
						} else if (e.key === 'Escape') {
							closeDialog();
						}
					}}
				/>
				<div class="dialog-actions">
					<button class="dialog-button dialog-button-secondary" onclick={removeLink}>
						Remove Link
					</button>
					<button class="dialog-button dialog-button-primary" onclick={setLink}> Insert </button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style lang="postcss">
	@reference "tailwindcss";

	.toolbar {
		@apply flex items-center gap-1 p-2 border-b border-gray-300 bg-gray-50 flex-wrap;
	}

	.heading-select {
		@apply px-3 py-1 border border-gray-300 rounded bg-white text-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500;
	}

	/* Dialog Styles */
	.dialog-overlay {
		@apply fixed inset-0 bg-black opacity-50 flex items-center justify-center z-50;
	}

	.dialog {
		@apply bg-white rounded-lg shadow-xl p-6 w-96 max-w-full mx-4;
	}

	.dialog-title {
		@apply text-lg font-semibold mb-4;
	}

	.dialog-input {
		@apply w-full px-3 py-2 border border-gray-300 rounded mb-4 focus:outline-none focus:ring-2 focus:ring-blue-500;
	}

	.dialog-actions {
		@apply flex justify-end gap-2;
	}

	.dialog-button {
		@apply px-4 py-2 rounded font-medium transition-colors;
	}

	.dialog-button-primary {
		@apply bg-blue-600 text-white hover:bg-blue-700;
	}

	.dialog-button-secondary {
		@apply bg-gray-200 text-gray-800 hover:bg-gray-300;
	}
</style>
