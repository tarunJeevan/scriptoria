import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

// https://vite.dev/config/
export default defineConfig({
	plugins: [sveltekit()],

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent Vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: '0.0.0.0',
		hmr: {
			protocol: 'ws',
			host: 'localhost',
			port: 1430
		},
		watch: {
			// 3. tell Vite to ignore watching `src-tauri`
			ignored: ['**/src-tauri/**']
		}
	},

	// Build optimizations
	build: {
		// Tauri uses Chromium on Windows and WebKit on macOS and Linux
		target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',

		// Don't minify for debug builds
		minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,

		// Produce sourcemaps for debug builds
		sourcemap: !!process.env.TAURI_DEBUG,

		// Optimize chunk splitting
		rollupOptions: {
			output: {
				manualChunks: (id) => {
					// Only chunk actual dependencies that get bundled
					// (not externals like @sveltejs/kit or @tauri-apps/api)
					if (id.includes('node_modules')) {
						// Svelte core library (not SvelteKit framework)
						if (id.includes('svelte/')) {
							return 'svelte-vendor';
						}
						// Other large dependencies can be split here
						// Example: if (id.includes("some-large-lib")) return "lib-vendor";
					}
				}
			}
		}
	},

	// Environment variable prefix
	envPrefix: ['VITE_', 'TAURI_'],

	// Prevent Vite from obscuring Rust errors
	esbuild: {
		logLevel: 'info',
		logLimit: 0
	}
});
