<script>
    import { onMount } from 'svelte';

    let wasm;
    let problem_id = 1;
    let problem = null;
    let solution_id = 1;
    let solution = null;
    let records = [];
    let filteredRecords = [];

    let colors = [
        '#11ff11',
        '#11dd33',
        '#11bb55',
        '#119977',
        '#117799',
        '#1155bb',
        '#1133dd',
        '#1111ff',
        '#33ff11',
        '#33dd33',
        '#33bb55',
        '#339977',
        '#337799',
        '#3355bb',
        '#3333dd',
        '#3311ff',
        '#55ff11',
        '#55dd33',
        '#55bb55',
        '#559977',
        '#557799',
        '#5555bb',
        '#5533dd',
        '#5511ff',
        '#77ff11',
        '#77dd33',
        '#77bb55',
        '#779977',
        '#777799',
        '#7755bb',
        '#7733dd',
        '#7711ff',
    ]

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
        canvas.fillStyle = '#999';
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
        for (let i = 0; i < solution.placements.length; ++i) {
            let m = solution.placements[i];
            let inst = problem.musicians[i];
            canvas.fillStyle = colors[inst % colors.length];
            canvas.beginPath();
            canvas.arc(m.x * scale, m.y * scale, 1.6, 0, 7, false)
            canvas.fill();
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
