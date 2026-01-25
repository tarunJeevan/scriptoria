<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { invoke } from '@tauri-apps/api/core';

	const appVersion = '0.1.0';
	let tauriVersion = '';

	onMount(async () => {
		try {
			// Get Tauri version (example of IPC call)
			tauriVersion = await invoke<string>('tauri', {
				__tauriModule: 'App',
				message: { cmd: 'version' }
			}).catch(() => 'Tauri 2.x');
		} catch (error) {
			console.error('Failed to get Tauri version:', error);
			tauriVersion = 'Unknown';
		}
	});
</script>

<main>
	<div class="container">
		<h1>Scriptoria</h1>
		<p class="subtitle">AI-Enhanced Creative Writing Studio</p>

		<!-- NOTE: Button to go to Editor Demo -->
		<button class="editor-demo-button" onclick={() => goto(resolve('/editor-demo', {}))}
			>Go to Editor Demo</button
		>

		<div class="info-card">
			<h2>Phase 1: Chunk 1 Complete! 🎉</h2>
			<p>Project infrastructure is set up and running.</p>

			<div class="version-info">
				<div class="version-item">
					<span class="label">App Version:</span>
					<span class="value">{appVersion}</span>
				</div>
				<div class="version-item">
					<span class="label">Tauri Version:</span>
					<span class="value">{tauriVersion}</span>
				</div>
			</div>
		</div>

		<div class="next-steps">
			<h3>Next Steps:</h3>
			<ul>
				<li>
					<strong>Chunk 0:</strong> Database schema & encryption foundation
				</li>
				<li><strong>Chunk 2:</strong> Rich text editor (Tiptap)</li>
				<li><strong>Chunk 3:</strong> Document management backend</li>
			</ul>
		</div>

		<footer>
			<p>Local-first • Privacy-respecting • AI-powered</p>
		</footer>
	</div>
</main>

<style lang="postcss">
	@reference "tailwindcss";

	.editor-demo-button {
		@apply bg-blue-400 px-6 py-3 text-white border-2 border-blue-500 rounded-md text-base cursor-pointer transition-all duration-300 ease-in-out;
	}

	.editor-demo-button:hover {
		@apply bg-indigo-400 border-violet-500;
	}

	:global(body) {
		@apply m-0 p-0 font-sans text-gray-700 bg-linear-135 from-indigo-400 to-violet-500;
	}

	main {
		@apply flex justify-center items-center min-h-screen p-8;
	}

	.container {
		@apply bg-white rounded-2xl p-12 max-w-[600px] shadow-2xl;
	}

	h1 {
		@apply text-5xl mb-2 bg-linear-135 from-indigo-400 to-violet-500 bg-clip-text text-transparent;
	}

	.subtitle {
		@apply text-xl text-gray-500 mb-8;
	}

	.info-card {
		@apply bg-gray-100 rounded-xl p-6 mb-8;
	}

	.info-card h2 {
		@apply mb-4 text-2xl text-indigo-400;
	}

	.info-card p {
		@apply mb-4 text-gray-500;
	}

	.version-info {
		@apply flex flex-col gap-2;
	}

	.version-item {
		@apply flex justify-between p-2 bg-white rounded-md;
	}

	.label {
		@apply font-semibold text-gray-600;
	}

	.value {
		@apply font-mono text-indigo-400;
	}

	.next-steps {
		@apply bg-yellow-50 border-l-4 border-yellow-400 rounded-lg p-6 mb-8;
	}

	.next-steps h3 {
		@apply mb-4 text-orange-400;
	}

	.next-steps ul {
		@apply m-0 pl-6;
	}

	.next-steps li {
		@apply mb-2 text-gray-500;
	}

	.next-steps strong {
		@apply text-orange-300;
	}

	footer {
		@apply text-center pt-8 border-t border-gray-200;
	}

	footer p {
		@apply m-0 text-gray-400 text-sm;
	}
</style>
