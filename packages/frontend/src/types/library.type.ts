import type { ID } from '$types/core.type';

export interface Library {
	id: ID;
	name: string;
	path: string;
	dateAdded: number;
}
