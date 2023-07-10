use geo::Point;
use rand::Rng;
use crate::get_time;
use crate::problem::{Input, Solution};

pub fn yamanobori(
    input: &Input,
    best_score: &mut f64,
    best: &mut Vec<Point>,
    best_volume: &Vec<f64>,
    timeout: f64,
) -> Vec<Point> {
    let mut rng = rand::thread_rng();
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

        let current_score = input.score_fast(&solution, false);
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
