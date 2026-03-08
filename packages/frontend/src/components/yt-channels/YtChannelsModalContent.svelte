<script lang="ts">
	import { apiUrl } from '$lib/api-base';
	import type {
		YouTubeRssVideo,
		YouTubeRssFeedResponse,
		YouTubeChannelMeta
	} from '$types/youtube.type';

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

	let channels: YouTubeChannel[] = $state([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let pagination = $state<TableDetailResponse['pagination'] | null>(null);

	// Channel metadata (avatar, description, subscribers)
	let channelMeta: Record<string, YouTubeChannelMeta> = $state({});

	// Feed drill-down state
	let selectedChannel = $state<YouTubeChannel | null>(null);
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
		// Populate from DB-cached fields first for instant display
		const initial: Record<string, YouTubeChannelMeta> = {};
		const needsFetch: YouTubeChannel[] = [];
		for (const channel of rows) {
			if (channel.image_url && channel.subscriber_text) {
				initial[channel.handle] = {
					channelId: channel.id,
					avatar: channel.image_url,
					description: '',
					subscriberText: channel.subscriber_text
				};
			} else {
				needsFetch.push(channel);
			}
		}
		channelMeta = { ...channelMeta, ...initial };

		// Fetch remaining from YouTube (backend will cache for next time)
		for (const channel of needsFetch) {
			if (channelMeta[channel.handle]) continue;
			try {
				const res = await fetch(apiUrl(`/api/youtube/channel-meta?handle=${channel.handle}`));
				if (!res.ok) continue;
				const data: YouTubeChannelMeta = await res.json();
				channelMeta = { ...channelMeta, [channel.handle]: data };
			} catch {
				// Silently ignore metadata fetch failures
			}
		}
	}

	async function fetchChannelRss(channel: YouTubeChannel) {
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

	function goBack() {
		selectedChannel = null;
		feedVideos = [];
		feedError = null;
	}

	$effect(() => {
		fetchChannels();
	});
</script>

{#if selectedChannel}
	<!-- Feed view -->
	<div class="pr-8">
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
		<p class="text-sm text-base-content/60">@{selectedChannel.handle} — Latest videos (RSS)</p>
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
		<div class="mt-4 flex flex-col gap-2">
			{#each feedVideos as video (video.videoId)}
				<a
					href={`https://www.youtube.com/watch?v=${video.videoId}`}
					target="_blank"
					rel="noopener noreferrer"
					class="flex gap-3 rounded-lg bg-base-200 p-3 transition-colors hover:bg-base-300"
				>
					<div class="shrink-0">
						<img
							src={video.thumbnail}
							alt={video.title}
							class="h-20 w-36 rounded-md object-cover"
							loading="lazy"
						/>
					</div>
					<div class="min-w-0 flex-1">
						<p class="line-clamp-2 font-medium">{video.title}</p>
						<p class="mt-1 text-sm text-base-content/60">
							{video.viewsText}
						</p>
						<p class="text-sm text-base-content/60">
							{video.publishedText}
						</p>
					</div>
				</a>
			{/each}
		</div>
	{/if}
{:else}
	<!-- Channel grid view -->
	<div class="pr-8">
		<h3 class="text-lg font-bold">YouTube Channels</h3>
		<p class="text-sm text-base-content/60">
			Subscribed channels ({pagination ? pagination.total : '...'})
		</p>
	</div>

	{#if error}
		<div class="mt-4 alert alert-error">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading}
		<div class="mt-6 flex justify-center">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if channels.length === 0}
		<div class="mt-6 rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No channels found. Try resetting the database from the DB modal.</p>
		</div>
	{:else}
		<div class="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-2">
			{#each channels as channel (channel.id)}
				{@const meta = channelMeta[channel.handle]}
				{@const avatar = channel.image_url || meta?.avatar}
				{@const subscriberText = channel.subscriber_text || meta?.subscriberText}
				<button
					onclick={() => fetchChannelRss(channel)}
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
