<script lang="ts">
	import { youtubeSearchService } from '$services/youtube-search.service';
	import { rightPanelService } from '$services/right-panel.service';
	import YouTubeSearchInput from '$components/youtube-search/YouTubeSearchInput.svelte';
	import YouTubeSearchResultCard from '$components/youtube-search/YouTubeSearchResultCard.svelte';
	import type { YouTubeSearchItem } from '$types/youtube-search.type';

	const searchState = youtubeSearchService.state;

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

<div class="mx-auto max-w-5xl p-6">
	<div class="flex items-center justify-between">
		<div>
			<h3 class="text-lg font-bold">YouTube Search</h3>
			<p class="text-sm text-base-content/60">
				Search for YouTube videos and send them to the downloader
			</p>
		</div>
		{#if $searchState.results.length > 0}
			<button class="btn btn-ghost btn-sm" onclick={handleClear}>Clear</button>
		{/if}
	</div>

	<div class="mt-4">
		<YouTubeSearchInput
			query={$searchState.query}
			searching={$searchState.searching}
			onsearch={handleSearch}
		/>
	</div>

	{#if $searchState.error}
		<div class="mt-4 alert alert-error">
			<span>{$searchState.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => youtubeSearchService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	{#if $searchState.searching}
		<div class="flex justify-center py-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if $searchState.results.length > 0}
		<div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each $searchState.results as item (item.url)}
				<YouTubeSearchResultCard {item} onselect={handleSelect} />
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
	{:else if $searchState.query && !$searchState.searching}
		<div class="mt-8 flex flex-col items-center gap-2 py-8 text-base-content/50">
			<p class="text-sm">No results found for "{$searchState.query}"</p>
		</div>
	{/if}
</div>
