<script lang="ts">
	import { ThemeColors } from '$types/core.type';

	interface ColorEntry {
		color: ThemeColors;
		label: string;
		bg: string;
		contentBg: string;
		contentText: string;
		btnClass: string;
		badgeClass: string;
	}

	const colors: ColorEntry[] = [
		{
			color: ThemeColors.Primary,
			label: 'Primary',
			bg: 'bg-primary',
			contentBg: 'bg-primary-content',
			contentText: 'text-primary-content',
			btnClass: 'btn btn-sm btn-primary',
			badgeClass: 'badge badge-primary'
		},
		{
			color: ThemeColors.Secondary,
			label: 'Secondary',
			bg: 'bg-secondary',
			contentBg: 'bg-secondary-content',
			contentText: 'text-secondary-content',
			btnClass: 'btn btn-sm btn-secondary',
			badgeClass: 'badge badge-secondary'
		},
		{
			color: ThemeColors.Accent,
			label: 'Accent',
			bg: 'bg-accent',
			contentBg: 'bg-accent-content',
			contentText: 'text-accent-content',
			btnClass: 'btn btn-sm btn-accent',
			badgeClass: 'badge badge-accent'
		},
		{
			color: ThemeColors.Neutral,
			label: 'Neutral',
			bg: 'bg-neutral',
			contentBg: 'bg-neutral-content',
			contentText: 'text-neutral-content',
			btnClass: 'btn btn-sm btn-neutral',
			badgeClass: 'badge badge-neutral'
		},
		{
			color: ThemeColors.Success,
			label: 'Success',
			bg: 'bg-success',
			contentBg: 'bg-success-content',
			contentText: 'text-success-content',
			btnClass: 'btn btn-sm btn-success',
			badgeClass: 'badge badge-success'
		},
		{
			color: ThemeColors.Warning,
			label: 'Warning',
			bg: 'bg-warning',
			contentBg: 'bg-warning-content',
			contentText: 'text-warning-content',
			btnClass: 'btn btn-sm btn-warning',
			badgeClass: 'badge badge-warning'
		},
		{
			color: ThemeColors.Error,
			label: 'Error',
			bg: 'bg-error',
			contentBg: 'bg-error-content',
			contentText: 'text-error-content',
			btnClass: 'btn btn-sm btn-error',
			badgeClass: 'badge badge-error'
		},
		{
			color: ThemeColors.Info,
			label: 'Info',
			bg: 'bg-info',
			contentBg: 'bg-info-content',
			contentText: 'text-info-content',
			btnClass: 'btn btn-sm btn-info',
			badgeClass: 'badge badge-info'
		}
	];

	const baseColors = [
		{ label: 'Base 100', bg: 'bg-base-100', text: 'text-base-content' },
		{ label: 'Base 200', bg: 'bg-base-200', text: 'text-base-content' },
		{ label: 'Base 300', bg: 'bg-base-300', text: 'text-base-content' },
		{ label: 'Base Content', bg: 'bg-base-content', text: 'text-base-100' }
	];
</script>

<div class="mx-auto max-w-3xl p-6">
	<h3 class="text-lg font-bold">Theme</h3>
	<p class="text-sm text-base-content/60">Active DaisyUI theme color palette</p>

	<div class="card mt-6 bg-base-200">
		<div class="card-body gap-3">
			<h2 class="card-title text-base">Semantic Colors</h2>
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
				{#each colors as entry (entry.color)}
					<div class="flex flex-col overflow-hidden rounded-lg">
						<div class={`${entry.bg} flex h-16 items-center justify-center`}>
							<span class={`text-sm font-semibold ${entry.contentText}`}>{entry.label}</span>
						</div>
						<div class={`${entry.contentBg} flex h-8 items-center justify-center`}>
							<span class="text-xs opacity-70">{entry.label} Content</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<div class="card mt-4 bg-base-200">
		<div class="card-body gap-3">
			<h2 class="card-title text-base">Base Colors</h2>
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
				{#each baseColors as entry (entry.label)}
					<div class="flex flex-col overflow-hidden rounded-lg border border-base-content/10">
						<div class={`${entry.bg} flex h-16 items-center justify-center`}>
							<span class={`text-sm font-semibold ${entry.text}`}>{entry.label}</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<div class="card mt-4 bg-base-200">
		<div class="card-body gap-3">
			<h2 class="card-title text-base">Components</h2>
			<div class="flex flex-wrap gap-2">
				{#each colors as entry (entry.color)}
					<button class={entry.btnClass}>{entry.label}</button>
				{/each}
			</div>
			<div class="flex flex-wrap gap-2">
				{#each colors as entry (entry.color)}
					<span class={entry.badgeClass}>{entry.label}</span>
				{/each}
			</div>
		</div>
	</div>
</div>
