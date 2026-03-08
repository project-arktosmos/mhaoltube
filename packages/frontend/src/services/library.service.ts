import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import type {
	Library,
	LibraryFile,
	LibraryFilesResponse,
	MediaTypeOption,
	CategoryOption
} from '$types/library.type';
import { type MediaType } from '$types/library.type';

export interface LibraryServiceState {
	files: LibraryFile[];
	filesLoading: boolean;
	filesError: string | null;
}

const initialState: LibraryServiceState = {
	files: [],
	filesLoading: false,
	filesError: null
};

class LibraryService {
	public library: Writable<Library | null> = writable(null);
	public state: Writable<LibraryServiceState> = writable(initialState);

	private initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;

		try {
			const library = await this.fetchJson<Library>('/api/libraries');
			this.library.set(library);
			this.initialized = true;
			this.fetchFiles();
		} catch (error) {
			console.error('[library] Failed to initialize:', error);
		}
	}

	async fetchFiles(): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, filesLoading: true, filesError: null }));

		try {
			const response = await this.fetchJson<LibraryFilesResponse>('/api/libraries/files');
			this.state.update((s) => ({
				...s,
				files: response.files,
				filesLoading: false
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				filesLoading: false,
				filesError: errorMsg
			}));
		}
	}

	async scanFiles(): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, filesLoading: true, filesError: null }));

		try {
			const response = await this.fetchJson<LibraryFilesResponse>('/api/libraries/scan', {
				method: 'POST'
			});
			this.state.update((s) => ({
				...s,
				files: response.files,
				filesLoading: false
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				filesLoading: false,
				filesError: errorMsg
			}));
		}
	}

	async linkTmdb(
		itemId: string,
		tmdbId: number,
		seasonNumber: number | null = null,
		episodeNumber: number | null = null
	): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/tmdb`, {
			method: 'PUT',
			body: JSON.stringify({ tmdbId, seasonNumber, episodeNumber })
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) =>
				f.id === itemId
					? {
							...f,
							links: {
								...f.links,
								tmdb: { serviceId: String(tmdbId), seasonNumber, episodeNumber }
							}
						}
					: f
			)
		}));
	}

	async unlinkTmdb(itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/tmdb`, {
			method: 'DELETE'
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) => {
				if (f.id !== itemId) return f;
				const { tmdb: _, ...rest } = f.links;
				return { ...f, links: rest };
			})
		}));
	}

	async linkYoutube(itemId: string, youtubeId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/youtube`, {
			method: 'PUT',
			body: JSON.stringify({ youtubeId })
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) =>
				f.id === itemId
					? {
							...f,
							links: {
								...f.links,
								youtube: { serviceId: youtubeId, seasonNumber: null, episodeNumber: null }
							}
						}
					: f
			)
		}));
	}

	async unlinkYoutube(itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/youtube`, {
			method: 'DELETE'
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) => {
				if (f.id !== itemId) return f;
				const { youtube: _, ...rest } = f.links;
				return { ...f, links: rest };
			})
		}));
	}

	async linkMusicBrainz(itemId: string, musicbrainzId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/musicbrainz`, {
			method: 'PUT',
			body: JSON.stringify({ musicbrainzId })
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) =>
				f.id === itemId
					? {
							...f,
							links: {
								...f.links,
								musicbrainz: {
									serviceId: musicbrainzId,
									seasonNumber: null,
									episodeNumber: null
								}
							}
						}
					: f
			)
		}));
	}

	async unlinkMusicBrainz(itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/musicbrainz`, {
			method: 'DELETE'
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) => {
				if (f.id !== itemId) return f;
				const { musicbrainz: _, ...rest } = f.links;
				return { ...f, links: rest };
			})
		}));
	}

	async updateCategory(itemId: string, categoryId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/category`, {
			method: 'PUT',
			body: JSON.stringify({ categoryId })
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) => (f.id === itemId ? { ...f, categoryId } : f))
		}));
	}

	async clearCategory(itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/category`, {
			method: 'DELETE'
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) => (f.id === itemId ? { ...f, categoryId: null } : f))
		}));
	}

	async updateMediaType(itemId: string, mediaTypeId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/items/${itemId}/media-type`, {
			method: 'PUT',
			body: JSON.stringify({ mediaTypeId })
		});

		this.state.update((s) => ({
			...s,
			files: s.files.map((f) =>
				f.id === itemId ? { ...f, mediaType: mediaTypeId as MediaType, categoryId: null } : f
			)
		}));
	}

	async fetchMediaTypes(): Promise<MediaTypeOption[]> {
		return this.fetchJson<MediaTypeOption[]>('/api/libraries/media-types');
	}

	async fetchCategories(mediaType?: string): Promise<CategoryOption[]> {
		const params = mediaType ? `?mediaType=${encodeURIComponent(mediaType)}` : '';
		return this.fetchJson<CategoryOption[]>(`/api/libraries/categories${params}`);
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
