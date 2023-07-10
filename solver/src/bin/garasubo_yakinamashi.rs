use clap::Parser;
use rand::Rng;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

use std::collections::HashMap;

use solver::garasubo_util;
use std::time::Duration;

use solver::problem::*;
use solver::solver_util::volume_optimize_fast;

const PER_COUNT: u128 = 8;

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

fn find_best(
    input: &Input,
    solution: &Solution,
    musician_map: &Vec<Vec<usize>>,
    seed: u128,
    time: Duration,
) -> (f64, Solution) {
    println!("my seed is {}", seed);
    let mut rnd = Pcg64Mcg::new(seed);
    let now = std::time::Instant::now();

    //println!("hanicomob");
    let new_solution =
        garasubo_util::make_honeycomb_line(&input, &solution, &mut rnd, musician_map);
    let new_solution = volume_optimize_fast(&input, &new_solution);
    let new_score = match input.score_fast(&new_solution) {
        Ok(new_score) => new_score,
        Err(_) => {
            println!("invalid solution");
            return (0.0, solution.clone());
        }
    };

    let mut best_solution = new_solution.clone();
    let mut best_score = new_score;

    let mut count = 0;
    while now.elapsed() < time {
        count += 1;
        let way = rnd.gen_range(0..4);
        if way == 0 {
            //println!("swap");
            let (new_solution, l, r) =
                garasubo_util::random_swap(&best_solution, musician_map, &mut rnd);
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
            let solution2 = garasubo_util::switch_volume(&new_solution, l);
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
                garasubo_util::switch_volume(&solution2, r)
            } else {
                garasubo_util::switch_volume(&new_solution, r)
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
            let (new_solution, tar) =
                garasubo_util::random_move(input, &best_solution, musician_map, &mut rnd);
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
            let solution2 = garasubo_util::switch_volume(&new_solution, tar);
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
            let (new_solution, tar) = garasubo_util::random_move2(&input, &best_solution, &mut rnd);
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
            let solution2 = garasubo_util::switch_volume(&new_solution, tar);
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
        } else {
            let (new_solution, tar) = garasubo_util::random_move3(&input, &best_solution, &mut rnd);
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
            let solution2 = garasubo_util::switch_volume(&new_solution, tar);
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
        }
    }

    println!("tried {} times", count);
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
    let solution = volume_optimize_fast(&input, &original_solution);
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
    if best_score <= original_score {
        println!("original solution is best");
    } else {
        println!("final best score: {}", best_score);

        std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
    }
}
