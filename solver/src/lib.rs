pub mod problem;
#[cfg(target_arch = "wasm32")]
mod wasm_util;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;
#[cfg(target_arch = "wasm32")]
use crate::wasm_util::Placement;

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
    placement: Vec<JsValue>
) -> Result<f64, JsValue> {
    use geo::Point;
    use crate::problem::Attendee;

    let solution = problem::Solution {
        placements: placement.iter().map(|p| {
            let p: Placement = serde_wasm_bindgen::from_value(p.into()).unwrap();
            Point::new(p.x, p.y)
        }).collect(),
    };
    let input = problem::Input {
        room_width,
        room_height,
        stage_width,
        stage_height,
        stage_bottom_left: Point::new(stage_bottom_left[0], stage_bottom_left[1]),
        musicians,
        attendees: attendees.iter().map(|a| {
            let a: Attendee = serde_wasm_bindgen::from_value(a.into()).unwrap();
            a
        }).collect()
    };

    solution.score(&input).map_err(|e| JsValue::from_str(&format!("{}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
