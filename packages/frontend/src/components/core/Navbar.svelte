<script lang="ts">
	import classNames from 'classnames';
	import { mediaModeService } from '$services/media-mode.service';
	import { youtubeService } from '$services/youtube.service';
	import YouTubeDownloadQueueModal from '$components/youtube/YouTubeDownloadQueueModal.svelte';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	let wrapperClasses = $derived(classNames('navbar bg-base-100 shadow-sm', classes));

	const mediaModeStore = mediaModeService.store;
	let mediaMode = $derived($mediaModeStore);

	const ytState = youtubeService.state;
	const ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];
	let activeCount = $derived(
		$ytState.downloads.filter((d) => ACTIVE_STATES.includes(d.state)).length
	);

	let queueOpen = $state(false);
</script>

<YouTubeDownloadQueueModal open={queueOpen} onClose={() => (queueOpen = false)} />

<nav class={wrapperClasses}>
	<div class="flex-1">
		<a href="/" class="btn text-xl btn-ghost">Mhaol<span class="text-primary">Tube</span></a>
	</div>
	<div class="flex-none flex items-center gap-2">
		<div class="indicator">
			{#if activeCount > 0}
				<span class="badge badge-primary badge-xs indicator-item">{activeCount}</span>
			{/if}
			<button
				class="btn btn-sm btn-ghost px-4"
				onclick={() => (queueOpen = true)}
				aria-label="Download queue"
				title="Download queue"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-5 w-5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
					/>
				</svg>
			</button>
		</div>
		<div class="join">
			<button
				class={classNames('btn join-item btn-sm', { 'btn-primary': mediaMode === 'audio' })}
				onclick={() => mediaModeService.setMode('audio')}
			>
				Audio
			</button>
			<button
				class={classNames('btn join-item btn-sm', { 'btn-primary': mediaMode === 'video' })}
				onclick={() => mediaModeService.setMode('video')}
			>
				Video
			</button>
		</div>
	</div>
</nav>
