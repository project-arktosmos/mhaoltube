<script lang="ts">
	import Modal from '$components/core/Modal.svelte';
	import { modalRouterService } from '$services/modal-router.service';
	import YouTubeModalContent from '$components/youtube/YouTubeModalContent.svelte';
	import LibraryModalContent from '$components/libraries/LibraryModalContent.svelte';
	import SettingsModalContent from '$components/settings/SettingsModalContent.svelte';
	import YouTubeSearchModalContent from '$components/youtube-search/YouTubeSearchModalContent.svelte';
	import DbModalContent from '$components/db/DbModalContent.svelte';
	import YtChannelsModalContent from '$components/yt-channels/YtChannelsModalContent.svelte';

	const routerStore = modalRouterService.store;

	const MAX_WIDTHS: Record<string, string> = {
		youtube: 'max-w-6xl',
		'youtube-search': 'max-w-5xl',
		libraries: 'max-w-5xl',
		settings: 'max-w-2xl',
		db: 'max-w-6xl',
		'yt-channels': 'max-w-4xl'
	};

	let activeId = $derived($routerStore.navbarModal);
	let maxWidth = $derived(activeId ? (MAX_WIDTHS[activeId] ?? 'max-w-lg') : 'max-w-lg');

	function handleClose() {
		modalRouterService.closeNavbar();
	}
</script>

<Modal open={!!activeId} {maxWidth} onclose={handleClose}>
	{#if activeId === 'youtube-search'}
		<YouTubeSearchModalContent />
	{:else if activeId === 'youtube'}
		<YouTubeModalContent />
	{:else if activeId === 'libraries'}
		<LibraryModalContent />
	{:else if activeId === 'settings'}
		<SettingsModalContent />
	{:else if activeId === 'db'}
		<DbModalContent />
	{:else if activeId === 'yt-channels'}
		<YtChannelsModalContent />
	{/if}
</Modal>
