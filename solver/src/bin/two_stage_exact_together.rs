use clap::Parser;
use geo::{EuclideanDistance, Point};
use ordered_float::OrderedFloat;
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;
use std::collections::HashMap;

use solver::problem::*;
use solver::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,

    #[arg(short, long, default_value_t = 100.0)]
    timeout: f64,
}

fn get_time() -> f64 {
    static mut STIME: f64 = -1.0;
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let ms = t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9;
    unsafe {
        if STIME < 0.0 {
            STIME = ms;
        }
        ms - STIME
    }
}


fn generate_first_level_candidates(input: &Input) -> Vec<Point> {
    let x_count = (input.stage_width / 10.0).floor() as usize - 1;
    let y_count = (input.stage_height / 10.0).floor() as usize - 1;
    dbg!(x_count, y_count);

    // Generate first level candidate
    let x_gap = if x_count > 1 {
        (input.stage_width - 20.0) / (x_count - 1) as f64
    } else {
        0.0
    };
    let y_gap = if y_count > 1 {
        (input.stage_height - 20.0) / (y_count - 1) as f64
    } else {
        0.0
    };

    // Prefer candidates closer to the stage borders
    let mut layered_candidates = vec![];
    for i in 0..x_count {
        for j in 0..y_count {
            let offset = Point::new(10.0 + i as f64 * x_gap, 10.0 + j as f64 * y_gap);
            let pos = input.stage_bottom_left + offset;
            let x_level = i.min(x_count - 1 - i);
            let y_level = j.min(y_count - 1 - j);
            let level = x_level.min(y_level);
            while level + 1 > layered_candidates.len() {
                layered_candidates.push(vec![]);
            }
            layered_candidates[level].push(pos);
        }
    }

    let mut candidates = vec![];
    for level in 0..layered_candidates.len() {
        candidates.extend(layered_candidates[level].clone());
        if candidates.len() >= input.musicians.len() {
            break;
        }
    }
    candidates
}

fn create_matching_matrix(input: &Input, candidates: &&Vec<Point>) -> Matrix<OrderedFloat<f64>> {
    let mut matrix = Matrix::new(input.musicians.len(), candidates.len(), OrderedFloat(0.0));
    let mut reachable_candidates = vec![];
    for attendee_id in 0..input.attendees.len() {
        let attendee_pos = input.attendees[attendee_id].pos();
        let non_blocked_candidate_ids = get_non_blocked_placement_ids(attendee_pos, &candidates);
        let candidate_ids = filter_placements_blocked_by_pillars(
            attendee_pos,
            &candidates,
            &input.pillars,
            &non_blocked_candidate_ids,
        );
        reachable_candidates.push(candidate_ids);
    }

    for musician_id in 0..input.musicians.len() {
        for attendee_id in 0..input.attendees.len() {
            for &reachable_candidate_id in &reachable_candidates[attendee_id] {
                // musician_id を placement_id に対応させたときの attendee_id に対応するスコアを計算
                let score = input.raw_impact(
                    attendee_id,
                    musician_id,
                    &candidates[reachable_candidate_id],
                );
                matrix[(musician_id, reachable_candidate_id)] += score;
            }
        }
    }
    matrix
}

fn exact_match_candidates(input: &Input, candidates: &Vec<Point>) -> (f64, Vec<Point>) {
    let matrix = create_matching_matrix(input, &candidates);
    let (score, assignments) = kuhn_munkres(&matrix);
    let mut filtered_candidates = vec![];
    for assignment in assignments {
        filtered_candidates.push(candidates[assignment].clone());
    }
    (score.0, filtered_candidates)
}

fn two_stage_optimization(input: &Input) -> Vec<Point> {
    let first_level_candidates = generate_first_level_candidates(input);
    // 最初はmusicianよりも多い候補地を用いて最適化を行い、その結果に基づき二段階目の最適化に使用する候補地を列挙
    let (first_level_score, second_level_candidates) =
        exact_match_candidates(input, &first_level_candidates);
    dbg!(first_level_score);
    let (second_level_score, best_placements) =
        exact_match_candidates(input, &second_level_candidates);
    dbg!(second_level_score);
    best_placements
}

fn hill_climbing(input: &Input, placements: &Vec<Point>, timeout: f64) -> Vec<Point> {
    // Compute the original score
    let mut solution: Solution = Default::default();
    solution.placements = placements.clone();
    let initial_score = solution.score(&input, true).unwrap();
    let mut final_score = initial_score;
    dbg!(initial_score);

    // Group musicians based on their instruments
    let mut same_instrument_group: HashMap<usize, Vec<usize>> = HashMap::new();
    for musician_id in 0..input.musicians.len() {
        let instrument = input.musicians[musician_id];
        if same_instrument_group.contains_key(&instrument) {
            same_instrument_group
                .get_mut(&instrument)
                .unwrap()
                .push(musician_id);
        } else {
            same_instrument_group.insert(instrument, vec![musician_id; 1]);
        }
    }

    let matrix = create_matching_matrix(input, &placements);
    let mut has_update = true;
    let mut assignments: Vec<usize> = (0..placements.len()).collect();

    let mut swap = |musician_i: usize, musician_j: usize| -> f64 {
        if musician_i == musician_j {
            return 0.0;
        }

        // musician_i は placements[ai] にいる
        let mut score_delta = 0.0;
        score_delta -= matrix[(musician_i, assignments[musician_i])].0
            + matrix[(musician_j, assignments[musician_j])].0;
        assignments.swap(musician_i, musician_j);
        score_delta += matrix[(musician_i, assignments[musician_i])].0
            + matrix[(musician_j, assignments[musician_j])].0;

        if input.musicians[musician_i] != input.musicians[musician_j] {
            // Compute musician i's impact
            let instrument_i = input.musicians[musician_i];
            for &musician_k in same_instrument_group.get(&instrument_i).unwrap() {
                if musician_k != musician_i && musician_k != musician_j {
                    let old_dist_ik = placements[assignments[musician_j]]
                        .euclidean_distance(&placements[assignments[musician_k]]);
                    let new_dist_ik = placements[assignments[musician_i]]
                        .euclidean_distance(&placements[assignments[musician_k]]);

                    let impact_k = matrix[(musician_k, assignments[musician_k])].0;
                    let old_impact_i = matrix[(musician_i, assignments[musician_j])].0;
                    let new_impact_i = matrix[(musician_i, assignments[musician_i])].0;
                    score_delta -= (1.0 / old_dist_ik) * (impact_k + old_impact_i);
                    score_delta += (1.0 / new_dist_ik) * (impact_k + new_impact_i);
                }
            }

            // Compute musician j's impact
            let instrument_j = input.musicians[musician_j];
            for &musician_k in same_instrument_group.get(&instrument_j).unwrap() {
                if musician_k != musician_i && musician_k != musician_j {
                    let old_dist_jk = placements[assignments[musician_i]]
                        .euclidean_distance(&placements[assignments[musician_k]]);
                    let new_dist_jk = placements[assignments[musician_j]]
                        .euclidean_distance(&placements[assignments[musician_k]]);

                    // dbg!(old_dist_jk, new_dist_jk);
                    let impact_k = matrix[(musician_k, assignments[musician_k])].0;
                    let old_impact_j = matrix[(musician_j, assignments[musician_i])].0;
                    let new_impact_j = matrix[(musician_j, assignments[musician_j])].0;
                    score_delta -= (1.0 / old_dist_jk) * (impact_k + old_impact_j);
                    score_delta += (1.0 / new_dist_jk) * (impact_k + new_impact_j);
                }
            }
        }
        score_delta
    };

    while has_update && get_time() < timeout {
        has_update = false;
        for musician_i in 0..placements.len() {
            for musician_j in 0..placements.len() {
                let delta = swap(musician_i, musician_j);
                if delta > 0.1 {
                    // There is some improvement
                    // eprintln!("Detected improvement: {} -> {}", final_score, final_score + delta);
                    final_score += delta;
                    has_update = true;
                } else {
                    swap(musician_i, musician_j);
                }
            }

            if has_update {
                break;
            }
        }
    }
    eprintln!("Finished hill-climbing: score = {}, elapsed-time = {}", final_score, get_time());
    let mut new_placements = vec![];
    for i in 0..assignments.len() {
        new_placements.push(placements[assignments[i]]);
    }
    new_placements
}

fn solve(input: &Input, timeout: f64) -> Vec<Point> {
    let base_placements = two_stage_optimization(input);
    let best_placements = if input.pillars.len() > 0 {
        // with extension
        hill_climbing(input, &base_placements, timeout)
    } else {
        base_placements
    };
    best_placements
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input.clone()).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let x_count = (input.stage_width / 10.0).floor() as usize - 1;
    let y_count = (input.stage_height / 10.0).floor() as usize - 1;

    // initialize timer
    get_time();

    let best_placements = if x_count * y_count >= input.musicians.len() {
        // Use new strategy!
        solve(&input, args.timeout)
    } else {
        // Give up
        let mut generator = PlacementGenerator::new(input.clone(), args.rand_seed);
        generator.generate()
    };
    let mut solution: Solution = Default::default();
    solution.placements = best_placements.clone();
    input.is_valid_placements(&best_placements).unwrap();
    eprintln!("Solver score: {}", solution.score(&input, true).unwrap());
    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap();
}
