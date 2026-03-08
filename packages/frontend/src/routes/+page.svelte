<script lang="ts">
	import classNames from 'classnames';
	import { onMount, onDestroy } from 'svelte';
	import { invalidateAll } from '$app/navigation';
	import { apiUrl } from '$lib/api-base';
	import { mediaDetailService } from '$services/media-detail.service';
	import { libraryService } from '$services/library.service';
	import { modalRouterService } from '$services/modal-router.service';
	import Modal from '$components/core/Modal.svelte';
	import type { MediaDetailCardType } from '$types/media-detail.type';
	import YouTubeLinkModal from '$components/libraries/YouTubeLinkModal.svelte';
	import MediaCard from '$components/media/MediaCard.svelte';
	import MediaListCard from '$components/media/MediaListCard.svelte';
	import MediaDetail from '$components/media/MediaDetail.svelte';
	import type { MediaList, MediaListLink } from '$types/media-list.type';
	import type { LibraryFile } from '$types/library.type';
	import type {
		MediaItem,
		MediaItemLink,
		MediaLinkSource,
		MediaCategory
	} from '$types/media-card.type';
	import type { YouTubeOEmbedResponse } from '$types/youtube.type';

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
			lists: MediaList[];
		};
	}

	const ALL_CATEGORY = '__all__';
	const ALL_TYPE = '__all_type__';
	const LISTS_TYPE = '__lists__';

	let { data }: Props = $props();

	let activeTypeId = $state(ALL_TYPE);
	let activeCategoryId = $state(ALL_CATEGORY);
	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);
	let selectedList: MediaList | null = $state(null);

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});

	// YouTube metadata state
	let youtubeMetadata: Record<string, YouTubeOEmbedResponse> = $state({});
	let youtubeLoading: Set<string> = $state(new Set());

	let metadataLoading = $derived(youtubeLoading);

	// Scan all libraries state
	let scanning = $state(false);

	async function handleScanAll() {
		scanning = true;
		try {
			await libraryService.scanAllLibraries();
			await invalidateAll();
		} finally {
			scanning = false;
		}
	}

	onMount(async () => {
		libraryService.initialize();
	});

	function getItemLinks(item: MediaItem): Record<string, MediaItemLink> {
		const overrides = linkOverrides[item.id];
		if (!overrides) return item.links;
		const merged = { ...item.links };
		for (const [service, link] of Object.entries(overrides)) {
			if (link === null) {
				delete merged[service];
			} else {
				merged[service] = link;
			}
		}
		return merged;
	}

	let isAllType = $derived(activeTypeId === ALL_TYPE);
	let isListsType = $derived(activeTypeId === LISTS_TYPE);

	let activeType = $derived(
		activeTypeId === ALL_TYPE || activeTypeId === LISTS_TYPE
			? activeTypeId
			: activeTypeId || data.mediaTypes[0]?.id || ''
	);

	let categoriesForType = $derived(
		isAllType ? data.categories : data.categories.filter((c) => c.mediaTypeId === activeType)
	);

	let activeCategory = $derived.by(() => {
		if (activeCategoryId === ALL_CATEGORY) return ALL_CATEGORY;
		if (categoriesForType.some((c) => c.id === activeCategoryId)) return activeCategoryId;
		return ALL_CATEGORY;
	});

	let isAllCategoryView = $derived(activeCategory === ALL_CATEGORY);

	let items = $derived.by(() => {
		if (isAllType && isAllCategoryView) {
			return Object.values(data.itemsByType).flat();
		}
		if (isAllType && !isAllCategoryView) {
			return data.itemsByCategory[activeCategory] ?? [];
		}
		if (isAllCategoryView) {
			return data.itemsByType[activeType] ?? [];
		}
		return data.itemsByCategory[activeCategory] ?? [];
	});

	// Apply link overrides to items for card rendering
	let itemsWithOverrides = $derived(
		items.map((item) => {
			const overrides = linkOverrides[item.id];
			if (!overrides) return item;
			const merged = { ...item.links };
			for (const [service, link] of Object.entries(overrides)) {
				if (link === null) {
					delete merged[service];
				} else {
					merged[service] = link;
				}
			}
			return { ...item, links: merged };
		})
	);

	// Media detail selection
	const mediaDetailStore = mediaDetailService.store;
	let selectedItemId = $derived($mediaDetailStore?.item.id ?? null);

	function resolveCardType(item: MediaItem): MediaDetailCardType {
		if (item.links.youtube) return 'youtube';
		if (item.mediaTypeId === 'audio') return 'audio';
		if (item.mediaTypeId === 'image') return 'image';
		return 'video';
	}

	function handlePlay(item: MediaItem) {
		const streamUrl = apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/stream`);
		window.open(streamUrl, '_blank');
	}

	function handleSelect(item: MediaItem) {
		mediaDetailService.select({
			item,
			cardType: resolveCardType(item),
			youtubeMetadata: youtubeMetadata[item.id] ?? null,
			onplay: (i) => handlePlay(i),
			onlink: (i, service) => {
				linkModalItem = i;
				linkModalService = service;
			},
			onunlink: (i, service) => handleUnlink(i, service)
		});
		modalRouterService.openMediaDetail(item.mediaTypeId, item.categoryId ?? '', item.id);
	}

	// Sync metadata updates into the active selection
	$effect(() => {
		const sel = $mediaDetailStore;
		if (!sel) return;
		const id = sel.item.id;
		const updatedItem = itemsWithOverrides.find((i) => i.id === id);
		if (!updatedItem) return;
		const newYt = youtubeMetadata[id] ?? null;
		if (newYt !== sel.youtubeMetadata || updatedItem !== sel.item) {
			mediaDetailService.select({
				...sel,
				item: updatedItem,
				cardType: resolveCardType(updatedItem),
				youtubeMetadata: newYt
			});
		}
	});

	function closeMediaDetail() {
		mediaDetailService.clear();
		modalRouterService.closeMediaDetail();
	}

	onDestroy(() => {
		mediaDetailService.clear();
	});

	// Deep-link restoration: open media detail from URL params on load
	const routerStore = modalRouterService.store;
	let deepLinkRestored = $state(false);

	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail || deepLinkRestored) return;
		deepLinkRestored = true;
		const allItems = Object.values(data.itemsByType).flat();
		const item = allItems.find((i) => i.id === detail.id);
		if (item) {
			handleSelect(item);
		}
	});

	// Sync router popstate back to media detail
	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail && $mediaDetailStore) {
			mediaDetailService.clear();
		}
	});

	function selectType(id: string) {
		activeTypeId = id;
		activeCategoryId = ALL_CATEGORY;
		selectedList = null;
		closeMediaDetail();
	}

	function selectCategory(id: string) {
		activeCategoryId = id;
		closeMediaDetail();
	}

	function updateItemLinks(itemId: string, service: string, link: MediaItemLink | null) {
		linkOverrides = {
			...linkOverrides,
			[itemId]: {
				...linkOverrides[itemId],
				[service]: link
			}
		};
	}

	async function handleYoutubeLink(youtubeId: string) {
		if (!linkModalItem) return;
		const item = linkModalItem;

		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/youtube`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ youtubeId })
		});

		if (res.ok) {
			updateItemLinks(item.id, 'youtube', {
				serviceId: youtubeId,
				seasonNumber: null,
				episodeNumber: null
			});
		}

		linkModalItem = null;
		linkModalService = null;
	}

	async function handleUnlink(item: MediaItem, service: string) {
		const res = await fetch(
			apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`),
			{
				method: 'DELETE'
			}
		);

		if (res.ok) {
			updateItemLinks(item.id, service, null);
			if (service === 'youtube') {
				const { [item.id]: _, ...rest } = youtubeMetadata;
				youtubeMetadata = rest;
			}
		}
	}

	function itemAsLibraryFile(item: MediaItem): LibraryFile {
		return {
			id: item.id,
			name: item.name,
			path: item.path,
			extension: item.extension,
			mediaType: item.mediaTypeId as LibraryFile['mediaType'],
			categoryId: item.categoryId,
			links: getItemLinks(item)
		};
	}

	async function fetchYoutubeMetadata(item: MediaItem) {
		const youtubeLink = item.links.youtube;
		if (!youtubeLink || youtubeMetadata[item.id] || youtubeLoading.has(item.id)) return;

		youtubeLoading = new Set([...youtubeLoading, item.id]);

		try {
			const res = await fetch(apiUrl(`/api/youtube/oembed?videoId=${youtubeLink.serviceId}`));
			if (res.ok) {
				youtubeMetadata[item.id] = await res.json();
			}
		} catch (e) {
			console.error('Failed to load YouTube metadata:', e);
		} finally {
			const next = new Set(youtubeLoading);
			next.delete(item.id);
			youtubeLoading = next;
		}
	}

	$effect(() => {
		for (const item of itemsWithOverrides) {
			if (item.links.youtube) {
				fetchYoutubeMetadata(item);
			}
		}
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Media</h1>
			<p class="text-sm opacity-70">Browse your media library</p>
		</div>
		<button class="btn btn-sm btn-accent" onclick={handleScanAll} disabled={scanning}>
			{#if scanning}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Scan
			{/if}
		</button>
	</div>

	<!-- Tier 1: All + Media Types -->
	<div class="mb-3 flex flex-wrap gap-2">
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isAllType,
				'btn-ghost': !isAllType
			})}
			onclick={() => selectType(ALL_TYPE)}
		>
			All
		</button>
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isListsType,
				'btn-ghost': !isListsType
			})}
			onclick={() => selectType(LISTS_TYPE)}
		>
			Lists
		</button>
		{#each data.mediaTypes as type}
			<button
				class={classNames('btn btn-sm', {
					'btn-primary': activeType === type.id,
					'btn-ghost': activeType !== type.id
				})}
				onclick={() => selectType(type.id)}
			>
				{type.label}
			</button>
		{/each}
	</div>

	<!-- Tier 2: All + Categories for selected type (hidden when Lists tab active) -->
	{#if !isListsType && categoriesForType.length > 0}
		<div class="mb-6 flex flex-wrap gap-2">
			<button
				class={classNames('btn btn-xs', {
					'btn-secondary': isAllCategoryView,
					'btn-ghost': !isAllCategoryView
				})}
				onclick={() => selectCategory(ALL_CATEGORY)}
			>
				All
			</button>
			{#each categoriesForType as category}
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': activeCategory === category.id,
						'btn-ghost': activeCategory !== category.id
					})}
					onclick={() => selectCategory(category.id)}
				>
					{category.label}
				</button>
			{/each}
		</div>
	{/if}

	{#if isListsType}
		<!-- Lists view -->
		{#if selectedList}
			<div class="mb-4 flex items-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => {
						selectedList = null;
					}}
				>
					&larr; Back
				</button>
				<h2 class="text-xl font-semibold">{selectedList.title}</h2>
			</div>
			{#if selectedList.items.length > 0}
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each selectedList.items as item (item.id)}
						<MediaCard
							{item}
							youtubeMetadata={youtubeMetadata[item.id] ?? null}
							metadataLoading={metadataLoading.has(item.id)}
							selected={selectedItemId === item.id}
							onselect={(i) => handleSelect(i)}
						/>
					{/each}
				</div>
			{:else}
				<div class="rounded-lg bg-base-200 p-8 text-center">
					<p class="opacity-50">No items in this list.</p>
				</div>
			{/if}
		{:else if data.lists.length > 0}
			<div
				class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each data.lists as list (list.id)}
					<MediaListCard
						{list}
						onselect={(l) => {
							selectedList = l;
						}}
					/>
				{/each}
			</div>
		{:else}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">
					No lists yet. Scan a library with directories containing multiple audio or video files.
				</p>
			</div>
		{/if}
	{:else}
		<!-- Items grid -->
		{#if itemsWithOverrides.length > 0}
			<div
				class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each itemsWithOverrides as item (item.id)}
					<MediaCard
						{item}
						youtubeMetadata={youtubeMetadata[item.id] ?? null}
						metadataLoading={metadataLoading.has(item.id)}
						selected={selectedItemId === item.id}
						onselect={(i) => handleSelect(i)}
					/>
				{/each}
			</div>
		{:else}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No items in this category.</p>
			</div>
		{/if}
	{/if}
</div>

<Modal open={!!$mediaDetailStore} maxWidth="max-w-lg" onclose={closeMediaDetail}>
	{#if $mediaDetailStore}
		<MediaDetail selection={$mediaDetailStore} onclose={closeMediaDetail} />
	{/if}
</Modal>

{#if linkModalItem && linkModalService === 'youtube'}
	<YouTubeLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		onlink={handleYoutubeLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}
