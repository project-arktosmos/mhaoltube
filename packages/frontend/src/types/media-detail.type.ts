import type { MediaItem } from '$types/media-card.type';
import type { YouTubeOEmbedResponse } from '$types/youtube.type';

export type MediaDetailCardType = 'movie' | 'tv' | 'youtube' | 'audio' | 'image' | 'video';

export interface MediaDetailSelection {
	item: MediaItem;
	cardType: MediaDetailCardType;
	youtubeMetadata: YouTubeOEmbedResponse | null;
	onplay?: (item: MediaItem) => void;
	onlink?: (item: MediaItem, service: string) => void;
	onunlink?: (item: MediaItem, service: string) => void;
}
