<!-- src/routes/documents/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		documentCommands,
		type DocumentListItem,
		type CreateDocumentParams
	} from '$lib/types/tauri-types';

	// Assumption: project_id = 1 for now. Project selection is a later chunk.
	const PROJECT_ID = 1;

	let documents: DocumentListItem[] = $state([]);
	let isLoading = $state(true);
	let loadError: string | null = $state(null);
	let isCreating = $state(false);
	let createError: string | null = $state(null);
	let newTitle = $state('');
	let showCreateForm = $state(false);

	// Load document list on mount
	onMount(async () => {
		await loadDocuments();
	});

	async function loadDocuments() {
		isLoading = true;
		loadError = null;
		try {
			documents = await documentCommands.list(PROJECT_ID);
		} catch (err) {
			loadError = err instanceof Error ? err.message : 'Failed to load documents.';
		} finally {
			isLoading = false;
		}
	}

	// Create document
	async function handleCreate() {
		if (!newTitle.trim()) return;

		isCreating = true;
		createError = null;

		try {
			const params: CreateDocumentParams = {
				project_id: PROJECT_ID,
				title: newTitle.trim(),
				content: '',
				doc_type: 'document'
			};
			const doc = await documentCommands.create(params);
			await goto(resolve(`/document/${doc.id}`, {}));
		} catch (err) {
			console.error('Create doc error: ', err);
			console.error('Error type: ', typeof err);
			console.error('Error JSON: ', JSON.stringify(err));
			createError =
				typeof err === 'string' ? err : err instanceof Error ? err.message : JSON.stringify(err);
			isCreating = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') handleCreate();
		if (e.key === 'Escape') {
			showCreateForm = false;
			newTitle = '';
			createError = null;
		}
	}

	// Helpers
	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString([], {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}
</script>

<div class="flex flex-col h-screen bg-gray-100">
	<!-- Header -->
	<header class="bg-white border-b border-gray-200 px-4 py-3 flex items-center gap-4">
		<button
			class="text-gray-500 hover:text-gray-700 transition-colors"
			onclick={() => goto(resolve('/', {}))}
		>
			← Back
		</button>
		<h1 class="text-lg font-semibold text-gray-800">Documents</h1>
		<div class="ml-auto">
			<button
				class="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700 transition-colors"
				onclick={() => {
					showCreateForm = true;
				}}
			>
				+ New Document
			</button>
		</div>
	</header>

	<!-- Main content -->
	<main class="flex-1 overflow-auto p-6">
		<div class="max-w-3xl mx-auto flex flex-col gap-4">
			<!-- Create document form -->
			{#if showCreateForm}
				<div class="bg-white rounded-xl shadow-sm border border-gray-200 p-4 flex flex-col gap-3">
					<p class="text-sm font-medium text-gray-700">New document title</p>
					<input
						type="text"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
						placeholder="Untitled document"
						bind:value={newTitle}
						onkeydown={handleKeydown}
					/>
					{#if createError}
						<p class="text-xs text-red-500">{createError}</p>
					{/if}
					<div class="flex gap-2 justify-end">
						<button
							class="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800 transition-colors"
							onclick={() => {
								showCreateForm = false;
								newTitle = '';
								createError = null;
							}}
						>
							Cancel
						</button>
						<button
							class="px-4 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
							onclick={handleCreate}
							disabled={isCreating || !newTitle.trim()}
						>
							<!-- {isCreating ? 'Creating...' : newTitle.trim()} -->
							Create
						</button>
					</div>
				</div>
			{/if}

			<!-- Load error -->
			{#if loadError}
				<div
					class="bg-red-50 border border-red-200 rounded-xl p-4 text-sm text-red-600 flex items-center justify-between"
				>
					<span>{loadError}</span>
					<button class="underline hover:text-red-800" onclick={loadDocuments}> Retry </button>
				</div>

				<!-- Loading state -->
			{:else if isLoading}
				<div class="text-center text-gray-400 text-sm py-12">Loading documents...</div>

				<!-- Empty state -->
			{:else if documents.length === 0}
				<div class="text-center text-gray-400 text-sm py-12 flex flex-col items-center gap-3">
					<p>No documents yet</p>
					<button
						class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm"
						onclick={() => {
							showCreateForm = true;
						}}
					>
						Create your first document
					</button>
				</div>

				<!-- Document list -->
			{:else}
				<ul class="flex flex-col gap-2">
					{#each documents as doc (doc.id)}
						<li>
							<button
								class="w-full bg-white rounded-xl shadow-sm border border-gray-200 px-4 py-3 flex items-center gap-4 hover:border-blue-300 hover:shadow-md transition-all text-left"
								onclick={() => goto(resolve(`/document/${doc.id}`, {}))}
							>
								<div class="flex-1 min-w-0">
									<p class="font-medium text-gray-800 truncate">
										{doc.title}
									</p>
									<p class="text-xs text-gray-400 mt-0.5">
										{doc.word_count ?? 0} words · Updated {formatDate(doc.updated_at)}
									</p>
								</div>
								<span class="text-gray-300 text-lg">→</span>
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</main>
</div>
