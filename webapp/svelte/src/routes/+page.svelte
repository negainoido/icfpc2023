<script>
	import { onMount } from 'svelte';
	onMount(async () => {
		const wasm = await import('wasm-sample');
		await wasm.default();
		console.log(wasm.add(1, 2));
	});
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
