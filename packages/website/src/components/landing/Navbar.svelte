<script lang="ts">
	import ThemeToggle from './ThemeToggle.svelte';
</script>

<nav class="navbar sticky top-0 z-50 bg-primary text-primary-content shadow-sm">
	<div class="flex-1">
		<a href="/" class="btn text-xl font-bold btn-ghost">Mhaoltube</a>
	</div>
	<div class="flex-none">
		<ThemeToggle />
	</div>
</nav>
