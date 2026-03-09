<script lang="ts">
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import { rightPanelService } from '$services/right-panel.service';
	import { libraryService } from '$services/library.service';
	import { youtubeService } from '$services/youtube.service';
	import { getStateLabel, getStateColor } from '$types/youtube.type';

	const panelStore = rightPanelService.store;
	const ytState = youtubeService.state;

	let video = $derived($panelStore.video);
	let isOpen = $derived(video !== null);

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

	let playerMode = $state<'audio' | 'video' | null>(null);
	let audioEl = $state<HTMLAudioElement | null>(null);

	// Video src: use in-progress stream during download, final stream once complete
	let videoSrc = $derived(
		video?.hasVideo
			? libraryService.streamVideoUrl(video.videoId)
			: activeDownload?.videoOutputPath && ['downloading', 'muxing'].includes(activeDownload.state)
				? libraryService.streamDownloadVideoUrl(activeDownload.downloadId)
				: null
	);

	$effect(() => {
		if (video) playerMode = null;
	});

	// Auto-show video player as soon as a streamable source is available
	$effect(() => {
		if (videoSrc && playerMode !== 'video') {
			playerMode = 'video';
		}
	});

	$effect(() => {
		if (audioEl) audioEl.play().catch(() => {});
	});

	let autoStartedId: string | null = null;

	$effect(() => {
		const v = video;
		if (!v) {
			autoStartedId = null;
			return;
		}
		if (v.videoId === autoStartedId) return;

		const downloads = get(youtubeService.state).downloads;
		const alreadyInQueue = downloads.some(
			(d) =>
				d.videoId === v.videoId &&
				['pending', 'fetching', 'downloading', 'muxing'].includes(d.state)
		);
		const alreadyDownloaded = v.hasVideo && v.hasAudio;

		if (!alreadyInQueue && !alreadyDownloaded) {
			autoStartedId = v.videoId;
			youtubeService.queueDownloadWithMode(v.videoId, v.title, v.thumbnail ?? null, 'both');
		}
	});

	let downloadingAudio = $state(false);
	let downloadingVideo = $state(false);
	let downloadingBoth = $state(false);

	const activeStates = ['pending', 'fetching', 'downloading', 'muxing'];

	let audioDone = $derived(video?.hasAudio === true);
	let videoDone = $derived(video?.hasVideo === true);
	let bothDone = $derived(video?.hasAudio === true && video?.hasVideo === true);

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
	let bothInProgress = $derived(
		videoDownloads.some((d) => d.mode === 'both' && activeStates.includes(d.state))
	);

	let audioDisabled = $derived(audioDone || audioInProgress || downloadingAudio);
	let videoDisabled = $derived(videoDone || videoInProgress || downloadingVideo);
	let bothDisabled = $derived(bothDone || bothInProgress || downloadingBoth);

	let wrapperClasses = $derived(
		classNames(
			'flex flex-col bg-base-200 border-l border-base-300 overflow-y-auto transition-[width] duration-200',
			isOpen ? 'w-80' : 'w-0 overflow-hidden'
		)
	);

	async function handleDownload(mode: 'audio' | 'video' | 'both') {
		if (!video) return;
		if (mode === 'audio' && audioDisabled) return;
		if (mode === 'video' && videoDisabled) return;
		if (mode === 'both' && bothDisabled) return;

		if (mode === 'audio') downloadingAudio = true;
		else if (mode === 'video') downloadingVideo = true;
		else downloadingBoth = true;

		await youtubeService.queueDownloadWithMode(video.videoId, video.title, video.thumbnail, mode);

		if (mode === 'audio') downloadingAudio = false;
		else if (mode === 'video') downloadingVideo = false;
		else downloadingBoth = false;
	}
</script>

<aside class={wrapperClasses}>
	{#if video}
		<div class="flex min-w-80 flex-col gap-4 p-4">
			<div class="flex items-center justify-between">
				<h3 class="text-xs font-semibold tracking-widest uppercase opacity-50">Video</h3>
				<button
					class="btn btn-circle btn-ghost btn-xs"
					onclick={() => rightPanelService.close()}
					aria-label="Close panel"
				>
					✕
				</button>
			</div>

			{#if playerMode === 'video' && videoSrc}
				<!-- svelte-ignore a11y_media_has_caption -->
				<video controls autoplay src={videoSrc} class="w-full rounded-lg">
					<source src={videoSrc} type="video/mp4" />
				</video>
			{:else}
				<img src={video.thumbnail} alt={video.title} class="w-full rounded-lg object-cover" />
				{#if playerMode === 'audio'}
					<audio bind:this={audioEl} controls autoplay class="w-full">
						<source src={libraryService.streamAudioUrl(video.videoId)} type="audio/x-m4a" />
					</audio>
				{/if}
			{/if}

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

			{#if video.hasVideo || video.hasAudio || videoSrc}
				<div class="divider my-0 text-xs opacity-50">Play</div>
				<div class="flex flex-col gap-2">
					{#if video.hasVideo || videoSrc}
						<button
							class={classNames('btn w-full gap-2 btn-sm btn-secondary', {
								'btn-active': playerMode === 'video'
							})}
							onclick={() => (playerMode = playerMode === 'video' ? null : 'video')}
						>
							▶ {video.hasVideo ? 'Play Video' : 'Play (downloading…)'}
						</button>
					{/if}
					{#if video.hasAudio}
						<button
							class={classNames('btn w-full gap-2 btn-sm btn-primary', {
								'btn-active': playerMode === 'audio'
							})}
							onclick={() => (playerMode = playerMode === 'audio' ? null : 'audio')}
						>
							♪ Play Audio
						</button>
					{/if}
				</div>
			{/if}

			{#if activeDownload}
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
					{:else if activeDownload.state === 'completed'}
						<progress class="progress w-full progress-success" value="1" max="1"></progress>
					{:else if activeDownload.state === 'failed'}
						<p class="text-xs text-error">{activeDownload.error ?? 'Download failed'}</p>
					{:else}
						<progress class="progress w-full progress-primary"></progress>
					{/if}
				</div>
			{/if}

			<div class="divider my-0 text-xs opacity-50">Download</div>

			<div class="flex flex-col gap-2">
				<button
					class="btn w-full gap-2 btn-sm btn-primary"
					disabled={audioDisabled}
					onclick={() => handleDownload('audio')}
				>
					{#if downloadingAudio || audioInProgress}
						<span class="loading loading-xs loading-spinner"></span>
					{/if}
					Audio only
					{#if audioDone}
						<span class="badge badge-xs badge-success">✓</span>
					{/if}
				</button>

				<button
					class="btn w-full gap-2 btn-sm btn-secondary"
					disabled={videoDisabled}
					onclick={() => handleDownload('video')}
				>
					{#if downloadingVideo || videoInProgress}
						<span class="loading loading-xs loading-spinner"></span>
					{/if}
					Video
					{#if videoDone}
						<span class="badge badge-xs badge-success">✓</span>
					{/if}
				</button>

				<button
					class="btn w-full gap-2 btn-sm btn-accent"
					disabled={bothDisabled}
					onclick={() => handleDownload('both')}
				>
					{#if downloadingBoth || bothInProgress}
						<span class="loading loading-xs loading-spinner"></span>
					{/if}
					Audio + Video
					{#if bothDone}
						<span class="badge badge-xs badge-success">✓</span>
					{/if}
				</button>
			</div>
		</div>
	{/if}
</aside>
