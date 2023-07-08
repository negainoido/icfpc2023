<script>
    import { onMount } from 'svelte';

    let wasm;
    let problem_id = 1;
    let solution_id = 1;
    let records = [];
    let filteredRecords = [];
    let solution = null;

    function updateAddition() {
        if (!wasm) return; // failed
    }

    /// 良いレコード全部取得
    function fetchRecords() {
        fetch('https://icfpc2023.negainoido.com/api/solutions/show')
            .then(data => data.json())
            .then(data => {
                records = data;
                filterRecords();
            })
    }

    /// problem_id ごとの結果を表示する
    function filterRecords() {
        if (!records) return;
        filteredRecords = [];
        for (let r of records) {
            if (r[1] === problem_id) {
                filteredRecords.push(r);
            }
        }
        solution = null;
    }

    function fetchSolution(solution_id) {
        fetch(`https://icfpc2023.negainoido.com/api/solutions?id=${solution_id}`)
            .then(response => response.json())
            .then(response => {
                if (response['message'] === 'not found') {
                    alert('not found');
                    solution = null;
                    return;
                }
                let contents = response['contents'];
                console.log(contents);
                solution = JSON.parse(contents);
                draw();
            });
    }

    function draw() {
        let canvas = document.getElementById('c').getContext('2d');
        let width = 1200;
        let height = 800;
        canvas.clearRect(0, 0, width, height);
    }

    onMount(async () => {
        fetchRecords();
        wasm = await import('wasm-sample');
        await wasm.default();
    });
</script>

<label>problem_id</label>
<input type='number' bind:value={problem_id} on:change={filterRecords} />

<div>
<p>{filteredRecords.length} records</p>
{#if filteredRecords.length > 0}
    <table>
        <tr>
                <th>id</th>
                <th>problem_id</th>
                <th>submission_id</th>
                <th>solver</th>
                <th>status</th>
                <th>score</th>
                <th>ts</th>
        </tr>
        {#each filteredRecords as r}
            <tr>
                <td><button on:click={fetchSolution(r[0])}>{r[0]}</button></td>
                <td>{r[1]}</td>
                <td>{r[2]}</td>
                <td>{r[3]}</td>
                <td>{r[4]}</td>
                <td>{r[5]}</td>
                <td>{r[6]}</td>
            </tr>
        {/each}
    </table>
{/if}
</div>

<div>
  <canvas id="c" width="1200" height="800" style='border:1px black solid' />
</div>
