import { browser } from '$app/environment';
import { dev } from '$app/environment';

function getApiBase(): string {
	if (!browser) return '';

	const override = localStorage.getItem('api-server-url');
	if (override) return override;

	// In production Tauri builds, the frontend is served from a custom protocol
	// (e.g. https://tauri.localhost), so relative /api calls don't reach the
	// Rust backend. Point them at the embedded server explicitly.
	if (!dev && '__TAURI__' in window) {
		return 'http://127.0.0.1:1530';
	}

	return '';
}

export const apiBase = getApiBase();

export function apiUrl(path: string): string {
	return `${apiBase}${path}`;
}
