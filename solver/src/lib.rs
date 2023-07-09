pub mod problem;
#[cfg(target_arch = "wasm32")]
mod wasm_util;

use anyhow::Context;
use geo::Point;
use problem::*;

#[cfg(not(target_arch = "wasm32"))]
use rand::prelude::SliceRandom;
#[cfg(not(target_arch = "wasm32"))]
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rand_pcg::Pcg64Mcg;

#[cfg(target_arch = "wasm32")]
use crate::wasm_util::Placement;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn calc_score(
    room_width: f64,
    room_height: f64,
    stage_width: f64,
    stage_height: f64,
    stage_bottom_left: Vec<f64>,
    musicians: Vec<usize>,
    attendees: Vec<JsValue>,
    pillars: Vec<JsValue>,
    placement: Vec<JsValue>,
    is_full: bool,
) -> Result<f64, JsValue> {
    use crate::problem::Attendee;
    use geo::Point;

    let solution = problem::Solution {
        placements: placement
            .iter()
            .map(|p| {
                let p: Placement = serde_wasm_bindgen::from_value(p.into()).unwrap();
                Point::new(p.x, p.y)
            })
            .collect(),
    };
    let input = problem::Input {
        room_width,
        room_height,
        stage_width,
        stage_height,
        stage_bottom_left: Point::new(stage_bottom_left[0], stage_bottom_left[1]),
        musicians,
        attendees: attendees
            .iter()
            .map(|a| {
                let a: Attendee = serde_wasm_bindgen::from_value(a.into()).unwrap();
                a
            })
            .collect(),
        pillars: pillars
            .iter()
            .map(|p| {
                let p: crate::wasm_util::Pillar = serde_wasm_bindgen::from_value(p.into()).unwrap();

                Pillar {
                    center: Point::new(p.center[0], p.center[1]),
                    radius: p.radius,
                }
            })
            .collect(),
    };

    solution
        .score(&input, is_full)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(not(target_arch = "wasm32"))]
pub struct PlacementGenerator<'a> {
    pub input: &'a Input,
    pub cartesian_coordinate_candidates: Vec<Point>,
    pub honeycomb_candidates: Vec<Point>,
    rand_gen: Pcg64Mcg,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> PlacementGenerator<'a> {
    pub fn new(input: &'a Input, rand_seed: u128) -> Self {
        let rand_gen = Pcg64Mcg::new(rand_seed);

        let cartesian_coordinate_candidates =
            PlacementGenerator::cartesian_coordinate_candidates(&input);

        let honeycomb_candidates = PlacementGenerator::honeycomb_candidates(&input);

        PlacementGenerator {
            input,
            cartesian_coordinate_candidates,
            honeycomb_candidates,
            rand_gen,
        }
    }

    pub fn cartesian_coordinate_candidates(input: &Input) -> Vec<Point> {
        let mut candidates = vec![];
        let mut cx = input.stage_bottom_left.x() + 10.0;
        let mut cy = input.stage_bottom_left.y() + 10.0;
        loop {
            candidates.push(Point::new(cx, cy));
            cx += 20.0;
            if cx + 10.0 > input.stage_bottom_left.x() + input.stage_width {
                cx = input.stage_bottom_left.x() + 10.0;
                cy += 20.0;
            }
            if cy + 10.0 > input.stage_bottom_left.y() + input.stage_height {
                break;
            }
        }
        candidates
    }

    pub fn honeycomb_candidates(input: &Input) -> Vec<Point> {
        let mut candidates = vec![];
        let musician_dist = 10.0;
        let mut cx = input.stage_bottom_left.x() + musician_dist;
        let mut cy = input.stage_bottom_left.y() + musician_dist;
        let mut j = 0;
        loop {
            candidates.push(Point::new(cx, cy));
            cx += musician_dist;
            if cx + musician_dist > input.stage_bottom_left.x() + input.stage_width {
                cx = input.stage_bottom_left.x() + musician_dist;
                j += 1;
                if j % 2 == 1 {
                    cx += musician_dist / 2.0;
                }
                cy += musician_dist * f64::sqrt(3.0 + 1e-7) / 2.0;
            }

            if cy + musician_dist > input.stage_bottom_left.y() + input.stage_height {
                break;
            }
        }
        candidates
    }

    pub fn generate(&mut self) -> Vec<Point> {
        let mut candidates =
            if self.cartesian_coordinate_candidates.len() < self.input.musicians.len() {
                self.honeycomb_candidates.clone()
            } else if self.honeycomb_candidates.len() < self.input.musicians.len() {
                self.cartesian_coordinate_candidates.clone()
            } else {
                if self.rand_gen.gen_bool(0.5) {
                    self.honeycomb_candidates.clone()
                } else {
                    self.cartesian_coordinate_candidates.clone()
                }
            };

        candidates.shuffle(&mut self.rand_gen);
        candidates
            .iter()
            .take(self.input.musicians.len())
            .cloned()
            .collect()
    }
}

pub fn get_id(input: &str) -> anyhow::Result<i32> {
    let path = std::path::Path::new(&input);
    let base = path.file_stem().context("no file_stem?")?;
    let base = base.to_str().context("to_str failed")?;
    let id = base
        .strip_prefix("problem-")
        .context("no problem- prefix?")?;
    let id = id.parse::<i32>()?;
    Ok(id)
}

pub fn is_lightning(path: &str) -> anyhow::Result<bool> {
    let id = get_id(path)?;
    Ok(id <= 55)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id() {
        assert_eq!(get_id("../problems/problem-42.json").unwrap(), 42);
    }

    #[test]
    fn test_is_lightning() {
        assert!(is_lightning("../problems/problem-55.json").unwrap());
        assert!(!is_lightning("../problems/problem-56.json").unwrap());
    }
}
