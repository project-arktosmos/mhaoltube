<script lang="ts">
	import classNames from 'classnames';
	import { rightPanelService } from '$services/right-panel.service';
	import { libraryService } from '$services/library.service';
	import { youtubeService } from '$services/youtube.service';
	import { mediaModeService } from '$services/media-mode.service';
	import { getStateLabel, getStateColor } from '$types/youtube.type';

	const panelStore = rightPanelService.store;
	const ytState = youtubeService.state;
	const libState = libraryService.state;
	const mediaModeStore = mediaModeService.store;
	let mediaMode = $derived($mediaModeStore);

	let video = $derived($panelStore.video);
	let isOpen = $derived(video !== null);

	let liveContent = $derived(
		video ? ($libState.content.find((c) => c.youtubeId === video!.videoId) ?? null) : null
	);

	let videoDownloads = $derived(
		video ? $ytState.downloads.filter((d) => d.videoId === video!.videoId) : []
	);
	let activeDownload = $derived(
		videoDownloads.find((d) =>
			['pending', 'fetching', 'downloading', 'muxing'].includes(d.state)
		) ??
			videoDownloads.at(-1) ??
			null
	);

	let audioEl = $state<HTMLAudioElement | null>(null);

	let hasVideo = $derived(liveContent?.hasVideo ?? false);
	let hasAudio = $derived(liveContent?.hasAudio ?? false);

	let videoSrc = $derived(hasVideo ? libraryService.streamVideoUrl(video!.videoId) : null);

	$effect(() => {
		if (audioEl) audioEl.play().catch(() => {});
	});

	let downloadingAudio = $state(false);
	let downloadingVideo = $state(false);

	const activeStates = ['pending', 'fetching', 'downloading', 'muxing'];

	let audioInProgress = $derived(
		videoDownloads.some(
			(d) => (d.mode === 'audio' || d.mode === 'both') && activeStates.includes(d.state)
		)
	);
	let videoInProgress = $derived(
		videoDownloads.some(
			(d) => (d.mode === 'video' || d.mode === 'both') && activeStates.includes(d.state)
		)
	);

	let wrapperClasses = $derived(
		classNames(
			'flex flex-col bg-base-200 border-l border-base-300 overflow-y-auto transition-[width] duration-200',
			isOpen ? 'w-80' : 'w-0 overflow-hidden'
		)
	);

	async function handleDownload(mode: 'audio' | 'video') {
		if (!video) return;

		if (mode === 'audio') downloadingAudio = true;
		else downloadingVideo = true;

		await youtubeService.queueDownloadWithMode(video.videoId, video.title, video.thumbnail, mode);

		if (mode === 'audio') downloadingAudio = false;
		else downloadingVideo = false;
	}
</script>

<aside class={wrapperClasses}>
	{#if video}
		<div class="flex min-w-80 flex-col gap-4 p-4">
			<div class="flex items-center justify-between">
				<h3 class="text-xs font-semibold tracking-widest uppercase opacity-50">
					{mediaMode === 'audio' ? 'Audio' : 'Video'}
				</h3>
				<button
					class="btn btn-circle btn-ghost btn-xs"
					onclick={() => rightPanelService.close()}
					aria-label="Close panel"
				>
					✕
				</button>
			</div>

			{#key video.videoId}
				{#if mediaMode === 'video' && videoSrc}
					<!-- svelte-ignore a11y_media_has_caption -->
					<video controls autoplay src={videoSrc} class="w-full rounded-lg">
						<source src={videoSrc} type="video/mp4" />
					</video>
				{:else if mediaMode === 'audio' && hasAudio}
					<img src={video.thumbnail} alt={video.title} class="w-full rounded-lg object-cover" />
					<audio bind:this={audioEl} controls autoplay class="w-full">
						<source src={libraryService.streamAudioUrl(video.videoId)} type="audio/x-m4a" />
					</audio>
				{:else}
					<img src={video.thumbnail} alt={video.title} class="w-full rounded-lg object-cover" />
				{/if}
			{/key}

			<div class="flex flex-col gap-1">
				<p class="leading-snug font-medium">{video.title}</p>
				{#if video.uploaderName}
					<div class="mt-1 flex items-center gap-2">
						{#if video.uploaderAvatar}
							<img
								src={video.uploaderAvatar}
								alt={video.uploaderName}
								class="h-4 w-4 rounded-full"
							/>
						{/if}
						<span class="text-sm text-base-content/60">{video.uploaderName}</span>
						{#if video.uploaderVerified}
							<span class="badge badge-xs badge-info">✓</span>
						{/if}
					</div>
				{/if}
				{#if video.viewsText}
					<p class="mt-1 text-sm text-base-content/60">{video.viewsText}</p>
				{/if}
				{#if video.publishedText}
					<p class="text-sm text-base-content/60">{video.publishedText}</p>
				{/if}
			</div>

			{#if activeDownload && activeStates.includes(activeDownload.state)}
				<div class="divider my-0 text-xs opacity-50">Download Progress</div>
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<span class="text-sm font-medium">{getStateLabel(activeDownload.state)}</span>
						<span class="badge badge-xs badge-{getStateColor(activeDownload.state)}">
							{activeDownload.mode}
						</span>
					</div>
					{#if activeDownload.state === 'downloading' || activeDownload.state === 'muxing'}
						<progress
							class="progress w-full progress-primary"
							value={activeDownload.progress}
							max="1"
						></progress>
						<p class="text-right text-xs text-base-content/50">
							{Math.round(activeDownload.progress * 100)}%
						</p>
					{:else}
						<progress class="progress w-full progress-primary"></progress>
					{/if}
				</div>
			{/if}

			{#if (mediaMode === 'audio' && !hasAudio) || (mediaMode === 'video' && !hasVideo)}
				<div class="divider my-0 text-xs opacity-50">Download</div>

				{#if mediaMode === 'audio'}
					<button
						class="btn w-full gap-2 btn-sm btn-primary"
						disabled={audioInProgress || downloadingAudio}
						onclick={() => handleDownload('audio')}
					>
						{#if downloadingAudio || audioInProgress}
							<span class="loading loading-xs loading-spinner"></span>
						{/if}
						Download Audio
					</button>
				{:else}
					<button
						class="btn w-full gap-2 btn-sm btn-secondary"
						disabled={videoInProgress || downloadingVideo}
						onclick={() => handleDownload('video')}
					>
						{#if downloadingVideo || videoInProgress}
							<span class="loading loading-xs loading-spinner"></span>
						{/if}
						Download Video
					</button>
				{/if}
			{/if}
		</div>
	{/if}
</aside>
