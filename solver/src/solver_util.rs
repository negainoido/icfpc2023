use crate::get_time;
use crate::problem::{Input, Segment, Solution};
use geo::Point;
use rand::Rng;

pub fn yamanobori(
    input: &Input,
    best_score: &mut f64,
    best: &mut Vec<Point>,
    best_volume: &Vec<f64>,
    timeout: f64,
    rand_seed: u128,
) -> Vec<Point> {
    let mut rng = rand_pcg::Pcg64Mcg::new(rand_seed);
    while get_time() < timeout {
        let mut current = best.clone();
        let idx = rng.gen_range(0..best.len());
        let dir = rng.gen_range(0..4);
        let dx = [0.0, 1.0, 0.0, -1.0];
        let dy = [1.0, 0.0, -1.0, 0.0];
        let step = rng.gen_range(1..100) as f64;
        *current[idx].x_mut() += dx[dir] * step;
        *current[idx].y_mut() += dy[dir] * step;

        if input.is_valid_placements(&current).is_err() {
            continue;
        }
        let solution = Solution {
            placements: current,
            volumes: Some(best_volume.clone()),
        };

        let current_score = input.score_fast(&solution);
        if let Ok(sc) = current_score {
            if sc > *best_score {
                eprintln!("score is improved: {} -> {}", *best_score, sc,);
                *best_score = sc;
                *best = solution.placements;
            }
        }
    }
    best.to_vec()
}

pub fn reduce_attendees(input: &Input) -> Input {
    let mut new_attendees = vec![];
    for attendee in &input.attendees {
        let mut min_dist: f64 = 1.0e9;
        for segment in [
            Segment {
                p1: input.stage_bottom_left,
                p2: input.stage_bottom_left + Point::new(input.stage_width, 0.0),
            },
            Segment {
                p1: input.stage_bottom_left,
                p2: input.stage_bottom_left + Point::new(0.0, input.stage_height),
            },
            Segment {
                p1: input.stage_bottom_left + Point::new(0.0, input.stage_height),
                p2: input.stage_bottom_left + Point::new(input.stage_width, input.stage_height),
            },
            Segment {
                p1: input.stage_bottom_left + Point::new(input.stage_width, 0.0),
                p2: input.stage_bottom_left + Point::new(input.stage_width, input.stage_height),
            },
        ] {
            min_dist = min_dist.min(segment.dist(&attendee.pos()));
        }
        new_attendees.push((min_dist, attendee));
    }

    new_attendees.sort_by_key(|k| ordered_float::OrderedFloat(k.0));
    new_attendees.truncate((new_attendees.len() / 5).max(100));

    let mut input = input.clone();
    input.attendees = new_attendees.into_iter().map(|v| v.1.clone()).collect();
    input
}

pub fn volume_optimize(input: &Input, solution: &Solution) -> Solution {
    let mut best_solution = solution.clone();
    let mut best_score = best_solution.score(&input).unwrap();

    // Volume optimize
    let mut tmp_solution = best_solution.clone();
    for i in 0..input.musicians.len() {
        let mut current_volume = best_solution
            .volumes
            .clone()
            .unwrap_or(vec![1.0; input.musicians.len()]);

        current_volume[i] = 10.0;
        tmp_solution.volumes = Some(current_volume.clone());
        match tmp_solution.score(&input) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = tmp_solution.clone();
                    println!("iter {}, score: {}", i, best_score);
                    continue;
                }
            }
            Err(e) => {
                println!("iter {} error exit: {:?}", i, e);
            }
        }
        current_volume[i] = 0.0;
        tmp_solution.volumes = Some(current_volume.clone());
        match tmp_solution.score(&input) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = tmp_solution.clone();
                    println!("iter {}, score: {}", i, best_score);
                    continue;
                }
            }
            Err(e) => {
                println!("iter {} error exit: {:?}", i, e);
            }
        }
    }

    best_solution
}
