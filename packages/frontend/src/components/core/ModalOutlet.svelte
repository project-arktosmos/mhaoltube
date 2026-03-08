<script lang="ts">
	import Modal from '$components/core/Modal.svelte';
	import { modalRouterService } from '$services/modal-router.service';
	import YouTubeModalContent from '$components/youtube/YouTubeModalContent.svelte';
	import LibraryModalContent from '$components/libraries/LibraryModalContent.svelte';
	import SettingsModalContent from '$components/settings/SettingsModalContent.svelte';
	import YouTubeSearchModalContent from '$components/youtube-search/YouTubeSearchModalContent.svelte';

	const routerStore = modalRouterService.store;

	const MAX_WIDTHS: Record<string, string> = {
		youtube: 'max-w-6xl',
		'youtube-search': 'max-w-5xl',
		libraries: 'max-w-5xl',
		settings: 'max-w-2xl'
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
	{/if}
</Modal>
