<script lang="ts">
	import classNames from 'classnames';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';
	import { getThumbnailUrl } from '$utils/youtube';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import { playerService } from '$services/player.service';
	import type { MediaDetailSelection } from '$types/media-detail.type';
	import type { MediaType } from '$types/library.type';

	interface Props {
		selection: MediaDetailSelection;
		onclose?: () => void;
	}

	let { selection, onclose }: Props = $props();

	let imageUrl = $derived.by(() => {
		const { cardType, item } = selection;
		if (cardType === 'youtube') {
			const videoId = item.links.youtube?.serviceId ?? '';
			return videoId ? getThumbnailUrl(videoId) : null;
		}
		return null;
	});

	let imageAlt = $derived.by(() => {
		const { cardType, youtubeMetadata, item } = selection;
		if (cardType === 'youtube') return youtubeMetadata?.title ?? item.name;
		return item.name;
	});

	const playerState = playerService.state;
	let isPlaying = $derived($playerState.currentFile?.id === selection.item.id);
</script>

<div class="flex flex-col gap-3">
	<div class="flex items-center justify-between">
		<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">Detail</h2>
		<button
			class="btn btn-square btn-ghost btn-xs"
			onclick={() => onclose?.()}
			aria-label="Close detail"
		>
			&times;
		</button>
	</div>

	<figure class="relative overflow-hidden rounded-lg bg-base-300">
		{#if isPlaying && $playerState.currentFile}
			<PlayerVideo
				file={$playerState.currentFile}
				connectionState={$playerState.connectionState}
				positionSecs={$playerState.positionSecs}
				durationSecs={$playerState.durationSecs}
			/>
		{:else if imageUrl}
			<img src={imageUrl} alt={imageAlt} class="w-full object-cover" />
		{:else}
			<div class="flex h-40 w-full items-center justify-center text-base-content/20">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-16 w-16"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1"
						d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
					/>
				</svg>
			</div>
		{/if}
	</figure>

	<h3 class="text-base font-semibold" title={selection.item.name}>{selection.item.name}</h3>

	<div class="flex flex-wrap gap-1">
		<span
			class={classNames(
				'badge badge-xs',
				libraryFileAdapter.getMediaTypeBadgeClass(selection.item.mediaTypeId as MediaType)
			)}
		>
			{libraryFileAdapter.getMediaTypeLabel(selection.item.mediaTypeId as MediaType)}
		</span>
		{#if selection.item.categoryId}
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getCategoryBadgeClass(selection.item.categoryId)
				)}
			>
				{libraryFileAdapter.getCategoryLabel(selection.item.categoryId)}
			</span>
		{/if}
		<span class="badge badge-ghost badge-xs">{selection.item.extension}</span>
	</div>

	{#if selection.cardType === 'youtube'}
		{#if selection.youtubeMetadata}
			<p class="text-xs font-semibold">{selection.youtubeMetadata.title}</p>
			<p class="text-xs opacity-60">{selection.youtubeMetadata.author_name}</p>
		{/if}
	{:else}
		<p class="text-xs opacity-60" title={selection.item.path}>{selection.item.path}</p>
	{/if}

	<div class="flex flex-wrap gap-2">
		{#if selection.cardType === 'youtube' || selection.cardType === 'video'}
			{#if isPlaying}
				<button class="btn btn-ghost btn-sm" onclick={() => playerService.stop()}>Stop</button>
			{:else}
				<button class="btn btn-sm btn-accent" onclick={() => selection.onplay?.(selection.item)}
					>Play</button
				>
			{/if}
		{/if}
		{#if selection.cardType === 'video'}
			<button
				class="btn btn-sm btn-info"
				onclick={() => selection.onlink?.(selection.item, 'youtube')}>Link YouTube</button
			>
		{/if}
		{#if selection.cardType === 'youtube'}
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => selection.onunlink?.(selection.item, 'youtube')}>Unlink</button
			>
		{/if}
	</div>
</div>
