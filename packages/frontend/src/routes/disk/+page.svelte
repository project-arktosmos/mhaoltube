<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { apiUrl } from '$lib/api-base';
	import type { LibraryFs, LibraryFsEntry } from '$types/library.type';

	let fsData = $state<LibraryFs | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
	}

	function sumSize(entries: LibraryFsEntry[]): number {
		return entries.reduce((acc, e) => acc + e.size, 0);
	}

	let audioTotal = $derived(fsData ? sumSize(fsData.audio) : 0);
	let videoTotal = $derived(fsData ? sumSize(fsData.video) : 0);
	let grandTotal = $derived(audioTotal + videoTotal);

	onMount(async () => {
		try {
			const res = await fetch(apiUrl('/api/libraries/fs'));
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
			}
			fsData = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="mx-auto max-w-3xl p-6">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Disk</h1>
		<p class="text-sm opacity-70">Library filesystem usage</p>
		{#if fsData}
			<div class="mt-1 flex items-center gap-2">
				<p class="truncate font-mono text-xs opacity-50">{fsData.path}</p>
				<button class="btn btn-xs btn-ghost opacity-60" onclick={() => invoke('open_path', { path: fsData!.path })}>
					open
				</button>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if error}
		<div class="alert alert-error">
			<span>{error}</span>
		</div>
	{:else if fsData}
		{@const sections: { label: string; entries: LibraryFsEntry[]; total: number }[] = [
			{ label: 'audio/', entries: fsData.audio, total: audioTotal },
			{ label: 'video/', entries: fsData.video, total: videoTotal }
		]}

		{#each sections as section (section.label)}
			<div class="card mb-4 bg-base-200">
				<div class="card-body p-4">
					<div class="mb-2 flex items-center justify-between">
						<h2 class="font-mono font-semibold">{section.label}</h2>
						<span class="badge badge-neutral">{formatBytes(section.total)}</span>
					</div>

					{#if section.entries.length === 0}
						<p class="text-sm opacity-50">No files</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="table w-full table-sm">
								<thead>
									<tr>
										<th class="text-left">File</th>
										<th class="text-right">Size</th>
									</tr>
								</thead>
								<tbody>
									{#each section.entries as entry (entry.name)}
										<tr>
											<td class="font-mono text-xs">{entry.name}</td>
											<td class="text-right font-mono text-xs opacity-70"
												>{formatBytes(entry.size)}</td
											>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</div>
			</div>
		{/each}

		<div class="card bg-base-300">
			<div class="card-body flex-row items-center justify-between p-4">
				<span class="font-semibold">Total</span>
				<span class="font-mono text-lg font-bold">{formatBytes(grandTotal)}</span>
			</div>
		</div>
	{/if}
</div>
