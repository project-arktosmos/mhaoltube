<script lang="ts">
	import { onMount } from 'svelte';
	import { libraryService } from '$services/library.service';
	import { rightPanelService } from '$services/right-panel.service';
	import { youtubeService } from '$services/youtube.service';
	import { youtubeSearchService } from '$services/youtube-search.service';
	import type { YouTubeDownloadProgress } from '$types/youtube.type';
	import type { YouTubeSearchItem } from '$types/youtube-search.type';
	import { youTubeCardAdapter } from '$adapters/classes/youtube-card.adapter';
	import type { LibraryCardItem } from '$types/library.type';
	import LibraryContentCard from '$components/libraries/LibraryContentCard.svelte';
	import YouTubeSearchInput from '$components/youtube-search/YouTubeSearchInput.svelte';

	const libState = libraryService.state;
	const ytState = youtubeService.state;
	const searchState = youtubeSearchService.state;

	const ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];

	let cardItems = $derived(
		$libState.content.map(youTubeCardAdapter.fromContent.bind(youTubeCardAdapter))
	);

	let favoriteItems = $derived(
		$libState.favorites.map(youTubeCardAdapter.fromContent.bind(youTubeCardAdapter))
	);

	let activeDownloadMap = $derived(
		new Map<string, YouTubeDownloadProgress>(
			$ytState.downloads.filter((d) => ACTIVE_STATES.includes(d.state)).map((d) => [d.videoId, d])
		)
	);

	let isSearchMode = $derived(
		$searchState.searching || $searchState.results.length > 0 || !!$searchState.query
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

	function handleSearch(query: string) {
		youtubeSearchService.search(query);
	}

	function handleSelect(item: YouTubeSearchItem) {
		rightPanelService.open({
			videoId: item.videoId,
			title: item.title,
			thumbnail: item.thumbnail,
			views: item.views,
			viewsText: item.viewsText,
			publishedText: item.uploadedDate || undefined,
			uploaderName: item.uploaderName || undefined,
			uploaderAvatar: item.uploaderAvatar || undefined,
			uploaderVerified: item.uploaderVerified
		});
	}

	function handleLoadMore() {
		youtubeSearchService.loadMore();
	}

	function handleClear() {
		youtubeSearchService.clearResults();
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-end justify-between gap-4">
		<div class="shrink-0">
			{#if isSearchMode}
				<h1 class="text-3xl font-bold">Search results</h1>
				<p class="text-sm opacity-70">YouTube videos matching your query</p>
			{:else}
				<h1 class="text-3xl font-bold">Library</h1>
				<p class="text-sm opacity-70">Your downloaded YouTube content</p>
			{/if}
		</div>

		<div class="flex flex-1 items-center gap-2">
			<div class="flex-1">
				<YouTubeSearchInput
					query={$searchState.query}
					searching={$searchState.searching}
					onsearch={handleSearch}
				/>
			</div>
			{#if isSearchMode}
				<button class="btn btn-ghost btn-sm" onclick={handleClear}>Clear</button>
			{/if}
		</div>
	</div>

	{#if $searchState.error}
		<div class="mb-4 alert alert-error">
			<span>{$searchState.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => youtubeSearchService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	{#if isSearchMode}
		{#if $searchState.searching}
			<div class="flex justify-center py-12">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else if $searchState.results.length > 0}
			<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
				{#each $searchState.results as item (item.url)}
					<LibraryContentCard
						item={youTubeCardAdapter.fromSearchItem(item)}
						onclick={() => handleSelect(item)}
					/>
				{/each}
			</div>

			{#if $searchState.continuation}
				<div class="mt-4 flex justify-center">
					<button
						class="btn btn-outline btn-sm"
						onclick={handleLoadMore}
						disabled={$searchState.loadingMore}
					>
						{#if $searchState.loadingMore}
							<span class="loading loading-sm loading-spinner"></span>
							Loading...
						{:else}
							Load More
						{/if}
					</button>
				</div>
			{/if}
		{:else if $searchState.query}
			<div class="mt-8 flex flex-col items-center gap-2 py-8 text-base-content/50">
				<p class="text-sm">No results found for "{$searchState.query}"</p>
			</div>
		{/if}
	{:else if $libState.contentLoading && $libState.content.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if $libState.contentError}
		<div class="rounded-lg bg-error/10 px-4 py-3 text-error">
			{$libState.contentError}
		</div>
	{:else if $libState.content.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No downloaded content yet. Search for videos above to get started.</p>
		</div>
	{:else}
		{#if favoriteItems.length > 0}
			<div class="mb-8">
				<h2 class="mb-3 text-xl font-semibold">Favorites</h2>
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each favoriteItems as item (item.videoId)}
						<LibraryContentCard
							{item}
							download={activeDownloadMap.get(item.videoId)}
							onclick={() => handleItemClick(item)}
						/>
					{/each}
				</div>
			</div>

			<h2 class="mb-3 text-xl font-semibold">All content</h2>
		{/if}

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
