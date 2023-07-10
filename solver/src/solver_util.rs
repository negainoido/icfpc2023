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
    reduce_num: usize,
) -> Vec<Point> {
    let mut rng = rand_pcg::Pcg64Mcg::new(rand_seed);
    let input = reduce_attendees(input, reduce_num);
    let mut best_score = input
        .score_fast(&Solution {
            placements: best.clone(),
            volumes: Some(best_volume.clone()),
        })
        .unwrap();

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
            if sc > best_score {
                eprintln!(
                    "score for reduced attendees is improved: {} -> {}",
                    best_score, sc,
                );
                best_score = sc;
                *best = solution.placements;
            }
        }
    }
    best.to_vec()
}

pub fn reduce_attendees(input: &Input, num: usize) -> Input {
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
    new_attendees.truncate(num);
    let mut input = input.clone();
    input.attendees = new_attendees.into_iter().map(|v| v.1.clone()).collect();
    input
}

pub fn volume_optimize(input: &Input, solution: &Solution) -> Solution {
    let mut solution = solution.clone();
    let mut best_score = solution.score(&input).unwrap();

    let original_volumes = solution
        .volumes
        .clone()
        .unwrap_or(vec![1.0; input.musicians.len()]);
    solution.volumes = Some(original_volumes);

    // Volume optimize
    for i in 0..input.musicians.len() {
        for vol in [0.0, 0.1, 9.9, 10.0] {
            let tmp = solution.volumes.as_ref().map(|v| v[i]).unwrap_or(1.0);
            if let Some(volumes) = &mut solution.volumes {
                volumes[i] = vol;
            }
            match solution.score(&input) {
                Ok(score) => {
                    if score > best_score {
                        best_score = score;
                        println!("iter {}, score: {}", i, best_score);
                        continue;
                    } else {
                        if let Some(volumes) = &mut solution.volumes {
                            volumes[i] = tmp;
                        }
                    }
                }
                Err(e) => {
                    println!("iter {} error exit: {:?}", i, e);
                }
            }
        }
    }
    solution
}
