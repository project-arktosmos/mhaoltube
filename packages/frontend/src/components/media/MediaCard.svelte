<script lang="ts">
	import type { MediaItem } from '$types/media-card.type';
	import type { YouTubeOEmbedResponse } from '$types/youtube.type';
	import YouTubeCard from './YouTubeCard.svelte';
	import VideoUncategorizedCard from './VideoUncategorizedCard.svelte';

	interface Props {
		item: MediaItem;
		youtubeMetadata?: YouTubeOEmbedResponse | null;
		metadataLoading?: boolean;
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let {
		item,
		youtubeMetadata = null,
		metadataLoading = false,
		selected = false,
		onselect
	}: Props = $props();

	let cardType = $derived.by(() => {
		if (item.links.youtube) return 'youtube';
		return 'video';
	});
</script>

{#if cardType === 'youtube'}
	<YouTubeCard {item} metadata={youtubeMetadata} loading={metadataLoading} {onselect} {selected} />
{:else}
	<VideoUncategorizedCard {item} {onselect} {selected} />
{/if}
