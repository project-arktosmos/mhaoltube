<script lang="ts">
	import classNames from 'classnames';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	let wrapperClasses = $derived(classNames('navbar bg-base-100 shadow-sm', classes));
</script>

<nav class={wrapperClasses}>
	<div class="flex-1">
		<a href="/" class="btn text-xl btn-ghost">Mhaol<span class="text-primary">Tube</span></a>
	</div>
</nav>
