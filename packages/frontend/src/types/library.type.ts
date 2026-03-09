import type { ID } from '$types/core.type';

export interface Library {
	id: ID;
	name: string;
	path: string;
	dateAdded: number;
}

export interface LibraryFsEntry {
	name: string;
	size: number;
}

export interface LibraryFs {
	path: string;
	audio: LibraryFsEntry[];
	video: LibraryFsEntry[];
}
