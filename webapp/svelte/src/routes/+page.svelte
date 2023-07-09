<script>
    import { onMount } from 'svelte';
    import {get, writable} from "svelte/store";

    let wasm;
    let problem_id = 1;
    let solution_id = 1;
    let records = [];
    let filteredRecords = [];

    let state = writable({
        problem: null,
        solution: null,
        colorful: true,
        ruler: false,
        zoom: 1.0,
        plusx: 0.0,
        plusy: 0.0,
    });
    let score = null;

    state.subscribe((value) => {
        if (value.problem) {
            console.log("start draw...");
            draw(
                value.problem,
                value.solution,
                value.colorful,
                value.zoom,
                value.plusx,
                value.plusy,
                value.ruler
            );
            console.log("...finish draw");
        } else {
            console.log("data not ready; cannot draw");
        }
        if (value.problem && value.solution && wasm) {
            if (score) {
                // 計算済み
                return;
            }
            // console.log(value.problem, value.solution);
            score = wasm.calc_score(
                value.problem.room_width,
                value.problem.room_height,
                value.problem.stage_width,
                value.problem.stage_height,
                value.problem.stage_bottom_left,
                value.problem.musicians,
                value.problem.attendees,
                value.solution.placements,
            );
            console.log(score);
        } else {
            console.log('not ready; cannot calc_score');
        }
    });

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
        clear();
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
        clear();
        state.update((prev) => ({
            ...prev,
            problem: null,
            solution: null,
            zoom: 1.0,
            plusx: 0.0,
            plusy: 0.0,
        }));
        for (let r of records) {
            if (r[1] === problem_id) {
                filteredRecords.push(r);
            }
        }
        fetch(`https://icfpc2023.negainoido.com/api/problem?problem_id=${problem_id}`)
            .then(data => data.json())
            .then(data => {
                state.update((prev) => {
                    return {
                        ...prev,
                        problem: data,
                    };
                });
            });
    }

    function fetchSolution(solution_id) {
        clear();
        fetch(`https://icfpc2023.negainoido.com/api/solutions?id=${solution_id}`)
            .then(response => response.json())
            .then(response => {
                if (response['message'] === 'not found') {
                    alert('not found');
                    return;
                }
                let contents = response['contents'];
                let data = JSON.parse(contents);
                state.update((prev) => {
                    return {
                        ...prev,
                        solution: data,
                    };
                });
            });
    }

    function clear() {
        if (!document) return;
        let obj = document.getElementById('c');
        if (!obj) return;
        let canvas = obj.getContext('2d');
        if (!canvas) return;
        let width = 1600;
        let height = 1200;
        canvas.clearRect(0, 0, width, height);
    }

    function draw(problem, solution, colorful, zoom, plusx, plusy, ruler) {
        if (!document) return;
        let obj = document.getElementById('c');
        if (!obj) return;
        let canvas = obj.getContext('2d');
        if (!canvas) return;

        // canvas size
        let width = 1600;
        let height = 1200;

        let minx = 0;
        let miny = 0;
        let maxx = -10000;
        let maxy = -10000;
        console.log("java", width, height, problem);
        for (let m of problem.attendees) {
            minx = Math.min(minx, m.x);
            miny = Math.min(miny, m.y);
            maxx = Math.max(maxx, m.x);
            maxy = Math.max(maxy, m.y);
        }
        if (solution) {
            for (let m of solution.placements) {
                minx = Math.min(minx, m.x);
                miny = Math.min(miny, m.y);
                maxx = Math.max(maxx, m.x);
                maxy = Math.max(maxy, m.y);
            }
        }

        let padding = 10;
        let offsetx = minx - padding + plusx;
        let offsety = miny - padding + plusy;
        let scale = Math.min(width / (maxx - minx + 2 * padding), height / (maxy - miny + 2 * padding));

        canvas.clearRect(0, 0, width, height);
        canvas.strokeStyle = '#000';
        canvas.fillStyle = '#fff';
        canvas.fillRect(
            offsetx,
            offsety,
            zoom * scale * problem.room_width,
            zoom * scale * problem.room_height
        );
        canvas.strokeRect(
            offsetx,
            offsety,
            zoom * scale * problem.room_width,
            zoom * scale * problem.room_height
        );
        // stage
        canvas.fillStyle = '#999';
        canvas.fillRect(
            offsetx + zoom * scale * problem.stage_bottom_left[0],
            offsety + zoom * scale * problem.stage_bottom_left[1],
            zoom * scale * problem.stage_width,
            zoom * scale * problem.stage_height
        );
        // 罫線
        if (ruler) {
            let w = 10;
            canvas.strokeStyle = '#9cc';
            for (let x = 0; x <= problem.room_width; x += w) {
                canvas.beginPath();
                canvas.moveTo(offsetx + zoom * scale * x, offsety);
                canvas.lineTo(offsetx + zoom * scale * x, offsety + zoom * scale * problem.room_height);
                canvas.stroke();
            }
            for (let y = 0; y <= problem.room_width; y += w) {
                canvas.beginPath();
                canvas.moveTo(offsetx, offsety + zoom * scale * y);
                canvas.lineTo(offsetx + zoom * scale * problem.room_width, offsety + zoom * scale * y);
                canvas.stroke();
            }
        }
        // musicians
        if (solution) {
            canvas.fillStyle = '#11a';
            for (let i = 0; i < solution.placements.length; ++i) {
                let m = solution.placements[i];
                let inst = problem.musicians[i];
                if (colorful) {
                    canvas.fillStyle = colors[inst % colors.length];
                }
                canvas.beginPath();
                canvas.arc(
                    offsetx + zoom * scale * m.x,
                    offsety + zoom * scale * m.y,
                    5, 0, 7, false
                );
                canvas.fill();
            }
        }
        // pillars
        canvas.fillStyle = '#ddd';
        canvas.strokeStyle = '#222';
        for (let p of problem.pillars) {
            canvas.beginPath();
            canvas.arc(
                offsetx + zoom * scale * p.center[0],
                offsety + zoom * scale * p.center[1],
                zoom * scale * p.radius,
                0, 7, false
            );
            canvas.fill();
            canvas.stroke();
        }
        // attendees
        canvas.fillStyle = '#a11';
        for (let a of problem.attendees) {
            canvas.beginPath();
            canvas.arc(
                offsetx + zoom * scale * a.x,
                offsety + zoom * scale * a.y,
                1.2, 0, 7, false
            )
            canvas.fill();
        }
    }

    function fullScreen() {
        var canvas = document.getElementById("c");
        if (!canvas) return;
        if (canvas.requestFullscreen) {
            canvas.requestFullscreen();
        } else if (canvas.mozRequestFullScreen) { // Firefox
            canvas.mozRequestFullScreen();
        } else if (canvas.webkitRequestFullscreen) { // Chrome, Safari and Opera
            canvas.webkitRequestFullscreen();
        } else if (canvas.msRequestFullscreen) { // IE/Edge
            canvas.msRequestFullscreen();
        }
    }

    function onKeyDown(e) {
        console.log(e);
        console.log(e.key);
        switch (e.key) {
            case 'c':
                state.update(prev => ({
                    ...prev,
                    colorful: !prev.colorful,
                }));
                break;
            case 'r':
                state.update(prev => ({
                    ...prev,
                    ruler: !prev.ruler,
                }));
                break;
            case 'a':
            case 'h':
                state.update(prev => ({
                    ...prev, 
                    plusx: prev.plusx + 40,
                }));
            break;
            case 'd':
            case 'l':
                state.update(prev => ({
                    ...prev, 
                    plusx: prev.plusx - 40,
                }));
            break;
            case 'w':
            case 'k':
                state.update(prev => ({
                    ...prev, 
                    plusy: prev.plusy + 40,
                }));
            break;
            case 's':
            case 'j':
                state.update(prev => ({
                    ...prev, 
                    plusy: prev.plusy - 40,
                }));
            break;
            case 'q':
                state.update(prev => ({
                    ...prev, 
                    zoom: prev.zoom - 0.1,
                }));
            break;
            case 'e':
                state.update(prev => ({
                    ...prev, 
                    zoom: prev.zoom + 0.1,
                }));
            break;
        }
    }

    onMount(async () => {
        fetchRecords();
        wasm = await import('solver');
        await wasm.default();
    });
</script>

<div class="section">
    <label for="problem_id">problem_id</label>
    <input id="problem_id" type='number' bind:value={problem_id} on:change={filterRecords} />
</div>

<div class="section">
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

<div class="section">
    <div>
        <label>
            <input type='checkbox' bind:checked={$state.colorful} />
            楽器で色を変える (C)
        </label>
        <label>
            <input type='checkbox' bind:checked={$state.ruler} />
            罫線 (幅=10) (R)
        </label>
        <br />
        <button on:click={fullScreen}>全画面表示</button>
        <br />
        <label>
            zoom
            <input type="range" min="0.1" max="10" step="0.1" bind:value={$state.zoom} class="slider" />
            x{$state.zoom}
        </label>
        <br />
        <label>
            x
            <input type="range" min="-10000" max="10000" step="10" bind:value={$state.plusx} class="slider" />
            {#if $state.plusx >= 0}
                +{$state.plusx}
            {:else}
                {$state.plusx}
            {/if}
        </label>
        <br />
        <label>
            y
            <input type="range" min="-10000" max="10000" step="10" bind:value={$state.plusy} class="slider" />
            {#if $state.plusy >= 0}
                +{$state.plusy}
            {:else}
                {$state.plusy}
            {/if}
        </label>
        <div>
            <ul>WASD: 移動</ul>
            <ul>hjkl: 移動</ul>
            <ul>E/Q: 拡大/縮小</ul>
        </div>
    </div>
    <div>
        <canvas id="c" width="1600" height="1200" />
    </div>
</div>

<svelte:window on:keydown={onKeyDown} />

<style>
    div.section {
        padding: 10px;
    }
    input[type=range] {
        width: 50%;
    }
</style>
