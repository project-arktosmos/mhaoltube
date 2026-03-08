<script lang="ts">
	import classNames from 'classnames';
	import { modalRouterService } from '$services/modal-router.service';
	import ThemeToggle from '$components/core/ThemeToggle.svelte';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	let wrapperClasses = $derived(classNames('navbar bg-base-100 shadow-sm', classes));
</script>

<nav class={wrapperClasses}>
	<div class="flex-1">
		<a href="/" class="btn text-xl btn-ghost">Mhaol</a>
	</div>

	<div class="flex-none">
		<!-- Desktop: horizontal buttons -->
		<div class="hidden gap-1 lg:flex">
			<button class="btn btn-sm btn-info" onclick={() => modalRouterService.openNavbar('youtube')}>
				YouTube
			</button>
			<button
				class="btn btn-outline btn-sm btn-info"
				onclick={() => modalRouterService.openNavbar('youtube-search')}
			>
				YT Search
			</button>
			<button
				class="btn btn-sm btn-neutral"
				onclick={() => modalRouterService.openNavbar('libraries')}
			>
				Libraries
			</button>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => modalRouterService.openNavbar('settings')}
			>
				Settings
			</button>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => modalRouterService.openNavbar('db')}
			>
				DB
			</button>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => modalRouterService.openNavbar('yt-channels')}
			>
				YT Channels
			</button>
			<ThemeToggle />
		</div>

		<!-- Mobile: burger menu -->
		<div class="dropdown dropdown-end lg:hidden">
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
			<ul
				tabindex="0"
				class="dropdown-content menu z-50 mt-3 w-52 rounded-box bg-base-200 p-2 shadow"
			>
				<li><button onclick={() => modalRouterService.openNavbar('youtube')}>YouTube</button></li>
				<li>
					<button onclick={() => modalRouterService.openNavbar('youtube-search')}>YT Search</button>
				</li>
				<li>
					<button onclick={() => modalRouterService.openNavbar('libraries')}>Libraries</button>
				</li>
				<li><button onclick={() => modalRouterService.openNavbar('settings')}>Settings</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('db')}>DB</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('yt-channels')}>YT Channels</button></li>
				<li><ThemeToggle /></li>
			</ul>
		</div>
	</div>
</nav>
