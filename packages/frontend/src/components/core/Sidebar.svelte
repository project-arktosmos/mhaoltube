<script lang="ts">
	import classNames from 'classnames';
	import { page } from '$app/stores';
	import routes from 'virtual:routes';
	import type { RouteEntry } from '$types/route.type';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	let navigableRoutes = $derived(routes.filter((r: RouteEntry) => !r.isDynamic));

	function isActive(route: RouteEntry): boolean {
		if (route.path === '/') return $page.url.pathname === '/';
		return $page.url.pathname === route.path || $page.url.pathname.startsWith(route.path + '/');
	}

	let wrapperClasses = $derived(classNames('menu w-56 bg-base-200 p-4 hidden lg:flex', classes));
</script>

<aside class={wrapperClasses}>
	<ul class="menu gap-1">
		{#each navigableRoutes as route (route.path)}
			{@const active = isActive(route)}
			{@const navigableChildren = route.children.filter((c: RouteEntry) => !c.isDynamic)}
			{#if navigableChildren.length > 0}
				<li>
					<a href={route.path} class={classNames({ active })}>
						{route.label}
					</a>
					<ul>
						{#each navigableChildren as child (child.path)}
							<li>
								<a
									href={child.path}
									class={classNames({ active: $page.url.pathname === child.path })}
								>
									{child.label}
								</a>
							</li>
						{/each}
					</ul>
				</li>
			{:else}
				<li>
					<a href={route.path} class={classNames({ active })}>
						{route.label}
					</a>
				</li>
			{/if}
		{/each}
	</ul>
</aside>
