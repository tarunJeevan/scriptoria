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

<main class="flex justify-center items-center min-h-screen p-8">
	<div class="bg-white rounded-2xl p-12 max-w-150 shadow-2xl">
		<h1
			class="text-5xl mb-2 bg-linear-135 from-indigo-400 to-violet-500 bg-clip-text text-transparent"
		>
			Scriptoria
		</h1>
		<p class="text-xl text-gray-500 mb-8">AI-Enhanced Creative Writing Studio</p>

		<!-- NOTE: Button to go to Editor Demo -->
		<button
			class="bg-blue-400 px-6 py-3 text-white border-2 border-blue-500 rounded-md text-base cursor-pointer transition-all duration-300 ease-in-out hover:bg-indigo-400 hover:border-violet-500"
			onclick={() => goto(resolve('/editor-demo', {}))}
		>
			Go to Editor Demo
		</button>

		<div class="bg-gray-100 rounded-xl p-6 mb-8">
			<h2 class="mb-4 text-2xl text-indigo-400">Phase 1: Chunk 1 Complete! 🎉</h2>
			<p class="mb-4 text-gray-500">Project infrastructure is set up and running.</p>

			<div class="flex flex-col gap-2">
				<div class="flex justify-between p-2 bg-white rounded-md">
					<span class="font-semibold text-gray-600">App Version:</span>
					<span class="font-mono text-indigo-400">{appVersion}</span>
				</div>
				<div class="flex justify-between p-2 bg-white rounded-md">
					<span class="font-semibold text-gray-600">Tauri Version:</span>
					<span class="font-mono text-indigo-400">{tauriVersion}</span>
				</div>
			</div>
		</div>

		<div class="bg-yellow-50 border-l-4 border-yellow-400 rounded-lg p-6 mb-8">
			<h3 class="mb-4 text-orange-400">Next Steps:</h3>
			<ul class="m-0 pl-6">
				<li class="mb-2 text-gray-500">
					<strong class="text-orange-300">Chunk 0:</strong> Database schema & encryption foundation
				</li>
				<li class="mb-2 text-gray-500">
					<strong class="text-orange-300">Chunk 2:</strong> Rich text editor (Tiptap)
				</li>
				<li class="mb-2 text-gray-500">
					<strong class="text-orange-300">Chunk 3:</strong> Document management backend
				</li>
			</ul>
		</div>

		<footer class="text-center pt-8 border-t border-gray-200">
			<p class="m-0 text-gray-400 text-sm">Local-first • Privacy-respecting • AI-powered</p>
		</footer>
	</div>
</main>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		font-family:
			-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serig;
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: #333;
	}
</style>
