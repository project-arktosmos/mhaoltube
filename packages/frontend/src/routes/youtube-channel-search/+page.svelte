<script lang="ts">
	import { youtubeChannelSearchService } from '$services/youtube-channel-search.service';
	import { rightPanelService } from '$services/right-panel.service';
	import { youTubeCardAdapter } from '$adapters/classes/youtube-card.adapter';
	import { apiUrl } from '$lib/api-base';
	import YouTubeSearchInput from '$components/youtube-search/YouTubeSearchInput.svelte';
	import YouTubeChannelCard from '$components/youtube-search/YouTubeChannelCard.svelte';
	import LibraryContentCard from '$components/libraries/LibraryContentCard.svelte';
	import type { YouTubeSearchChannelItem } from '$types/youtube-search.type';
	import type { YouTubeRssVideo, YouTubeRssFeedResponse } from '$types/youtube.type';

	const searchState = youtubeChannelSearchService.state;

	let selectedChannel = $state<YouTubeSearchChannelItem | null>(null);
	let feedVideos: YouTubeRssVideo[] = $state([]);
	let feedLoading = $state(false);
	let feedError = $state<string | null>(null);

	function handleSearch(query: string) {
		youtubeChannelSearchService.search(query);
	}

	function handleLoadMore() {
		youtubeChannelSearchService.loadMore();
	}

	function handleClear() {
		youtubeChannelSearchService.clearResults();
	}

	function extractHandle(url: string): string | null {
		if (url.startsWith('/@')) return url.slice(2);
		return null;
	}

	async function handleChannelSelect(channel: YouTubeSearchChannelItem) {
		const handle = extractHandle(channel.url);
		if (!handle) return;

		selectedChannel = channel;
		feedVideos = [];
		feedLoading = true;
		feedError = null;

		try {
			const res = await fetch(apiUrl(`/api/youtube/channel-rss?handle=${handle}`));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data: YouTubeRssFeedResponse = await res.json();
			feedVideos = data.videos;
		} catch (e) {
			feedError = e instanceof Error ? e.message : String(e);
		} finally {
			feedLoading = false;
		}
	}

	function goBack() {
		selectedChannel = null;
		feedVideos = [];
		feedError = null;
		rightPanelService.close();
	}
</script>

<div class="mx-auto max-w-5xl p-6">
	{#if selectedChannel}
		<div>
			<button class="btn gap-1 btn-ghost btn-sm" onclick={goBack}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4"
					viewBox="0 0 20 20"
					fill="currentColor"
				>
					<path
						fill-rule="evenodd"
						d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z"
						clip-rule="evenodd"
					/>
				</svg>
				Back
			</button>
			<h3 class="mt-2 text-lg font-bold">{selectedChannel.name}</h3>
			<p class="text-sm text-base-content/60">
				{selectedChannel.subscriberText
					? `${selectedChannel.subscriberText} · `
					: ''}{selectedChannel.videoCountText || 'Latest videos'}
			</p>
		</div>

		{#if feedError}
			<div class="mt-4 alert alert-error">
				<span>{feedError}</span>
				<button class="btn btn-ghost btn-sm" onclick={() => (feedError = null)}>x</button>
			</div>
		{/if}

		{#if feedLoading}
			<div class="mt-6 flex justify-center">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else if feedVideos.length === 0 && !feedError}
			<div class="mt-6 rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No videos found for this channel.</p>
			</div>
		{:else}
			<div class="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
				{#each feedVideos as video (video.videoId)}
					<LibraryContentCard
						item={youTubeCardAdapter.fromRssVideo(video)}
						onclick={() => rightPanelService.open(video)}
					/>
				{/each}
			</div>
		{/if}
	{:else}
		<div class="flex items-center justify-between">
			<div>
				<h3 class="text-lg font-bold">YouTube Channel Search</h3>
				<p class="text-sm text-base-content/60">Search for YouTube channels</p>
			</div>
			{#if $searchState.channels.length > 0}
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
					onclick={() => youtubeChannelSearchService.state.update((s) => ({ ...s, error: null }))}
				>
					Dismiss
				</button>
			</div>
		{/if}

		{#if $searchState.searching}
			<div class="flex justify-center py-8">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else if $searchState.channels.length > 0}
			<div class="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-2">
				{#each $searchState.channels as channel (channel.channelId)}
					<YouTubeChannelCard
						{channel}
						onclick={extractHandle(channel.url) ? handleChannelSelect : undefined}
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
		{:else if $searchState.query && !$searchState.searching}
			<div class="mt-8 flex flex-col items-center gap-2 py-8 text-base-content/50">
				<p class="text-sm">No channels found for "{$searchState.query}"</p>
			</div>
		{/if}
	{/if}
</div>
