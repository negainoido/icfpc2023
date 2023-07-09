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

    async function calc_score(wasm, problem, solution) {
        console.log('start calc_score')
        if (score) {
            // 計算済み
            console.log('skip calc_score')
            return;
        }
        try {
            // console.log(problem, solution);
            score = wasm.calc_score(
                problem.room_width,
                problem.room_height,
                problem.stage_width,
                problem.stage_height,
                problem.stage_bottom_left,
                problem.musicians,
                problem.attendees,
                solution.placements,
            );
            console.log('wasm success:', score);
        } catch (err) {
            console.warn(err);
            wasm = null;
            score = 'failed';
        }
    }

    state.subscribe(async (value) => {
        if (value.problem) {
            console.log("start draw...");
            await draw(
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
            setTimeout(async () => {
                await calc_score(wasm, value.problem, value.solution);
            }, 100);
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
            .then(problem => {
                let [zoom, plusx, plusy] = baseDisplayParams(problem);
                score = 0;
                state.update((prev) => {
                    return {
                        ...prev,
                        problem: problem,
                        zoom: zoom,
                        plusx: plusx,
                        plusy: plusy,
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
                let solution = JSON.parse(contents);
                score = 0;
                state.update((prev) => {
                    return {
                        ...prev,
                        solution: solution,
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
        canvas.clearRect(0, 0, 5000, 5000);
    }

    function baseDisplayParams(problem) {
        let width = 1600;
        let height = 1200;
        let minx = problem.stage_bottom_left[0];
        let miny = problem.stage_bottom_left[1];
        let maxx = problem.stage_bottom_left[0] + problem.stage_width;
        let maxy = problem.stage_bottom_left[1] + problem.stage_height;
        for (let m of problem.attendees) {
            minx = Math.min(minx, m.x);
            miny = Math.min(miny, m.y);
            maxx = Math.max(maxx, m.x);
            maxy = Math.max(maxy, m.y);
        }
        let padding = 10;
        minx -= padding;
        miny -= padding;
        maxx += padding;
        maxy += padding;
        console.log([width / (maxx - minx), height / (maxy - miny)]);
        let zoom = Math.min(width / (maxx - minx), height / (maxy - miny));
        let plusx = 800 - zoom * (minx + maxx) / 2;
        let plusy = 600 - zoom * (miny + maxy) / 2;
        return [zoom, plusx, plusy];
    }

    async function draw(problem, solution, colorful, zoom, plusx, plusy, ruler) {
        if (!document) return;
        let obj = document.getElementById('c');
        if (!obj) return;
        let canvas = obj.getContext('2d');
        if (!canvas) return;

        // canvas size
        let width = 1600;
        let height = 1200;

        canvas.clearRect(0, 0, width, height);
        canvas.strokeStyle = '#000';
        canvas.fillStyle = '#fff';
        canvas.fillRect(
            plusx,
            plusy,
            zoom * problem.room_width,
            zoom * problem.room_height
        );
        canvas.strokeRect(
            plusx,
            plusy,
            zoom * problem.room_width,
            zoom * problem.room_height
        );
        // stage
        canvas.fillStyle = '#999';
        canvas.fillRect(
            plusx + zoom * problem.stage_bottom_left[0],
            plusy + zoom * problem.stage_bottom_left[1],
            zoom * problem.stage_width,
            zoom * problem.stage_height
        );
        // 罫線
        if (ruler) {
            let w = 10;
            canvas.strokeStyle = '#9cc';
            for (let x = 0; x <= problem.room_width; x += w) {
                canvas.beginPath();
                canvas.moveTo(plusx + zoom * x, plusy);
                canvas.lineTo(plusx + zoom * x, plusy + zoom * problem.room_height);
                canvas.stroke();
            }
            for (let y = 0; y <= problem.room_width; y += w) {
                canvas.beginPath();
                canvas.moveTo(plusx, plusy + zoom * y);
                canvas.lineTo(plusx + zoom * problem.room_width, plusy + zoom * y);
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
                    plusx + zoom * m.x,
                    plusy + zoom * m.y,
                    zoom * 5,
                    0, 7, false
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
                plusx + zoom * p.center[0],
                plusy + zoom * p.center[1],
                zoom * p.radius,
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
                plusx + zoom * a.x,
                plusy + zoom * a.y,
                1.2,
                0, 7, false
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
        switch (e.key) {
            case '0':
                state.update(prev => {
                    let [zoom, plusx, plusy] = baseDisplayParams(prev.problem);
                    return {
                        ...prev,
                        zoom: zoom,
                        plusx: plusx,
                        plusy: plusy,
                    };
                });
                break;
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
                state.update(prev => {
                    let newzoom = Math.max(0.001, prev.zoom - 0.1);
                    let ratio = newzoom / prev.zoom;
                    let newplusx = ratio * prev.plusx + 800 * (1 - ratio);
                    let newplusy = ratio * prev.plusy + 600 * (1 - ratio);
                    return {
                        ...prev,
                        zoom: newzoom,
                        plusx: newplusx,
                        plusy: newplusy,
                    };
                });
            break;
            case 'e':
                state.update(prev => {
                    let newzoom = prev.zoom + 0.1;
                    let ratio = newzoom / prev.zoom;
                    let newplusx = ratio * prev.plusx + 800 * (1 - ratio);
                    let newplusy = ratio * prev.plusy + 600 * (1 - ratio);
                    return {
                        ...prev,
                        zoom: newzoom,
                        plusx: newplusx,
                        plusy: newplusy,
                    };
                });
            break;
        }
    }

    function clockInit() {
      document.getElementById("countdown").style.color = "red";
      setInterval(() => {
          let targetTime = new Date('2023-07-10T21:00:00');
          let now = new Date();
          let diffMs = targetTime - now;
          let h = Math.floor(diffMs / 3600000); // hours
          let m = Math.floor((diffMs % 3600000) / 60000); // minutes
          let s = Math.round(((diffMs % 3600000) % 60000) / 1000); // seconds
          document.getElementById("countdown").innerText = `⏰ ${h}:${m}:${s}`;
      }, 1000);
    }

    onMount(async () => {
        fetchRecords();
        clockInit();
        wasm = await import('solver');
        await wasm.default();
    });
</script>

<div class="section">
    <h1 id="countdown">⏰</h1>
</div>

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

<div>
    <b>変更履歴</b>
    <dl>
        <dt>07/09 13:21</dt>
        <dd>画面の真ん中を固定にしてちゃんと拡大縮小します</dd>
        <dt>07/09 12:46</dt>
        <dd>表示リセットは 0 (ゼロ) キー</dd>
        <dt>07/09 12:32</dt>
        <dd>ミュージシャンは常に半径 5 で表示される。観客はちっちゃい点のままです</dd>
        <dt>07/09 12:30</dt>
        <dd>ミュージシャンの色付け・罫線の表示切り替えもキー (C/R) でできる</dd>
        <dt>07/09 12:29</dt>
        <dd>拡大縮小・視点の移動ができる。スライダーまたはキー(WASD/QE)</dd>
    </dl>
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
