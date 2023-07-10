use clap::Parser;
use geo::{EuclideanDistance, Point};
use rand::prelude::SliceRandom;
use rand::seq::IteratorRandom;
use rand::Rng;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

use std::collections::{HashMap, HashSet, VecDeque};

use std::time::Duration;

use solver::problem::*;
use solver::solver_util::volume_optimize;

const PER_COUNT: u128 = 2;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    solution: String,

    #[arg(long, default_value_t = 30)]
    time_sec: u64,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn make_honeycomb_line(
    input: &Input,
    solution: &Solution,
    rnd: &mut Pcg64Mcg,
    musician_map: &Vec<Vec<usize>>,
) -> Solution {
    let target = (0..input.musicians.len()).choose(rnd).unwrap();
    let inst = input.musicians[target];
    let candidates = &musician_map[inst];
    let mut graph = vec![Vec::new(); input.musicians.len()];
    for i in 0..candidates.len() {
        for j in i + 1..candidates.len() {
            let left = candidates[i];
            let right = candidates[j];
            if solution.placements[left].euclidean_distance(&solution.placements[right]) < 15.0 {
                graph[left].push(right);
                graph[right].push(left);
            }
        }
    }
    if graph[target].is_empty() {
        return solution.clone();
    }
    let mut cluster = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(target);
    while let Some(cur) = queue.pop_front() {
        if cluster.contains(&cur) {
            continue;
        }
        cluster.insert(cur);
        if cluster.len() >= 7 {
            break;
        }
        for &next in &graph[cur] {
            queue.push_back(next);
        }
    }
    let cluster = cluster.into_iter().collect::<Vec<_>>();
    if cluster.len() < 2 {
        return solution.clone();
    }

    let min_x = cluster
        .iter()
        .map(|&i| solution.placements[i].x())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_x = cluster
        .iter()
        .map(|&i| solution.placements[i].x())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let min_y = cluster
        .iter()
        .map(|&i| solution.placements[i].y())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_y = cluster
        .iter()
        .map(|&i| solution.placements[i].y())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let starts = [
        Point::new(min_x, min_y),
        Point::new(min_x, max_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
    ];
    let mut dir = [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]];
    let dist = 10.0 + rnd.gen_range(0.0..0.5);
    let delta_x = dist / 2.0;
    let delta_y = dist * f64::sqrt(3.0 / 2.0) + 1e-06;
    let deltas = vec![[delta_x, delta_y], [-delta_x, delta_y]];
    dir.shuffle(rnd);
    for &start in starts.iter() {
        for &[dx, dy] in dir.iter() {
            let mut points = vec![];
            let mut x = start.x();
            let mut y = start.y();
            points.push(Point::new(x, y));
            for i in 0..cluster.len() - 1 {
                x += deltas[i % 2][0] * dx;
                y += deltas[i % 2][1] * dx;
                x += deltas[i % 2][1] * dy;
                y += deltas[i % 2][0] * dy;
                points.push(Point::new(x, y));
            }

            let mut new_solution = solution.clone();
            for (&i, &p) in cluster.iter().zip(points.iter()) {
                new_solution.placements[i] = p;
            }
            if input.is_valid_placements(&new_solution.placements).is_ok() {
                println!("try honeycomb line: {:?}", points);
                return new_solution;
            }
        }
    }
    //println!("failed to find honeycomb line");

    solution.clone()
}

fn random_swap(
    solution: &Solution,
    musician_map: &Vec<Vec<usize>>,
    rnd: &mut Pcg64Mcg,
) -> (Solution, usize, usize) {
    let mut new_solution = solution.clone();
    let target_insts = (0..musician_map.len()).choose_multiple(rnd, 2);
    let target_insts = (target_insts[0], target_insts[1]);
    let left = *musician_map[target_insts.0].choose(rnd).unwrap();
    let right = *musician_map[target_insts.1].choose(rnd).unwrap();
    new_solution.placements.swap(left, right);

    (new_solution, left, right)
}
fn random_move2(input: &Input, solution: &Solution, rnd: &mut Pcg64Mcg) -> (Solution, usize) {
    let target = (0..solution.placements.len()).choose(rnd).unwrap();
    let delta = rnd.gen_range(0.0..1.0);
    let theta = rnd.gen_range(0.0..2.0 * std::f64::consts::PI);
    let mut deltas = vec![
        [delta * f64::cos(theta), delta * f64::sin(theta)],
        [-delta * f64::cos(theta), delta * f64::sin(theta)],
        [delta * f64::cos(theta), -delta * f64::sin(theta)],
        [-delta * f64::cos(theta), -delta * f64::sin(theta)],
    ];

    deltas.shuffle(rnd);
    for &d in deltas.iter() {
        let mut tmp_solution = solution.clone();
        tmp_solution.placements[target] = Point::new(
            solution.placements[target].x() + d[0],
            solution.placements[target].y() + d[1],
        );
        if input.is_valid_placements(&tmp_solution.placements).is_ok() {
            return (tmp_solution, target);
        }
    }
    println!("no valid delta move");

    (solution.clone(), target)
}

// volumeが低いmusicianを遠くに移動させてみる
fn random_move3(input: &Input, solution: &Solution, rnd: &mut Pcg64Mcg) -> (Solution, usize) {
    if solution.volumes.is_none() {
        return random_move2(input, solution, rnd);
    }
    let v0_candidates = solution
        .volumes
        .as_ref()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, &v)| v < 1.0)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if v0_candidates.is_empty() {
        return random_move2(input, solution, rnd);
    }

    let target = *v0_candidates.choose(rnd).unwrap();
    let delta = rnd.gen_range(10.0..100.0);
    let theta = rnd.gen_range(0.0..2.0 * std::f64::consts::PI);
    let mut deltas = vec![
        [delta * f64::cos(theta), delta * f64::sin(theta)],
        [-delta * f64::cos(theta), delta * f64::sin(theta)],
        [delta * f64::cos(theta), -delta * f64::sin(theta)],
        [-delta * f64::cos(theta), -delta * f64::sin(theta)],
    ];
    deltas.shuffle(rnd);
    for &d in deltas.iter() {
        let mut tmp_solution = solution.clone();
        tmp_solution.placements[target] = Point::new(
            solution.placements[target].x() + d[0],
            solution.placements[target].y() + d[1],
        );
        if input.is_valid_placements(&tmp_solution.placements).is_ok() {
            println!("big delta move");
            return (tmp_solution, target);
        }
    }
    //println!("no valid delta move");

    return (solution.clone(), target);
}

fn random_move(
    input: &Input,
    solution: &Solution,
    musician_map: &[Vec<usize>],
    rnd: &mut Pcg64Mcg,
) -> (Solution, usize) {
    let target = (0..solution.placements.len()).choose(rnd).unwrap();
    let inst = input.musicians[target];
    if musician_map[inst].len() == 1 {
        return (solution.clone(), target);
    }
    let mut candidates: HashSet<usize> = HashSet::from_iter(musician_map[inst].iter().cloned());
    candidates.remove(&target);
    let tar2 = candidates.iter().choose(rnd).unwrap();
    let delta = rnd.gen_range(0.0..0.5);
    let mut neighbors = find_neighbor(solution, *tar2, delta);
    neighbors.shuffle(rnd);
    for &n in neighbors.iter() {
        let mut tmp_solution = solution.clone();
        tmp_solution.placements[target] = n;
        if input.is_valid_placements(&tmp_solution.placements).is_ok() {
            return (tmp_solution, target);
        }
    }
    println!("no valid neighbor");

    (solution.clone(), target)
}
fn find_neighbor(solution: &Solution, target: usize, delta: f64) -> Vec<Point> {
    let point = solution.placements[target];
    let mut result = vec![];
    let dist = 10.0 + delta;
    result.push(Point::new(point.x() - dist, point.y()));
    result.push(Point::new(point.x() + dist, point.y()));
    result.push(Point::new(point.x(), point.y() - dist));
    result.push(Point::new(point.x(), point.y() + dist));
    let dx = dist / 2.0;
    let dy = dist * f64::sqrt(3.0 / 2.0) + 1e-06;
    result.push(Point::new(point.x() + dx, point.y() + dy));
    result.push(Point::new(point.x() - dx, point.y() + dy));
    result.push(Point::new(point.x() + dx, point.y() - dy));
    result.push(Point::new(point.x() - dx, point.y() - dy));
    result.push(Point::new(point.x() + dy, point.y() + dx));
    result.push(Point::new(point.x() - dy, point.y() + dx));
    result.push(Point::new(point.x() + dy, point.y() - dx));
    result.push(Point::new(point.x() - dy, point.y() - dx));
    result
}

fn switch_volume(solution: &Solution, target: usize) -> Solution {
    let mut new_solution = solution.clone();
    let mut volumes = solution
        .volumes
        .clone()
        .unwrap_or(vec![1.0; solution.placements.len()]);
    if volumes[target] < 1.1 {
        volumes[target] = 10.0;
    } else {
        volumes[target] = 0.0;
    }

    new_solution.volumes = Some(volumes);
    new_solution
}

fn find_best(
    input: &Input,
    solution: &Solution,
    musician_map: &Vec<Vec<usize>>,
    seed: u128,
    time: Duration,
) -> (f64, Solution) {
    println!("my seed is {}", seed);
    let mut best_solution = solution.clone();
    let mut best_score = input.score_fast(&best_solution).unwrap();
    let mut rnd = Pcg64Mcg::new(seed);
    let now = std::time::Instant::now();

    while now.elapsed() < time {
        let way = rnd.gen_range(0..5);
        if way == 0 {
            //println!("swap");
            let (new_solution, l, r) = random_swap(&best_solution, musician_map, &mut rnd);
            let mut flag = false;
            match input.score_fast(&new_solution) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = new_solution.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            let solution2 = switch_volume(&new_solution, l);
            let mut flag2 = false;
            match input.score_fast(&solution2) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = solution2.clone();
                        best_score = new_score;
                        flag = true;
                        flag2 = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            let solution3 = if flag2 {
                switch_volume(&solution2, r)
            } else {
                switch_volume(&new_solution, r)
            };
            match input.score_fast(&solution3) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = solution3.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            if flag {
                println!("swap best score: {}", best_score);
            }
        } else if way == 1 {
            // println!("random move");
            let (new_solution, tar) = random_move(input, &best_solution, musician_map, &mut rnd);
            let mut flag = false;
            match input.score_fast(&new_solution) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = new_solution.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            let solution2 = switch_volume(&new_solution, tar);
            match input.score_fast(&solution2) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = solution2.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            if flag {
                println!("move best score: {}", best_score);
            }
        } else if way == 2 {
            let (new_solution, tar) = random_move2(&input, &best_solution, &mut rnd);
            let mut flag = false;
            match input.score_fast(&new_solution) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = new_solution.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            let solution2 = switch_volume(&new_solution, tar);
            match input.score_fast(&solution2) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = solution2.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            if flag {
                println!("delta move best score: {}", best_score);
            }
        } else if way == 3 {
            let (new_solution, tar) = random_move3(&input, &best_solution, &mut rnd);
            let mut flag = false;
            match input.score_fast(&new_solution) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = new_solution.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            let solution2 = switch_volume(&new_solution, tar);
            match input.score_fast(&solution2) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = solution2.clone();
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            if flag {
                println!("big move best score: {}", best_score);
            }
        } else {
            let mut flag = false;
            //println!("hanicomob");
            let new_solution = make_honeycomb_line(&input, &best_solution, &mut rnd, musician_map);
            let new_solution = volume_optimize(&input, &new_solution);
            match input.score_fast(&new_solution) {
                Ok(new_score) => {
                    if new_score > best_score {
                        best_solution = new_solution;
                        best_score = new_score;
                        flag = true;
                    }
                }
                Err(_) => {
                    println!("invalid solution");
                }
            }
            if flag {
                println!("hanicomb best score: {}", best_score);
            }
        }
    }

    (best_score, best_solution)
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let solution_str = std::fs::read_to_string(&args.solution).unwrap();
    let original_solution: Solution = serde_json::from_str(&solution_str).unwrap();
    let original_score = input.score_fast(&original_solution).unwrap();

    let mut instruments = HashMap::new();
    for &m in &input.musicians {
        let count = instruments.get(&m).unwrap_or(&0);
        instruments.insert(m, count + 1);
    }
    let mut musician_map = vec![Vec::new(); instruments.keys().len()];
    for (i, &m) in input.musicians.iter().enumerate() {
        musician_map[m].push(i);
    }
    let musician_map = musician_map;
    if musician_map.len() < 2 {
        panic!("musicians are too few");
    }
    let solution = volume_optimize(&input, &original_solution);
    let (best_score, best_solution) = (0..PER_COUNT)
        .into_par_iter()
        .map(|i| {
            let seed = args.rand_seed + i * 4;
            find_best(
                &input,
                &solution,
                &musician_map,
                seed,
                Duration::from_secs(args.time_sec),
            )
        })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();
    if best_score == original_score {
        println!("original solution is best");
    } else {
        println!("final best score: {}", best_score);

        std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
    }
}
