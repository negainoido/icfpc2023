export function postSolution(problem_id: number, solver: string, solution: object) {
    // return fetch('https://icfpc2023.negainoido.com/api/solutions/submit_json', {
    return fetch('http://localhost:8080/api/solutions/submit_json', {
        method: 'POST',
        mode: 'cors',
        credentials: 'include',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            problem_id,
            solver,
            contents: JSON.stringify(solution),
        }),
    })
        .then((data) => data.json());
}
