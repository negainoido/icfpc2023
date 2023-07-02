<script>
    import { onMount } from 'svelte';
    let wasm;
    let x = 1;
    let y = 2;
    let z;

    onMount(async () => {
        wasm = await import('wasm-sample');
        await wasm.default();
        z = wasm.add(x, y);
    });

    function updateAddition() {
        if (!wasm) return; // failed
        z = wasm.add(x, y);
    }

</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
<div>
    <input type="number" bind:value={x} on:change={updateAddition} /> +
    <input type="number" bind:value={y} on:change={updateAddition} /> =
    <input type="number" bind:value={z} disabled />
</div>
