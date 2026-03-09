<script lang="ts">
	import { apiUrl } from '$lib/api-base';
	import type {
		YouTubeRssVideo,
		YouTubeRssFeedResponse,
		YouTubeChannelMeta
	} from '$types/youtube.type';
	import type { YouTubeSearchChannelItem } from '$types/youtube-search.type';
	import { rightPanelService } from '$services/right-panel.service';
	import { youtubeChannelSearchService } from '$services/youtube-channel-search.service';
	import { youTubeCardAdapter } from '$adapters/classes/youtube-card.adapter';
	import LibraryContentCard from '$components/libraries/LibraryContentCard.svelte';
	import YouTubeSearchInput from '$components/youtube-search/YouTubeSearchInput.svelte';
	import YouTubeChannelCard from '$components/youtube-search/YouTubeChannelCard.svelte';

	interface YouTubeChannel {
		id: string;
		handle: string;
		name: string;
		url: string;
		subscriber_text: string | null;
		image_url: string | null;
		created_at: string;
		updated_at: string;
	}

	interface TableDetailResponse {
		table: string;
		columns: { name: string; type: string }[];
		rows: YouTubeChannel[];
		pagination: {
			page: number;
			limit: number;
			total: number;
			totalPages: number;
		};
	}

	type SelectedChannel = { name: string; handle: string; subscriberText?: string };

	// Subscriptions state
	let channels: YouTubeChannel[] = $state([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let pagination = $state<TableDetailResponse['pagination'] | null>(null);
	let channelMeta: Record<string, YouTubeChannelMeta> = $state({});

	// Search state
	const ytSearchState = youtubeChannelSearchService.state;
	let isSearchMode = $derived(
		$ytSearchState.channels.length > 0 || $ytSearchState.searching || !!$ytSearchState.query
	);
	let subscribedIds = $derived(new Set(channels.map((c) => c.id)));

	// Feed drill-down state
	let selectedChannel = $state<SelectedChannel | null>(null);
	let feedVideos: YouTubeRssVideo[] = $state([]);
	let feedLoading = $state(false);
	let feedError = $state<string | null>(null);

	async function fetchChannels(page: number = 1) {
		loading = true;
		error = null;
		try {
			const res = await fetch(
				apiUrl(`/api/database/tables/youtube_channels?page=${page}&limit=50`)
			);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data: TableDetailResponse = await res.json();
			channels = data.rows;
			pagination = data.pagination;
			fetchAllChannelMeta(data.rows);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function fetchAllChannelMeta(rows: YouTubeChannel[]) {
		const initial: Record<string, YouTubeChannelMeta> = {};
		const needsFetch: YouTubeChannel[] = [];
		for (const channel of rows) {
			if (channel.image_url) {
				initial[channel.handle] = {
					channelId: channel.id,
					avatar: channel.image_url!,
					description: '',
					subscriberText: channel.subscriber_text ?? ''
				};
			} else {
				needsFetch.push(channel);
			}
		}
		channelMeta = { ...channelMeta, ...initial };

		for (const channel of needsFetch) {
			if (channelMeta[channel.handle]) continue;
			try {
				const res = await fetch(apiUrl(`/api/youtube/channel-meta?handle=${channel.handle}`));
				if (!res.ok) {
					console.error(`[channel-meta] ${channel.handle} → HTTP ${res.status}`, await res.text());
					continue;
				}
				const data: YouTubeChannelMeta = await res.json();
				channelMeta = { ...channelMeta, [channel.handle]: data };
			} catch (e) {
				console.error(`[channel-meta] ${channel.handle} fetch failed`, e);
			}
		}
	}

	async function fetchChannelRss(channel: SelectedChannel) {
		selectedChannel = channel;
		feedLoading = true;
		feedError = null;
		feedVideos = [];
		try {
			const res = await fetch(apiUrl(`/api/youtube/channel-rss?handle=${channel.handle}`));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data: YouTubeRssFeedResponse = await res.json();
			feedVideos = data.videos;
		} catch (e) {
			feedError = e instanceof Error ? e.message : String(e);
		} finally {
			feedLoading = false;
		}
	}

	function handleDbChannelSelect(channel: YouTubeChannel) {
		const meta = channelMeta[channel.handle];
		fetchChannelRss({
			name: channel.name,
			handle: channel.handle,
			subscriberText: channel.subscriber_text || meta?.subscriberText
		});
	}

	function handleSearchChannelSelect(channel: YouTubeSearchChannelItem) {
		const handle = extractHandle(channel.url);
		if (!handle) return;
		fetchChannelRss({
			name: channel.name,
			handle,
			subscriberText: channel.subscriberText
		});
	}

	function extractHandle(url: string): string | null {
		if (url.startsWith('/@')) return url.slice(2);
		return null;
	}

	function goBack() {
		selectedChannel = null;
		feedVideos = [];
		feedError = null;
		rightPanelService.close();
	}

	function handleSearch(query: string) {
		youtubeChannelSearchService.search(query);
	}

	function handleClear() {
		youtubeChannelSearchService.clearResults();
	}

	async function handleSubscribe(channel: YouTubeSearchChannelItem) {
		const handle = extractHandle(channel.url);
		if (!handle || !channel.channelId) return;
		await fetch(apiUrl('/api/youtube/channel-subscribe'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				id: channel.channelId,
				handle,
				name: channel.name,
				url: `https://www.youtube.com${channel.url}`,
				subscriber_text: channel.subscriberText || null,
				image_url: channel.thumbnail || null
			})
		});
		await fetchChannels();
	}

	$effect(() => {
		fetchChannels();
	});
</script>

<div class="mx-auto max-w-4xl p-6">
	{#if selectedChannel}
		<!-- Feed view -->
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
				@{selectedChannel.handle}{selectedChannel.subscriberText
					? ` · ${selectedChannel.subscriberText}`
					: ''} — Latest videos (RSS)
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
			<div
				class="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each feedVideos as video (video.videoId)}
					<LibraryContentCard
						item={youTubeCardAdapter.fromRssVideo(video)}
						onclick={() => rightPanelService.open(video)}
					/>
				{/each}
			</div>
		{/if}
	{:else}
		<!-- Search bar — always visible -->
		<div class="flex items-center justify-between gap-4">
			<div class="flex-1">
				<YouTubeSearchInput
					query={$ytSearchState.query}
					searching={$ytSearchState.searching}
					onsearch={handleSearch}
				/>
			</div>
			{#if isSearchMode}
				<button class="btn shrink-0 btn-ghost btn-sm" onclick={handleClear}>Clear</button>
			{/if}
		</div>

		<!-- Heading -->
		<div class="mt-4">
			{#if isSearchMode}
				<h3 class="text-lg font-bold">Search results</h3>
				{#if $ytSearchState.query}
					<p class="text-sm text-base-content/60">for "{$ytSearchState.query}"</p>
				{/if}
			{:else}
				<h3 class="text-lg font-bold">Channels</h3>
				<p class="text-sm text-base-content/60">
					Subscribed channels ({pagination ? pagination.total : '...'})
				</p>
			{/if}
		</div>

		<!-- Error banners -->
		{#if error && !isSearchMode}
			<div class="mt-4 alert alert-error">
				<span>{error}</span>
				<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
			</div>
		{/if}

		{#if $ytSearchState.error}
			<div class="mt-4 alert alert-error">
				<span>{$ytSearchState.error}</span>
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => youtubeChannelSearchService.state.update((s) => ({ ...s, error: null }))}
				>
					Dismiss
				</button>
			</div>
		{/if}

		<!-- Channel grid -->
		{#if isSearchMode}
			{#if $ytSearchState.searching}
				<div class="mt-6 flex justify-center">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else if $ytSearchState.channels.length > 0}
				<div class="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
					{#each $ytSearchState.channels as channel (channel.channelId)}
						<YouTubeChannelCard
							{channel}
							subscribed={subscribedIds.has(channel.channelId)}
							onclick={extractHandle(channel.url)
								? () => handleSearchChannelSelect(channel)
								: undefined}
							onsubscribe={handleSubscribe}
						/>
					{/each}
				</div>

				{#if $ytSearchState.continuation}
					<div class="mt-4 flex justify-center">
						<button
							class="btn btn-outline btn-sm"
							onclick={() => youtubeChannelSearchService.loadMore()}
							disabled={$ytSearchState.loadingMore}
						>
							{#if $ytSearchState.loadingMore}
								<span class="loading loading-sm loading-spinner"></span>
								Loading...
							{:else}
								Load More
							{/if}
						</button>
					</div>
				{/if}
			{:else if $ytSearchState.query && !$ytSearchState.searching}
				<div class="mt-8 flex flex-col items-center gap-2 py-8 text-base-content/50">
					<p class="text-sm">No channels found for "{$ytSearchState.query}"</p>
				</div>
			{/if}
		{:else if loading}
			<div class="mt-6 flex justify-center">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else if channels.length === 0}
			<div class="mt-6 rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No channels found. Try resetting the database from the DB modal.</p>
			</div>
		{:else}
			<div class="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
				{#each channels as channel (channel.id)}
					{@const meta = channelMeta[channel.handle]}
					{@const avatar = channel.image_url || meta?.avatar}
					{@const subscriberText = channel.subscriber_text || meta?.subscriberText}
					<button
						onclick={() => handleDbChannelSelect(channel)}
						class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
					>
						{#if avatar}
							<img
								src={apiUrl(`/api/youtube/image-proxy?url=${encodeURIComponent(avatar)}`)}
								alt={channel.name}
								class="h-10 w-10 shrink-0 rounded-full object-cover"
								loading="lazy"
							/>
						{:else}
							<div
								class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-error/10 text-error"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-5 w-5"
									viewBox="0 0 24 24"
									fill="currentColor"
								>
									<path
										d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"
									/>
								</svg>
							</div>
						{/if}
						<div class="min-w-0 flex-1">
							<p class="truncate font-medium">{channel.name}</p>
							<p class="truncate text-sm opacity-50">
								@{channel.handle}{subscriberText ? ` · ${subscriberText}` : ''}
							</p>
						</div>
					</button>
				{/each}
			</div>

			{#if pagination && pagination.totalPages > 1}
				<div class="mt-3 flex items-center justify-center gap-2">
					<button
						class="btn btn-ghost btn-xs"
						disabled={pagination.page <= 1}
						onclick={() => fetchChannels(pagination!.page - 1)}
					>
						Prev
					</button>
					<span class="text-sm">
						Page {pagination.page} of {pagination.totalPages}
					</span>
					<button
						class="btn btn-ghost btn-xs"
						disabled={pagination.page >= pagination.totalPages}
						onclick={() => fetchChannels(pagination!.page + 1)}
					>
						Next
					</button>
				</div>
			{/if}
		{/if}
	{/if}
</div>
