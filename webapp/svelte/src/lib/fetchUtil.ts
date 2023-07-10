export function postSolution(id: number, solver: string, solution: object) {
    return fetch('https://icfpc2023.negainoido.com/api/submit_json', {
        method: 'POST',
        mode: 'cors',
        credentials: 'include',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            problem_id: id,
            solver,
            solution,
        }),
    })
        .then((data) => data.json());
}
