<script lang="ts">
	import { onMount } from 'svelte';
	import { libraryService } from '$services/library.service';
	import { rightPanelService } from '$services/right-panel.service';
	import { formatDuration } from '$types/youtube.type';
	import type { YouTubeContent } from '$types/youtube.type';

	const libState = libraryService.state;

	onMount(() => {
		libraryService.initialize();
	});

	function handleItemClick(item: YouTubeContent) {
		rightPanelService.open({
			videoId: item.youtubeId,
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
			{#each $libState.content as item (item.youtubeId)}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="card bg-base-100 shadow-sm cursor-pointer transition-shadow hover:shadow-md" onclick={() => handleItemClick(item)}>
					<figure class="aspect-video bg-base-300">
						{#if item.thumbnailUrl}
							<img
								src={item.thumbnailUrl}
								alt={item.title}
								class="h-full w-full object-cover"
								loading="lazy"
							/>
						{/if}
					</figure>
					<div class="card-body p-2">
						<p class="line-clamp-2 text-xs leading-tight font-medium" title={item.title}>
							{item.title}
						</p>
						{#if item.channelName}
							<p class="truncate text-xs opacity-50">{item.channelName}</p>
						{/if}
						{#if item.durationSeconds}
							<p class="text-xs opacity-40">{formatDuration(item.durationSeconds)}</p>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
