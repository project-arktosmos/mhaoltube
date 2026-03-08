<script lang="ts">
	import { onMount } from 'svelte';
	import { libraryService } from '$services/library.service';
	import LibraryFileList from '$components/libraries/LibraryFileList.svelte';

	const library = libraryService.library;
	const libState = libraryService.state;

	onMount(() => {
		libraryService.initialize();
	});
</script>

<div class="mx-auto max-w-5xl p-6">
	<div class="flex items-center justify-between">
		<div>
			<h3 class="text-lg font-bold">Library</h3>
			<p class="text-sm text-base-content/60">Your downloaded YouTube content</p>
		</div>
	</div>

	{#if $library}
		<div class="mt-4">
			<p class="truncate font-mono text-xs text-base-content/50">{$library.path}</p>

			<LibraryFileList
				content={$libState.content}
				loading={$libState.contentLoading}
				error={$libState.contentError}
			/>
		</div>
	{:else}
		<div class="mt-4 flex justify-center py-8">
			<span class="loading loading-sm loading-spinner"></span>
		</div>
	{/if}
</div>
