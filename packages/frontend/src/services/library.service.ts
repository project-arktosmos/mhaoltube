import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import type { Library } from '$types/library.type';
import type { YouTubeContent } from '$types/youtube.type';

export interface LibraryServiceState {
	content: YouTubeContent[];
	contentLoading: boolean;
	contentError: string | null;
}

const initialState: LibraryServiceState = {
	content: [],
	contentLoading: false,
	contentError: null
};

class LibraryService {
	public library: Writable<Library | null> = writable(null);
	public state: Writable<LibraryServiceState> = writable(initialState);

	private initialized = false;

	reset(): void {
		this.initialized = false;
		this.library.set(null);
		this.state.set(initialState);
	}

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;

		try {
			const library = await this.fetchJson<Library>('/api/libraries');
			this.library.set(library);
			this.initialized = true;
			this.fetchContent();
		} catch (error) {
			console.error('[library] Failed to initialize:', error);
		}
	}

	async fetchContent(): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, contentLoading: true, contentError: null }));

		try {
			const content = await this.fetchJson<YouTubeContent[]>('/api/media');
			this.state.update((s) => ({ ...s, content, contentLoading: false }));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, contentLoading: false, contentError: errorMsg }));
		}
	}

	streamVideoUrl(youtubeId: string): string {
		return apiUrl(`/api/libraries/content/${youtubeId}/stream/video`);
	}

	streamAudioUrl(youtubeId: string): string {
		return apiUrl(`/api/libraries/content/${youtubeId}/stream/audio`);
	}

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(apiUrl(path), {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...init?.headers
			}
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}
}

export const libraryService = new LibraryService();
