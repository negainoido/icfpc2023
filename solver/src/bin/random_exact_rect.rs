use clap::Parser;
use geo::Point;
use ordered_float::OrderedFloat;
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;

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

fn exact_match_candidates(input: &Input, candidates: &Vec<Point>) -> (f64, Vec<Point>, Vec<f64>) {
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

    for musician_id in 0..input.musicians.len() {
        for candidate_id in 0..candidates.len() {
            let e = matrix[(musician_id, candidate_id)].0;
            if e < 0.0 {
                matrix[(musician_id, candidate_id)] = OrderedFloat(0.0);
            } else {
                matrix[(musician_id, candidate_id)] *= OrderedFloat(10.0);
            }
        }
    }

    let (score, assignments) = kuhn_munkres(&matrix);
    let mut filtered_candidates = vec![];
    let mut volumes = vec![];
    for assignment_id in 0..assignments.len() {
        let assignment = assignments[assignment_id];
        filtered_candidates.push(candidates[assignment].clone());
        if matrix[(assignment_id, assignment)].0 == 0.0 {
            volumes.push(0.0);
        } else {
            volumes.push(10.0);
        }
    }

    (score.0, filtered_candidates, volumes)
}

fn solve(input: &Input) -> (Vec<Point>, Vec<f64>) {
    let first_level_candidates = generate_first_level_candidates(input);
    // 最初はmusicianよりも多い候補地を用いて最適化を行い、その結果に基づき二段階目の最適化に使用する候補地を列挙
    let (first_level_score, second_level_candidates, _) =
        exact_match_candidates(input, &first_level_candidates);
    dbg!(first_level_score);
    let (second_level_score, best_placements, best_volumes) =
        exact_match_candidates(input, &second_level_candidates);
    dbg!(second_level_score);
    (best_placements, best_volumes)
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input.clone()).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let x_count = (input.stage_width / 10.0).floor() as usize - 1;
    let y_count = (input.stage_height / 10.0).floor() as usize - 1;
    let (best_placements, best_volumes) = if x_count * y_count >= input.musicians.len() {
        // Use new strategy!
        solve(&input)
    } else {
        // Give up
        let mut generator = PlacementGenerator::new(&input, args.rand_seed);
        (generator.generate(), vec![10.0; input.musicians.len()])
    };
    let mut solution: Solution = Default::default();
    solution.placements = best_placements.clone();
    solution.volumes = Some(best_volumes);
    input.is_valid_placements(&best_placements).unwrap();
    eprintln!("Solver score: {}", solution.score(&input).unwrap());
    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap();
}
