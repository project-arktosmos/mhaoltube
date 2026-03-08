<script lang="ts">
	import { _ } from 'svelte-i18n';
	import ThemeToggle from './ThemeToggle.svelte';
</script>

<nav class="navbar sticky top-0 z-50 bg-base-100/80 shadow-sm backdrop-blur-md">
	<div class="flex-1">
		<a href="/" class="btn btn-ghost text-xl font-bold">Mhaoltube</a>
	</div>
	<div class="flex-none">
		<div class="hidden gap-1 sm:flex">
			<a href="#features" class="btn btn-ghost btn-sm">{$_('nav.features')}</a>
			<a href="#platforms" class="btn btn-ghost btn-sm">{$_('nav.platforms')}</a>
			<a
				href="https://github.com/arktosmos/mhaoltube"
				class="btn btn-ghost btn-sm"
				target="_blank"
				rel="noopener noreferrer"
			>
				{$_('nav.github')}
			</a>
			<ThemeToggle />
		</div>
		<div class="dropdown dropdown-end sm:hidden">
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<div tabindex="0" role="button" class="btn btn-ghost">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="h-6 w-6"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
					/>
				</svg>
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<ul tabindex="0" class="dropdown-content menu z-50 mt-3 w-52 rounded-box bg-base-200 p-2 shadow">
				<li><a href="#features">{$_('nav.features')}</a></li>
				<li><a href="#platforms">{$_('nav.platforms')}</a></li>
				<li>
					<a href="https://github.com/arktosmos/mhaoltube" target="_blank" rel="noopener noreferrer">
						{$_('nav.github')}
					</a>
				</li>
				<li><ThemeToggle /></li>
			</ul>
		</div>
	</div>
</nav>
