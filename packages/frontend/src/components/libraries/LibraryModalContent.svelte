<script lang="ts">
	import { onMount } from 'svelte';
	import { libraryService } from '$services/library.service';
	import type { LibraryFile } from '$types/library.type';
	import LibraryFileList from './LibraryFileList.svelte';

	const library = libraryService.library;
	const libState = libraryService.state;

	onMount(() => {
		libraryService.initialize();
	});

	function handleScan() {
		libraryService.scanFiles();
	}

	function handleYoutubeLink(file: LibraryFile, youtubeId: string) {
		libraryService.linkYoutube(file.id, youtubeId);
	}

	function handleYoutubeUnlink(file: LibraryFile) {
		libraryService.unlinkYoutube(file.id);
	}

	async function handleEditType(file: LibraryFile, mediaType: string, categoryId: string | null) {
		if (mediaType !== file.mediaType) {
			await libraryService.updateMediaType(file.id, mediaType);
		}
		if (categoryId !== file.categoryId) {
			if (categoryId) {
				await libraryService.updateCategory(file.id, categoryId);
			} else {
				await libraryService.clearCategory(file.id);
			}
		}
	}
</script>

<div class="flex items-center justify-between pr-8">
	<div>
		<h3 class="text-lg font-bold">Library</h3>
		<p class="text-sm text-base-content/60">Manage your media library files</p>
	</div>
</div>

{#if $library}
	<div class="mt-4">
		<p class="truncate font-mono text-xs text-base-content/50">{$library.path}</p>

		<LibraryFileList
			files={$libState.files}
			loading={$libState.filesLoading}
			error={$libState.filesError}
			onscan={handleScan}
			onyoutubelink={(f, ytId) => handleYoutubeLink(f, ytId)}
			onyoutubeunlink={(f) => handleYoutubeUnlink(f)}
			onedittype={(f, mediaType, categoryId) => handleEditType(f, mediaType, categoryId)}
		/>
	</div>
{:else}
	<div class="mt-4 flex justify-center py-8">
		<span class="loading loading-sm loading-spinner"></span>
	</div>
{/if}
