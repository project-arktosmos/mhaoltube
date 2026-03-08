<script lang="ts">
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';

	interface TableSummary {
		name: string;
		columns: { name: string; type: string }[];
		rowCount: number;
	}

	interface TableDetailResponse {
		table: string;
		columns: { name: string; type: string }[];
		rows: Record<string, unknown>[];
		pagination: {
			page: number;
			limit: number;
			total: number;
			totalPages: number;
		};
	}

	let tables: TableSummary[] = $state([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let selectedTable = $state<string | null>(null);
	let tableData = $state<TableDetailResponse | null>(null);
	let tableLoading = $state(false);

	let resetting = $state(false);
	let resetError = $state<string | null>(null);

	async function fetchTables() {
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl('/api/database/tables'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			tables = await res.json();
			if (tables.length > 0 && !selectedTable) {
				selectedTable = tables[0].name;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function fetchTableData(name: string, page: number = 1) {
		tableLoading = true;
		try {
			const res = await fetch(apiUrl(`/api/database/tables/${name}?page=${page}&limit=20`));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			tableData = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			tableLoading = false;
		}
	}

	$effect(() => {
		fetchTables();
	});

	$effect(() => {
		if (selectedTable) {
			fetchTableData(selectedTable);
		}
	});

	function selectTable(name: string) {
		selectedTable = name;
	}

	function goToPage(page: number) {
		if (selectedTable) {
			fetchTableData(selectedTable, page);
		}
	}

	async function handleReset() {
		resetting = true;
		resetError = null;
		try {
			const res = await fetch(apiUrl('/api/database/reset'), { method: 'POST' });
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
			}
			selectedTable = null;
			tableData = null;
			await fetchTables();
		} catch (e) {
			resetError = e instanceof Error ? e.message : String(e);
		} finally {
			resetting = false;
		}
	}

	function formatCellValue(value: unknown): string {
		if (value === null || value === undefined) return 'NULL';
		if (typeof value === 'object') return JSON.stringify(value);
		return String(value);
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Database Inspector</h3>
	<p class="text-sm text-base-content/60">Browse and inspect SQLite tables</p>
</div>

{#if error}
	<div class="mt-4 alert alert-error">
		<span>{error}</span>
		<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
	</div>
{/if}

{#if loading}
	<div class="mt-6 flex justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<!-- Table tabs -->
	<div class="mt-4 flex flex-wrap gap-1">
		{#each tables as table}
			<button
				class={classNames('btn btn-xs', {
					'btn-primary': selectedTable === table.name,
					'btn-ghost': selectedTable !== table.name
				})}
				onclick={() => selectTable(table.name)}
			>
				{table.name}
				<span class="badge badge-xs">{table.rowCount}</span>
			</button>
		{/each}
	</div>

	<!-- Table data -->
	{#if tableData}
		<div class="mt-4 overflow-x-auto rounded-lg border border-base-300">
			{#if tableLoading}
				<div class="flex justify-center p-8">
					<span class="loading loading-md loading-spinner"></span>
				</div>
			{:else}
				<table class="table w-full table-zebra table-xs">
					<thead>
						<tr>
							{#each tableData.columns as col}
								<th class="whitespace-nowrap"
									>{col.name}<span class="ml-1 text-xs opacity-40">{col.type}</span></th
								>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each tableData.rows as row}
							<tr>
								{#each tableData.columns as col}
									<td class="max-w-xs truncate" title={formatCellValue(row[col.name])}>
										{formatCellValue(row[col.name])}
									</td>
								{/each}
							</tr>
						{:else}
							<tr>
								<td colspan={tableData.columns.length} class="text-center opacity-50"> No rows </td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</div>

		<!-- Pagination -->
		{#if tableData.pagination.totalPages > 1}
			<div class="mt-3 flex items-center justify-center gap-2">
				<button
					class="btn btn-ghost btn-xs"
					disabled={tableData.pagination.page <= 1}
					onclick={() => goToPage(tableData!.pagination.page - 1)}
				>
					Prev
				</button>
				<span class="text-sm">
					Page {tableData.pagination.page} of {tableData.pagination.totalPages}
					<span class="opacity-50">({tableData.pagination.total} rows)</span>
				</span>
				<button
					class="btn btn-ghost btn-xs"
					disabled={tableData.pagination.page >= tableData.pagination.totalPages}
					onclick={() => goToPage(tableData!.pagination.page + 1)}
				>
					Next
				</button>
			</div>
		{/if}
	{/if}

	<!-- Reset section -->
	<div class="card mt-6 bg-base-200">
		<div class="card-body p-4">
			{#if resetError}
				<div class="alert-sm mb-2 alert alert-error">
					<span>{resetError}</span>
					<button class="btn btn-ghost btn-xs" onclick={() => (resetError = null)}>x</button>
				</div>
			{/if}
			<div class="flex items-center justify-between">
				<div>
					<h3 class="font-semibold text-error">Reset Database</h3>
					<p class="text-sm opacity-70">Drop all tables, recreate schema, and reseed data</p>
				</div>
				<button class="btn btn-sm btn-error" disabled={resetting} onclick={handleReset}>
					{#if resetting}
						<span class="loading loading-sm loading-spinner"></span>
					{:else}
						Reset
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}
