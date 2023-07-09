<script>
    export let data;
    import { onMount } from 'svelte';
    import { get, writable } from 'svelte/store';
    import Log from '$lib/Log.svelte';

    let wasm;
    let problem_id = data.problem_id;
    let solution_id = 1;
    let records = [];
    let filteredRecords = [];
    let openjson = false;

    let state = writable({
        problem: null,
        solution: null,
        solution_json: "",
        colorful: true,
        ruler: false,
        zoom: 1.0,
        plusx: 0.0,
        plusy: 0.0,
        log: [],
        target_musician_id: null,
        target_musician_x: 0.0,
        target_musician_y: 0.0,
        target_musician_volume: 1.0,
    });
    let score = null;

    async function calc_score(wasm, problem, solution) {
        console.log('start calc_score');
        if (score) {
            // 計算済み
            console.log('skip calc_score');
            return;
        }
        try {
            console.log(problem, solution);
            const is_full = problem_id > 55;
            if (!solution.volumes) {
                solution.volumes = [];
                for (let i = 0; i < solution.placements.length; i++) {
                    solution.volumes.push(1.0);
                }
            }
            score = wasm.calc_score(
                problem.room_width,
                problem.room_height,
                problem.stage_width,
                problem.stage_height,
                problem.stage_bottom_left,
                problem.musicians,
                problem.attendees,
                problem.pillars,
                solution.placements,
                solution.volumes,
                is_full
            );
            console.log('wasm success:', score);
            state.update((prev) => {
                return {
                    ...prev,
                    log: [`score=${score}`],
                };
            });

        } catch (err) {
            score = `failed`
            state.update((prev) => {
                return {
                    ...prev,
                    log: [`failed: ${err}`],
                };
            });
            console.warn(err);
            // wasm = null;
        }
    }

    state.subscribe(async (value) => {
        if (value.problem) {
            console.log('start draw...');
            await draw(
                value.problem,
                value.solution,
                value.colorful,
                value.zoom,
                value.plusx,
                value.plusy,
                value.ruler,
                value.log
            );
            console.log('...finish draw');
        } else {
            console.log('data not ready; cannot draw');
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
    ];

    function updateAddition() {
        if (!wasm) return; // failed
    }

    /// 良いレコード全部取得
    function fetchRecords() {
        clear();
        fetch('https://icfpc2023.negainoido.com/api/solutions/show', { credentials: 'include' })
            .then((data) => data.json())
            .then((data) => {
                records = data;
                filterRecords();
            });
    }

    /// problem_id ごとの結果を表示する
    function filterRecords() {
        if (!records) return;
        filteredRecords = [];
        clear();
        history.pushState({}, `/${problem_id}`, `/${problem_id}`);
        state.update((prev) => ({
            ...prev,
            problem: null,
            solution: null,
            solution_json: "",
            target_musician_id: null,
            zoom: 1.0,
            plusx: 0.0,
            plusy: 0.0,
        }));
        let used = [];
        for (let r of records) {
            let key = r[3] + r[5];
            if (r[1] !== problem_id) {
                continue;
            } else if (used.includes(key)) {
                continue;
            } else {
                used.push(key);
                filteredRecords.push(r);
                if (filteredRecords.length >= 8) break;
            }
        }
        fetch(`https://icfpc2023.negainoido.com/api/problem?problem_id=${problem_id}`, {
            credentials: 'include',
        })
            .then((data) => data.json())
            .then((problem) => {
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
        fetch(`https://icfpc2023.negainoido.com/api/solutions?id=${solution_id}`, {
            credentials: 'include',
        })
            .then((response) => response.json())
            .then((response) => {
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
                        solution_json: JSON.stringify(solution, null, 2),
                        target_musician_id: null,
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
        let zoom = Math.min(width / (maxx - minx), height / (maxy - miny));
        let plusx = 800 - (zoom * (minx + maxx)) / 2;
        let plusy = 600 - (zoom * (miny + maxy)) / 2;
        return [zoom, plusx, plusy];
    }

    async function draw(problem, solution, colorful, zoom, plusx, plusy, ruler, log) {
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
        canvas.fillRect(plusx, plusy, zoom * problem.room_width, zoom * problem.room_height);
        canvas.strokeRect(plusx, plusy, zoom * problem.room_width, zoom * problem.room_height);
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
                canvas.arc(plusx + zoom * m.x, plusy + zoom * m.y, zoom * 5, 0, 7, false);
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
                0,
                7,
                false
            );
            canvas.fill();
            canvas.stroke();
        }
        // attendees
        canvas.fillStyle = '#a11';
        for (let a of problem.attendees) {
            canvas.beginPath();
            canvas.arc(plusx + zoom * a.x, plusy + zoom * a.y, 1.2, 0, 7, false);
            canvas.fill();
        }
        // log
        if (log) {
            canvas.fillStyle = '#500';
            canvas.font = '24px monospace';
            for (let i = 0; i < log.length; ++i) {
                canvas.fillText(log[i], 12, 29 * (i+1));
            }
        }
    }

    function fullScreen() {
        var canvas = document.getElementById('c');
        if (!canvas) return;
        if (canvas.requestFullscreen) {
            canvas.requestFullscreen();
        } else if (canvas.mozRequestFullScreen) {
            // Firefox
            canvas.mozRequestFullScreen();
        } else if (canvas.webkitRequestFullscreen) {
            // Chrome, Safari and Opera
            canvas.webkitRequestFullscreen();
        } else if (canvas.msRequestFullscreen) {
            // IE/Edge
            canvas.msRequestFullscreen();
        }
    }

    function onKeyDown(e) {
        console.log(e);
        switch (e.key) {
            case '0':
            case 'o':
            case 'O':
                state.update((prev) => {
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
            case 'C':
                state.update((prev) => ({
                    ...prev,
                    colorful: !prev.colorful,
                }));
                break;
            case 'r':
            case 'R':
                state.update((prev) => ({
                    ...prev,
                    ruler: !prev.ruler,
                }));
                break;
            case 'a':
            case 'A':
            case 'h':
                state.update((prev) => ({
                    ...prev,
                    plusx: prev.plusx + 40,
                }));
                break;
            case 'd':
            case 'D':
            case 'l':
                state.update((prev) => ({
                    ...prev,
                    plusx: prev.plusx - 40,
                }));
                break;
            case 'w':
            case 'W':
            case 'k':
                state.update((prev) => ({
                    ...prev,
                    plusy: prev.plusy + 40,
                }));
                break;
            case 's':
            case 'S':
            case 'j':
                state.update((prev) => ({
                    ...prev,
                    plusy: prev.plusy - 40,
                }));
                break;
            case 'q':
            case 'Q':
                state.update((prev) => {
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
            case 'E':
                state.update((prev) => {
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

    function clickCanvas(event) {
        let canvas = document.getElementById('c');
        let rect = canvas.getBoundingClientRect();
        let x = event.clientX - rect.left;
        let y = event.clientY - rect.top;
        x = (x - $state.plusx) / $state.zoom;
        y = (y - $state.plusy) / $state.zoom;
        console.log('click', [x, y]);
        let log = [];
        if ($state.solution && $state.solution.placements) {
            for (let i = 0; i < $state.solution.placements.length; ++i) {
                let m = $state.solution.placements[i];
                let inst = $state.problem.musicians[i];
                const volume = $state.solution.volumes ? $state.solution.volumes[i] : 1.0;
                if (Math.hypot(x - m.x, y - m.y) <= 5.0) {
                    log.push(`musicians[${i}]=${inst}; (x,y)=(${m.x},${m.y})`);
                    state.update((prev) => ({
                        ...prev,
                        target_musician_id: i,
                        target_musician_x: m.x,
                        target_musician_y: m.y,
                        target_musician_volume: volume,
                    }));
                    break;
                }
            }
        }
        if ($state.problem && $state.problem.pillars) {
            for (let i = 0; i < $state.problem.pillars.length; ++i) {
                let p = $state.problem.pillars[i];
                if (Math.hypot(x - p.center[0], y - p.center[1]) <= p.radius) {
                    log.push(
                        `pillars[${i}]; (x,y,radius)=(${p.center[0]},${p.center[1]},${p.radius})`
                    );
                    break;
                }
            }
        }
        if ($state.problem) {
            for (let i = 0; i < $state.problem.attendees.length; ++i) {
                let m = $state.problem.attendees[i];
                if (Math.hypot(x - m.x, y - m.y) <= 8.0) {
                    log.push(`attendees[${i}]; (x,y)=(${m.x},${m.y})`);
                    break;
                }
            }
        }
        state.update((prev) => ({
            ...prev,
            log: log,
        }));
    }

    function clockInit() {
        document.getElementById('countdown').style.color = 'red';
        setInterval(() => {
            let targetTime = new Date('2023-07-10T21:00:00');
            let now = new Date();
            let diffMs = targetTime - now;
            let h = Math.floor(diffMs / 3600000); // hours
            let m = Math.floor((diffMs % 3600000) / 60000); // minutes
            let s = Math.round(((diffMs % 3600000) % 60000) / 1000); // seconds
            document.getElementById('countdown').innerText = `⏰ ${h}:${m}:${s}`;
        }, 1000);
    }

    onMount(async () => {
        fetchRecords();
        clockInit();
        wasm = await import('solver');
        await wasm.default();
    });

    function loadSolutionJSON() {
        score = 0;
        state.update((prev) => ({
            ...prev,
            solution: JSON.parse(prev.solution_json),
        }));
    }

    function updateSolutionJson() {
        score = null;
        console.log('called update');
        state.update((prev) => {
            console.log('update', prev);
            if (prev.solution_json) {
                const new_solution = JSON.parse(prev.solution_json);
                if (prev.target_musician_id !== null) {
                    new_solution.placements[prev.target_musician_id] = {
                        x: prev.target_musician_x,
                        y: prev.target_musician_y,
                    };
                    if (!new_solution.volumes) {
                        new_solution.volumes = new Array(prev.solution.placements.length);
                        new_solution.volumes.fill(1.0);
                    }
                    new_solution.volumes[prev.target_musician_id] = prev.target_musician_volume;
                }
                return {
                    ...prev,
                    solution_json: JSON.stringify(new_solution, null, 2),
                };
            }

            return { ...prev };
        });
    }

    function downloadSolution() {
        const solution = get(state).solution;
        if (!solution) {
            console.warn('no solution');
            return;
        }
        const a = document.createElement('a');
        a.style = 'display: none';
        const json = JSON.stringify(solution);
        const blob = new Blob([json], { type: 'application/json' });
        const url = window.URL.createObjectURL(blob);
        a.download = `solution_${problem_id}.json`;
        a.href = url;
        a.click();
        window.URL.revokeObjectURL(url);
    }
</script>

<section class="section">
    <div class="container">
        <h1 class="title" id="countdown">⏰</h1>
        <a rel="external" data-sveltekit-reload href="https://icfpc2023.negainoido.com/streamlit/?id={problem_id}">
            🔗 streamlit/?id={problem_id}</a>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="control">
            <label for="problem_id">problem_id</label>
            <input
                id="problem_id"
                class="input"
                type="number"
                bind:value={problem_id}
                on:change={filterRecords}
            />
        </div>
    </div>

    <div class="container">
        {#if filteredRecords.length > 0}
            <table class="table">
                <thead>
                    <tr>
                        <th>id</th>
                        <th>problem_id</th>
                        <th>submission_id</th>
                        <th>solver</th>
                        <th>status</th>
                        <th>score</th>
                        <th>ts</th>
                    </tr>
                </thead>
                <tbody>
                    {#each filteredRecords as r}
                        <tr>
                            <td><button on:click={fetchSolution(r[0])}>{r[0]}</button></td>
                            <td>{r[1]}</td>
                            <td>{r[2]}</td>
                            <td>{r[3]}</td>
                            <td>{r[4]}</td>
                            <td>{r[5].toLocaleString()}</td>
                            <td>{r[6]}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        {:else}
            No records!!
        {/if}
    </div>

    <div class="container">
        <div class="box">
            <label>
                <input type="checkbox" bind:checked={openjson} />
                Solution JSON
            </label>
            {#if openjson}
                <div class="container">
                    <textarea
                        style="width: 100%; height: 20vh"
                        class="textarea is-primary"
                        bind:value={$state.solution_json}
                    />
                    <div>
                        musician id: <input type="number" bind:value={$state.target_musician_id} on:change={updateSolutionJson} />
                        x: <input type="number" bind:value={$state.target_musician_x} on:change={updateSolutionJson} />
                        y: <input type="number" bind:value={$state.target_musician_y} on:change={updateSolutionJson}/>
                        volume: <input type="number" bind:value={$state.target_musician_volume} on:change={updateSolutionJson} />
                    </div>
                    <button class="button" on:click={loadSolutionJSON}>show solution</button>
                </div>
            {/if}
        </div>
        <div class="box">
            <label>
                <input type="checkbox" bind:checked={$state.colorful} />
                楽器で色を変える (C)
            </label>
            <label>
                <input type="checkbox" bind:checked={$state.ruler} />
                罫線 (幅=10) (R)
            </label>
            <br />
        <!--
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
        -->
            <nav class="breadcrumb is-centered has-bullet-separator" aria-label="breadcrumbs">
                <ul>
                    <li class="is-active"><a>キー割当</a></li>
                    <li class="is-active"><a>WASD: 移動</a></li>
                    <li class="is-active"><a>E/Q: 拡大/縮小</a></li>
                    <li class="is-active"><a>C: 楽器の色表示</a></li>
                    <li class="is-active"><a>R: 罫線</a></li>
                    <li class="is-active"><a>0: 表示リセット</a></li>
                </ul>
            </nav>
        </div>
        <div>
            <canvas id="c" width="1600" height="1200" on:click={clickCanvas} />
        </div>
        <div>
            <button disabled={$state.solution === null} on:click={downloadSolution}
                >download solution</button
            >
        </div>
        <div class=box>
            <textarea
                style="width: 100%; height: 20vh"
                class="textarea is-primary"
                bind:value={$state.log}
            />
        </div>
    </div>
</section>

<section class="section">
    <div class="container">
        <Log />
    </div>
</section>

<svelte:window on:keydown={onKeyDown} />

<style>
    @import 'https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css';
    canvas {
        border: 1px black solid;
    }
</style>
