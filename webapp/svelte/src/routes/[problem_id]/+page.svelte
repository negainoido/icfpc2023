<script>
    export let data;
    import { onMount } from 'svelte';
    import { get, writable } from 'svelte/store';
    import Log from '$lib/Log.svelte';
    import { page } from '$app/stores';
    import { postSolution } from "$lib/fetchUtil";

    let wasm;
    let problem_id = data.problem_id;
    let records = [];
    let filteredRecords = [];
    let openjson = false;
    let debug = "";
    let editorname = 'hand';
    let submitresult = "";
    let wasm_temporary_disable = false;

    let state = writable({
        problem: null,
        solution: null,
        solution_id: -1,
        solution_json: '',
        colorful: true,
        showvolume: true,
        ruler: false,
        zoom: 1.0,
        plusx: 0.0,
        plusy: 0.0,
        log: [],
        target_musician_id: null,
        target_musician_x: 0.0,
        target_musician_y: 0.0,
        target_musician_volume: 1.0,
        wasm_state: '...', // 'running', 'done', 'failed'
    });
    let score = null;
    let mouse = {
        x: 0,
        y: 0,
        oncanvas: false,
        status: 'mouseup', // 'mousedown'
        target_musician_id: null,
        target_musician_x: 0.0,
        target_musician_y: 0.0,
    };
    let edit_history = [];  // list of (musician_id, (old_x, old_y), (new_x, new_y))
    let edit_redo = [];  // list of (musician_id, (old_x, old_y), (new_x, new_y))

    function canvasLog(msg, append=false) {
        state.update((prev) => {
            return {
                ...prev,
                log: append ? prev.log.concat(msg) : [msg],
            };
        });
    }

    /// 無理やり再描画させる
    function canvasRefresh() {
        state.update((prev) => prev);
    }

    async function calc_score(wasm, problem, solution) {
        console.log('start calc_score');
        if (wasm_temporary_disable || score != null) {
            // 計算済み
            console.log('skip calc_score');
            return;
        }
        try {
            state.update((prev) => ({ ...prev, wasm_state: 'running' }));
            // console.log(problem, solution);
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
            state.update((prev) => ({ ...prev, wasm_state: 'done' }));
        } catch (err) {
            score = `failed`;
            state.update((prev) => ({ ...prev, wasm_state: `failed: ${err}` }));
            console.warn(err);
            // wasm = null;
        }
    }

    state.subscribe(async (state) => {
        if (state.problem) {
            console.log('start draw...');
            await draw(
                state.problem,
                state.solution,
                state.colorful,
                state.zoom,
                state.plusx,
                state.plusy,
                state.ruler,
                state.log
            );
            console.log('...finish draw');
        } else {
            console.log('data not ready; cannot draw');
        }
        if (state.problem && state.solution && wasm) {
            if (mouse.status === 'mouseup') {
                setTimeout(async () => {
                    await calc_score(wasm, state.problem, state.solution);
                }, 100);
            }
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
            solution_json: '',
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
            }
        }
        fetch(`https://icfpc2023.negainoido.com/api/problem?problem_id=${problem_id}`, {
            credentials: 'include',
        })
            .then((data) => data.json())
            .then((problem) => {
                let [zoom, plusx, plusy] = baseDisplayParams(problem);
                score = null;
                state.update((prev) => {
                    return {
                        ...prev,
                        problem: problem,
                        zoom: zoom,
                        plusx: plusx,
                        plusy: plusy,
                    };
                });
                let query_solution_id = $page.url.searchParams.get('solution_id');
                let found = false;
                if (query_solution_id) {
                    let solution_id = parseInt(query_solution_id);
                    let known_score = null;
                    let solvername = null;
                    for (let r of records) {
                        if (r[0] == solution_id && r[1] == problem_id) {
                            found = true;
                            known_score = r[5];
                            solvername = r[3];
                            break;
                        }
                    }
                    fetchSolution(solution_id, known_score, solvername);
                }
                if (!found) {
                    // problem_id の中で一番良い solution をデフォルトで表示させる
                    for (let r of records) {
                        if (r[1] == problem_id) {
                            let solution_id = r[0];
                            let solvername = r[3];
                            fetchSolution(solution_id, r[5], solvername);
                            break;
                        }
                    }
                }
            });
    }

    function fetchSolution(solution_id, known_score, solvername) {
        history.pushState(
            {},
            `/${problem_id}?solution_id=${solution_id}`,
            `/${problem_id}?solution_id=${solution_id}`
        );
        editorname = solvername;
        clear();
        state.update((prev) => {
            return {
                ...prev,
                solution_id: solution_id,
                soluiton: null,
                solution_json: '',
                log: [],
            };
        });
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
                if (known_score) {
                    score = known_score;
                } else {
                    score = null; // to be calced by wasm
                }
                state.update((prev) => {
                    return {
                        ...prev,
                        solution_id: solution_id,
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
        let minx = problem.stage_bottom_left[0] - problem.stage_width * 0.5;
        let miny = problem.stage_bottom_left[1] - problem.stage_height * 0.5;
        let maxx = problem.stage_bottom_left[0] + problem.stage_width * 1.5;
        let maxy = problem.stage_bottom_left[1] + problem.stage_height * 1.5;
        // for (let m of problem.attendees) {
        //     minx = Math.min(minx, m.x);
        //     miny = Math.min(miny, m.y);
        //     maxx = Math.max(maxx, m.x);
        //     maxy = Math.max(maxy, m.y);
        // }
        // let padding = 10;
        // minx -= padding;
        // miny -= padding;
        // maxx += padding;
        // maxy += padding;
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
            for (let y = 0; y <= problem.room_height; y += w) {
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
                if ($state.showvolume && solution.volumes) {
                    let vol = solution.volumes[i];
                    canvas.strokeStyle = vol < 5 ? '#fff' : '#000';
                    canvas.beginPath();
                    canvas.arc(plusx + zoom * m.x, plusy + zoom * m.y, zoom * 5, 0, 7, false);
                    canvas.stroke();
                }
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
            let lines = [
                `problem: ${problem_id}`,
                `solution: ${$state.solution_id}`,
                `score: ${score != null ? score.toLocaleString() : null}`,
                `wasm: ${$state.wasm_state}`,
            ].concat(log);
            for (let i = 0; i < lines.length; ++i) {
                canvas.fillText(lines[i], 12, 29 * (i + 1));
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
        if (!mouse.oncanvas) return;
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
            case 'v':
            case 'V':
                state.update((prev) => ({
                    ...prev,
                    showvolume: !prev.showvolume,
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
                    let newzoom = Math.max(prev.zoom / 2, prev.zoom - 0.1);
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
                    let newzoom = Math.min(prev.zoom + 0.05, prev.zoom * 2);
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
            case 'z':
            case 'Z':
                undoEdit();
                break;
            case 'x':
            case 'X':
                redoEdit();
                break;

        }
    }

    function mousePos(event) {
        let canvas = document.getElementById('c');
        let rect = canvas.getBoundingClientRect();
        let x = event.clientX - rect.left;
        let y = event.clientY - rect.top;
        x = (x - $state.plusx) / $state.zoom;
        y = (y - $state.plusy) / $state.zoom;
        return [x, y];
    }

    function clickCanvas(event) {
        mouse.oncanvas = true;
        let [x, y] = mousePos(event);
        let log = [];
        if ($state.solution && $state.solution.placements) {
            for (let i = 0; i < $state.solution.placements.length; ++i) {
                let m = $state.solution.placements[i];
                let inst = $state.problem.musicians[i];
                const volume = $state.solution.volumes ? $state.solution.volumes[i] : 1.0;
                if (Math.hypot(x - m.x, y - m.y) <= 5.0) {
                    log.push(`musicians[${i}]=${inst}; (x,y)=(${m.x},${m.y})`);
                    if ($state.solution.volumes) {
                        let vol = $state.solution.volumes[i];
                        log.push(`; volume=${vol}`);
                    }
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

    function mousedownCanvas(event) {
        mouse.oncanvas = true;
        if (!event.shiftKey) return;
        let [x, y] = mousePos(event);
        mouse = {
            x: x,
            y: y,
            status: 'mousedown',
            target_musician_id: null,
        };
        if ($state.solution && $state.solution.placements) {
            for (let i = 0; i < $state.solution.placements.length; ++i) {
                let m = $state.solution.placements[i];
                let inst = $state.problem.musicians[i];
                if (Math.hypot(x - m.x, y - m.y) <= 5.0) {
                    mouse.target_musician_id = i;
                    mouse.target_musician_x = m.x;
                    mouse.target_musician_y = m.y;
                }
            }
        }
    }
    function mouseupCanvas(event) {
        mouse.oncanvas = true;
        // if (!event.shiftKey) return;
        let [x, y] = mousePos(event);
        if (mouse.target_musician_id != null) {
            let mid = mouse.target_musician_id;
            score = null;
            editorname = editorname.replace('(edit)', '') + '(edit)';
            edit_history.push([mid, [mouse.target_musician_x, mouse.target_musician_y], [x, y]]);
            canvasLog(`edit musicians[${mid}]: ${[mouse.target_musician_x, mouse.target_musician_y]} -> ${[x, y]}`);
            state.update((state) => {
                state.solution.placements[mid].x = x;
                state.solution.placements[mid].y = y;
                state.solution_json = JSON.stringify(state.solution, null, 2);
                return state;
            });
        }
        mouse = {
            x: x,
            y: y,
            status: 'mouseup',
            target_musician_id: null,
        };
    }
    function mousemoveCanvas(event) {
        mouse.oncanvas = true;
        if (!event.shiftKey) return;
        let [x, y] = mousePos(event);
        if (mouse.target_musician_id != null) {
            let mid = mouse.target_musician_id;
            score = null;
            state.update((state) => {
                state.solution.placements[mid].x = x;
                state.solution.placements[mid].y = y;
                return state;
            });
        }
    }
    function mouseoutCanvas(event) {
        mouse.oncanvas = false;
        console.log(mouse);
    }

    function undoEdit() {
        if (edit_history.length == 0) {
            canvasLog('Cannot undo; this is oldest');
            return;
        }
        let [mid, [old_x, old_y], [new_x, new_y]] = edit_history.pop();
        edit_redo.push([mid, [old_x, old_y], [new_x, new_y]]);
        canvasLog(`Undo musicians[${mid}]`)
        state.update((state) => {
            state.solution.placements[mid].x = old_x;
            state.solution.placements[mid].y = old_y;
            return state;
        });
    }

    function redoEdit() {
        if (edit_redo.length == 0) {
            canvasLog('Cannot redo; this is newest');
            return;
        }
        let [mid, [old_x, old_y], [new_x, new_y]] = edit_redo.pop();
        edit_history.push([mid, [old_x, old_y], [new_x, new_y]]);
        canvasLog(`Redo musicians[${mid}]`)
        state.update((state) => {
            state.solution.placements[mid].x = new_x;
            state.solution.placements[mid].y = new_y;
            return state;
        });
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
        score = null;
        editorname = editorname.replace('(edit)', '') + '(edit)';
        state.update((prev) => ({
            ...prev,
            solution_id: -1,
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

    function copyToClipboard() {
        var copyText = document.getElementById("solutionTextarea");
        copyText.select();
        document.execCommand("copy");
    }

    function submitSolution() {
        console.log('submit');
        const solution = get(state).solution;
        postSolution(problem_id, editorname, solution)
            .then((res) => {
                console.log(res);
                debug += JSON.stringify(res);
                submitresult = JSON.stringify(res);
                state.update((prev) => ({
                    ...prev,
                    solution_id: res.solution_id,
                }));
            })
            .catch((err) => {
                console.warn(err);
                debug += err.toString();
                submitresult = err.toString();
            });
    }
</script>

<section class="section">
    <div class="container">
        <h1 class="title" id="countdown">⏰</h1>
        <a
            rel="external"
            data-sveltekit-reload
            href="https://icfpc2023.negainoido.com/streamlit/?id={problem_id}"
        >
            🔗 streamlit/?id={problem_id}</a
        >
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
            <div class="scrollable-table">
                <table class="table">
                    <thead>
                        <tr>
                            <th>id</th>
                            <th>submission_id</th>
                            <th>solver</th>
                            <th>score</th>
                            <th>ts</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each filteredRecords as r}
                            <tr>
                                <td><button on:click={fetchSolution(r[0], r[5], r[3])}>{r[0]}</button></td
                                >
                                <td>{r[2]}</td>
                                <td>{r[3]}</td>
                                <td>{r[5]?.toLocaleString()}</td>
                                <td>{r[6]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
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
                    <div style="position: relative; ">
                        <textarea
                            style="width: 100%; height: 20vh"
                            class="textarea is-primary"
                            id="solutionTextarea"
                            bind:value={$state.solution_json}
                        />
                        <button
                            style="position: absolute; top: 0; right: 0;"
                            on:click={copyToClipboard}
                        >Copy</button>
                    </div>
                    <div class="field">
                        musician id: <input
                            type="number"
                            bind:value={$state.target_musician_id}
                            on:change={updateSolutionJson}
                        />
                        x:
                        <input
                            type="number"
                            bind:value={$state.target_musician_x}
                            on:change={updateSolutionJson}
                        />
                        y:
                        <input
                            type="number"
                            bind:value={$state.target_musician_y}
                            on:change={updateSolutionJson}
                        />
                        volume:
                        <input
                            type="number"
                            bind:value={$state.target_musician_volume}
                            on:change={updateSolutionJson}
                        />
                    </div>
                    <button class="button is-primary" on:click={loadSolutionJSON}>show/update solution</button>
                </div>
                <div class="field has-addons">
                    <div class="control">
                        <input class="input" type="text" bind:value={editorname} />
                    </div>
                    <div class="control">
                        <a class="button is-info" on:click={submitSolution}>
                            submit solution
                        </a>
                    </div>
                </div>
                {#if submitresult}
                <div class="field">
                    <div class="control">
                        <label class="label">{submitresult}</label>
                    </div>
                </div>
                {/if}
            {/if}
        </div>
        <div class="box">
            <label>
                <input type="checkbox" bind:checked={$state.colorful} />
                楽器で色を変える (C)
            </label>
            <label>
                <input type="checkbox" bind:checked={$state.showvolume} />
                ボリュームで縁の色を変える (V)
            </label>
            <label>
                <input type="checkbox" bind:checked={$state.ruler} />
                罫線 (幅=10) (R)
            </label>
            <label>
                <input type="checkbox" bind:checked={wasm_temporary_disable} />
                一時的にwasmをオフ
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
                    <li class="is-active"><a>V: ボリューム表示</a></li>
                    <li class="is-active"><a>0: 表示リセット</a></li>
                    <li class="is-active"><a>Shift+ドラッグ: 演奏者移動</a></li>
                    <li class="is-active"><a>Z: Undo</a></li>
                    <li class="is-active"><a>X: Redo</a></li>
                </ul>
            </nav>
        </div>
        <div>
            <canvas
                id="c"
                width="1600"
                height="1200"
                on:click={clickCanvas}
                on:mousedown={mousedownCanvas}
                on:mouseup={mouseupCanvas}
                on:mousemove={mousemoveCanvas}
                on:mouseout={mouseoutCanvas}
            />
        </div>
        <div>
            <button disabled={$state.solution === null} on:click={downloadSolution}
                >download solution</button
            >
        </div>
        <div class="box">
            <textarea
                style="width: 100%; height: 20vh"
                class="textarea is-primary"
                bind:value={$state.log}
            />
        </div>
        {#if debug}
        <div class="box">
            <textarea
                style="width: 100%; height: 20vh"
                class="textarea is-primary"
                bind:value={debug}
                disabled
            />
        </div>
        {/if}
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
    .scrollable-table {
        height: 240px;
        overflow-y: auto;
    }
</style>
