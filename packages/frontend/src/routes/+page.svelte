<script lang="ts">
	import { onMount } from 'svelte';
	import { libraryService } from '$services/library.service';
	import { rightPanelService } from '$services/right-panel.service';
	import { youtubeService } from '$services/youtube.service';
	import type { YouTubeDownloadProgress } from '$types/youtube.type';
	import { youTubeCardAdapter } from '$adapters/classes/youtube-card.adapter';
	import type { LibraryCardItem } from '$types/library.type';
	import LibraryContentCard from '$components/libraries/LibraryContentCard.svelte';

	const libState = libraryService.state;
	const ytState = youtubeService.state;

	const ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];

	let cardItems = $derived($libState.content.map(youTubeCardAdapter.fromContent.bind(youTubeCardAdapter)));

	let activeDownloadMap = $derived(
		new Map<string, YouTubeDownloadProgress>(
			$ytState.downloads.filter((d) => ACTIVE_STATES.includes(d.state)).map((d) => [d.videoId, d])
		)
	);

	onMount(() => {
		libraryService.initialize();
	});

	function handleItemClick(item: LibraryCardItem) {
		rightPanelService.open({
			videoId: item.videoId,
			title: item.title,
			thumbnail: item.thumbnailUrl ?? '',
			uploaderName: item.channelName ?? undefined,
			hasVideo: item.hasVideo,
			hasAudio: item.hasAudio
		});
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Library</h1>
		<p class="text-sm opacity-70">Your downloaded YouTube content</p>
	</div>

	{#if $libState.contentLoading && $libState.content.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if $libState.contentError}
		<div class="rounded-lg bg-error/10 px-4 py-3 text-error">
			{$libState.contentError}
		</div>
	{:else if $libState.content.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No downloaded content yet. Download videos from the YouTube tab.</p>
		</div>
	{:else}
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
			{#each cardItems as item (item.videoId)}
				<LibraryContentCard
					{item}
					download={activeDownloadMap.get(item.videoId)}
					onclick={() => handleItemClick(item)}
				/>
			{/each}
		</div>
	{/if}
</div>
