
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```sh
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const IMSETTINGS_INTEGRATE_DESKTOP: string;
	export const SHELL: string;
	export const npm_command: string;
	export const SESSION_MANAGER: string;
	export const WINDOWID: string;
	export const COLORTERM: string;
	export const XDG_CONFIG_DIRS: string;
	export const XDG_SESSION_PATH: string;
	export const HISTCONTROL: string;
	export const XDG_MENU_PREFIX: string;
	export const TERM_PROGRAM_VERSION: string;
	export const npm_config_npm_globalconfig: string;
	export const HISTSIZE: string;
	export const HOSTNAME: string;
	export const ICEAUTHORITY: string;
	export const NODE: string;
	export const TAURI_ENV_DEBUG: string;
	export const DOTNET_ROOT: string;
	export const SSH_AUTH_SOCK: string;
	export const npm_config_verify_deps_before_run: string;
	export const npm_config__jsr_registry: string;
	export const MEMORY_PRESSURE_WRITE: string;
	export const XMODIFIERS: string;
	export const DESKTOP_SESSION: string;
	export const GTK_RC_FILES: string;
	export const GDK_CORE_DEVICE_EVENTS: string;
	export const npm_config_globalconfig: string;
	export const GPG_TTY: string;
	export const EDITOR: string;
	export const XDG_SEAT: string;
	export const PWD: string;
	export const LOGNAME: string;
	export const XDG_SESSION_DESKTOP: string;
	export const TAURI_ENV_PLATFORM: string;
	export const XDG_SESSION_TYPE: string;
	export const PNPM_HOME: string;
	export const SYSTEMD_EXEC_PID: string;
	export const XAUTHORITY: string;
	export const XKB_DEFAULT_MODEL: string;
	export const GTK2_RC_FILES: string;
	export const HOME: string;
	export const SSH_ASKPASS: string;
	export const LANG: string;
	export const LS_COLORS: string;
	export const _JAVA_AWT_WM_NONREPARENTING: string;
	export const XDG_CURRENT_DESKTOP: string;
	export const npm_package_version: string;
	export const MEMORY_PRESSURE_WATCH: string;
	export const WAYLAND_DISPLAY: string;
	export const XDG_SEAT_PATH: string;
	export const INVOCATION_ID: string;
	export const pnpm_config_verify_deps_before_run: string;
	export const MANAGERPID: string;
	export const IMSETTINGS_MODULE: string;
	export const INIT_CWD: string;
	export const DOTNET_BUNDLE_EXTRACT_BASE_DIR: string;
	export const KDE_SESSION_UID: string;
	export const npm_lifecycle_script: string;
	export const MOZ_GMP_PATH: string;
	export const XKB_DEFAULT_LAYOUT: string;
	export const TAURI_ENV_PLATFORM_VERSION: string;
	export const XDG_SESSION_CLASS: string;
	export const TAURI_ENV_FAMILY: string;
	export const TERM: string;
	export const npm_package_name: string;
	export const ZED_FORCE_CLI_MODE: string;
	export const LESSOPEN: string;
	export const USER: string;
	export const npm_config_frozen_lockfile: string;
	export const QT_WAYLAND_RECONNECT: string;
	export const KDE_SESSION_VERSION: string;
	export const PAM_KWALLET5_LOGIN: string;
	export const TAURI_ENV_TARGET_TRIPLE: string;
	export const DISPLAY: string;
	export const npm_lifecycle_event: string;
	export const SHLVL: string;
	export const CVS_RSH: string;
	export const TAURI_CLI_VERBOSITY: string;
	export const XDG_VTNR: string;
	export const XDG_SESSION_ID: string;
	export const MANAGERPIDFDID: string;
	export const npm_config_user_agent: string;
	export const PNPM_SCRIPT_SRC_DIR: string;
	export const npm_execpath: string;
	export const LD_LIBRARY_PATH: string;
	export const XDG_RUNTIME_DIR: string;
	export const NODE_PATH: string;
	export const DEBUGINFOD_URLS: string;
	export const npm_package_json: string;
	export const DEBUGINFOD_IMA_CERT_PATH: string;
	export const TAURI_ENV_ARCH: string;
	export const KDEDIRS: string;
	export const JOURNAL_STREAM: string;
	export const XDG_DATA_DIRS: string;
	export const KDE_FULL_SESSION: string;
	export const PATH: string;
	export const DBUS_SESSION_BUS_ADDRESS: string;
	export const KDE_APPLICATIONS_AS_SCOPE: string;
	export const npm_package_engines_pnpm: string;
	export const MAIL: string;
	export const npm_config_registry: string;
	export const ALACRITTY_WINDOW_ID: string;
	export const ZED_TERM: string;
	export const npm_node_execpath: string;
	export const npm_package_engines_node: string;
	export const TERM_PROGRAM: string;
	export const NODE_ENV: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		IMSETTINGS_INTEGRATE_DESKTOP: string;
		SHELL: string;
		npm_command: string;
		SESSION_MANAGER: string;
		WINDOWID: string;
		COLORTERM: string;
		XDG_CONFIG_DIRS: string;
		XDG_SESSION_PATH: string;
		HISTCONTROL: string;
		XDG_MENU_PREFIX: string;
		TERM_PROGRAM_VERSION: string;
		npm_config_npm_globalconfig: string;
		HISTSIZE: string;
		HOSTNAME: string;
		ICEAUTHORITY: string;
		NODE: string;
		TAURI_ENV_DEBUG: string;
		DOTNET_ROOT: string;
		SSH_AUTH_SOCK: string;
		npm_config_verify_deps_before_run: string;
		npm_config__jsr_registry: string;
		MEMORY_PRESSURE_WRITE: string;
		XMODIFIERS: string;
		DESKTOP_SESSION: string;
		GTK_RC_FILES: string;
		GDK_CORE_DEVICE_EVENTS: string;
		npm_config_globalconfig: string;
		GPG_TTY: string;
		EDITOR: string;
		XDG_SEAT: string;
		PWD: string;
		LOGNAME: string;
		XDG_SESSION_DESKTOP: string;
		TAURI_ENV_PLATFORM: string;
		XDG_SESSION_TYPE: string;
		PNPM_HOME: string;
		SYSTEMD_EXEC_PID: string;
		XAUTHORITY: string;
		XKB_DEFAULT_MODEL: string;
		GTK2_RC_FILES: string;
		HOME: string;
		SSH_ASKPASS: string;
		LANG: string;
		LS_COLORS: string;
		_JAVA_AWT_WM_NONREPARENTING: string;
		XDG_CURRENT_DESKTOP: string;
		npm_package_version: string;
		MEMORY_PRESSURE_WATCH: string;
		WAYLAND_DISPLAY: string;
		XDG_SEAT_PATH: string;
		INVOCATION_ID: string;
		pnpm_config_verify_deps_before_run: string;
		MANAGERPID: string;
		IMSETTINGS_MODULE: string;
		INIT_CWD: string;
		DOTNET_BUNDLE_EXTRACT_BASE_DIR: string;
		KDE_SESSION_UID: string;
		npm_lifecycle_script: string;
		MOZ_GMP_PATH: string;
		XKB_DEFAULT_LAYOUT: string;
		TAURI_ENV_PLATFORM_VERSION: string;
		XDG_SESSION_CLASS: string;
		TAURI_ENV_FAMILY: string;
		TERM: string;
		npm_package_name: string;
		ZED_FORCE_CLI_MODE: string;
		LESSOPEN: string;
		USER: string;
		npm_config_frozen_lockfile: string;
		QT_WAYLAND_RECONNECT: string;
		KDE_SESSION_VERSION: string;
		PAM_KWALLET5_LOGIN: string;
		TAURI_ENV_TARGET_TRIPLE: string;
		DISPLAY: string;
		npm_lifecycle_event: string;
		SHLVL: string;
		CVS_RSH: string;
		TAURI_CLI_VERBOSITY: string;
		XDG_VTNR: string;
		XDG_SESSION_ID: string;
		MANAGERPIDFDID: string;
		npm_config_user_agent: string;
		PNPM_SCRIPT_SRC_DIR: string;
		npm_execpath: string;
		LD_LIBRARY_PATH: string;
		XDG_RUNTIME_DIR: string;
		NODE_PATH: string;
		DEBUGINFOD_URLS: string;
		npm_package_json: string;
		DEBUGINFOD_IMA_CERT_PATH: string;
		TAURI_ENV_ARCH: string;
		KDEDIRS: string;
		JOURNAL_STREAM: string;
		XDG_DATA_DIRS: string;
		KDE_FULL_SESSION: string;
		PATH: string;
		DBUS_SESSION_BUS_ADDRESS: string;
		KDE_APPLICATIONS_AS_SCOPE: string;
		npm_package_engines_pnpm: string;
		MAIL: string;
		npm_config_registry: string;
		ALACRITTY_WINDOW_ID: string;
		ZED_TERM: string;
		npm_node_execpath: string;
		npm_package_engines_node: string;
		TERM_PROGRAM: string;
		NODE_ENV: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
