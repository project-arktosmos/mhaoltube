<script lang="ts">
	import '../css/app.css';
	import '$services/i18n';
	import { themeService } from '$services/theme.service';
	import Navbar from '$components/core/Navbar.svelte';
	import ModalOutlet from '$components/core/ModalOutlet.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar />
	<main class="min-w-0 flex-1 bg-base-300">
		{@render children?.()}
	</main>
</div>

<ModalOutlet />
