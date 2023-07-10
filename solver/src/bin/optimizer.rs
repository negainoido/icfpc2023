use clap::Parser;
use geo::Point;
use rand::prelude::SliceRandom;
use rand::seq::IteratorRandom;
use rand::Rng;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

use std::collections::{HashMap, HashSet};

use std::time::Duration;

use solver::problem::*;
use solver::solver_util::volume_optimize;

const PER_COUNT: u128 = 4;

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
    let mut neighbors = find_neighbor(solution, *tar2);
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
fn find_neighbor(solution: &Solution, target: usize) -> Vec<Point> {
    let point = solution.placements[target];
    let mut result = vec![];
    result.push(Point::new(point.x() - 10.0, point.y()));
    result.push(Point::new(point.x() + 10.0, point.y()));
    result.push(Point::new(point.x(), point.y() - 10.0));
    result.push(Point::new(point.x(), point.y() + 10.0));
    let delta = 10.0 * f64::sqrt(3.0 / 2.0) + 1e06;
    result.push(Point::new(point.x() + delta, point.y() + 5.0));
    result.push(Point::new(point.x() - delta, point.y() + 5.0));
    result.push(Point::new(point.x() + delta, point.y() - 5.0));
    result.push(Point::new(point.x() - delta, point.y() - 5.0));
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
    let mut best_solution = solution.clone();
    let mut best_score = input.score_fast(&best_solution).unwrap();
    let mut rnd = Pcg64Mcg::new(seed);
    let now = std::time::Instant::now();

    while now.elapsed() < time {
        let way = rnd.gen_range(0..2);
        if way == 0 {
            println!("swap");
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
                println!("best score: {}", best_score);
            }
        } else {
            println!("random move");
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
                println!("best score: {}", best_score);
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
            let seed = args.rand_seed + i * 2;
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

    println!("final best score: {}", best_score);

    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}
