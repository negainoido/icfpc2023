use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
}
