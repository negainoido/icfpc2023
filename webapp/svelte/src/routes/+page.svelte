<script>
    import { onMount } from 'svelte';

    let wasm;
    let problem_id = 1;
    let problem = null;
    let solution_id = 1;
    let solution = null;
    let records = [];
    let filteredRecords = [];

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
        fetch(`https://icfpc2023.negainoido.com/api/problem?problem_id=${problem_id}`)
            .then(data => data.json())
            .then(data => {
                problem = data;
            });
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
                solution = JSON.parse(contents);
                draw();
            });
    }

    function draw() {
        let canvas = document.getElementById('c').getContext('2d');
        let width = 1600;
        let height = 1200;
        let scale = Math.min(width / problem.room_width, height / problem.room_height);
        canvas.clearRect(0, 0, width, height);
        canvas.strokeRect(
            0,
            0,
            problem.room_width * scale,
            problem.room_height * scale
        );
        canvas.fillStyle = '#ddd';
        canvas.fillRect(
            scale * problem.stage_bottom_left[0],
            scale * problem.stage_bottom_left[1],
            scale * problem.stage_width,
            scale * problem.stage_height
        );
        canvas.strokeStyle = '#a11';
        for (let a of problem.attendees) {
            canvas.beginPath();
            canvas.arc(a.x * scale, a.y * scale, 1.2, 0, 7, false)
            canvas.stroke();
        }
        canvas.strokeStyle = '#11a';
        for (let m of solution.placements) {
            canvas.beginPath();
            canvas.arc(m.x * scale, m.y * scale, 1.2, 0, 7, false)
            canvas.stroke();
        }


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
    <canvas id="c" width="1600" height="1200" />
</div>
