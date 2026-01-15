import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://svelte.dev/docs/kit/integrations
	// for more information about preprocessors
	preprocess: vitePreprocess(),

	kit: {
		// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://svelte.dev/docs/kit/adapters for more information about adapters.
		adapter: adapter({
			// default settings produce 'build' directory which matches Tauri config
			pages: 'build', // Where HTML files go
			assets: 'build', // Where JS/CSS/Images go
			fallback: 'index.html', // SPA fallback (required for client-side routing)
			precompress: false, // Son't gzip (Tauri serves frontend files locally, no network transfer)
			strict: true // Error on missing prerendered routes
		}),

		// Application directories
		appDir: '_app',

		// Path configuration
		paths: {
			base: '', // No base path (served from localhost)
			assets: '' // Assets served from same origin
		},

		// Alias configuration
		alias: {
			$lib: './src/lib'
		},

		// Environment variables configuration
		env: {
			dir: process.cwd(),
			publicPrefix: 'PUBLIC_' // Determines which env vars are exposed to the webview
		},

		// Disable SSR
		// ssr: false,

		// CSP - relaxed for Tauri since webview is already sandboxed
		csp: {
			mode: 'auto',
			directives: {
				'script-src': ['self'] // Only allow scripts from same origin
			}
		}
	}
};

export default config;
