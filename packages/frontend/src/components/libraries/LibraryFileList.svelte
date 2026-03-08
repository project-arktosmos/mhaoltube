<script lang="ts">
	import type { LibraryFile } from '$types/library.type';
	import LibraryFileItem from './LibraryFileItem.svelte';
	import YouTubePreviewModal from './YouTubePreviewModal.svelte';
	import MediaTypeCategoryModal from './MediaTypeCategoryModal.svelte';

	interface Props {
		files: LibraryFile[];
		loading: boolean;
		error: string | null;
		onscan: () => void;
		onyoutubelink: (file: LibraryFile, youtubeId: string) => void;
		onyoutubeunlink: (file: LibraryFile) => void;
		onedittype: (file: LibraryFile, mediaType: string, categoryId: string | null) => void;
	}

	let {
		files,
		loading,
		error,
		onscan,
		onyoutubelink,
		onyoutubeunlink,
		onedittype
	}: Props = $props();

	let youtubePreviewFile: LibraryFile | null = $state(null);
	let typeCategoryModalFile: LibraryFile | null = $state(null);

	function openYoutubePreview(file: LibraryFile) {
		youtubePreviewFile = file;
	}

	function closeYoutubePreview() {
		youtubePreviewFile = null;
	}

	function openTypeCategoryModal(file: LibraryFile) {
		typeCategoryModalFile = file;
	}

	function closeTypeCategoryModal() {
		typeCategoryModalFile = null;
	}

	function handleTypeCategorySave(mediaType: string, categoryId: string | null) {
		if (typeCategoryModalFile) {
			onedittype(typeCategoryModalFile, mediaType, categoryId);
			closeTypeCategoryModal();
		}
	}
</script>

<div class="mt-3 border-t border-base-300 pt-3">
	<div class="mb-2 flex items-center justify-between">
		<span class="text-xs text-base-content/50">
			{files.length} file{files.length !== 1 ? 's' : ''}
		</span>
		<button class="btn btn-ghost btn-xs" onclick={onscan} disabled={loading}>
			{#if loading}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Scan
			{/if}
		</button>
	</div>

	{#if loading && files.length === 0}
		<div class="flex justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
		</div>
	{:else if error}
		<div class="rounded-lg bg-error/10 px-3 py-2 text-sm text-error">
			{error}
		</div>
	{:else if files.length === 0}
		<div class="rounded-lg bg-base-300 py-4 text-center">
			<p class="text-sm opacity-50">No media files found</p>
		</div>
	{:else}
		<div class="max-h-96 overflow-y-auto rounded-lg bg-base-100">
			<table class="table w-full table-xs">
				<thead class="sticky top-0 bg-base-100">
					<tr>
						<th>Name</th>
						<th class="w-20">Type</th>
						<th class="w-24">Category</th>
						<th class="w-20">Ext</th>
						<th class="w-28">YouTube</th>
					</tr>
				</thead>
				<tbody>
					{#each files as file (file.path)}
						<LibraryFileItem
							{file}
							onyoutubelink={(f, ytId) => onyoutubelink(f, ytId)}
							onyoutubeunlink={(f) => onyoutubeunlink(f)}
							onyoutubepreview={openYoutubePreview}
							onedittype={openTypeCategoryModal}
						/>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

{#if youtubePreviewFile && youtubePreviewFile.links.youtube}
	<YouTubePreviewModal
		file={youtubePreviewFile}
		videoId={youtubePreviewFile.links.youtube.serviceId}
		onclose={closeYoutubePreview}
	/>
{/if}

{#if typeCategoryModalFile}
	<MediaTypeCategoryModal
		file={typeCategoryModalFile}
		onsave={handleTypeCategorySave}
		onclose={closeTypeCategoryModal}
	/>
{/if}
