use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: Pos,
    pub musicians: Vec<u64>,
    pub attendees: Vec<Attendee>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Solution {
    pub placements: Vec<Pos>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn parse_input() {
        let input = r#"
        {
  "room_width": 2000,
  "room_height": 5000,
  "stage_width": 1000,
  "stage_height": 200,
  "stage_bottom_left": [
    500,
    0
  ],
  "musicians": [
    0,
    1,
    0
  ],
  "attendees": [
    {
      "x": 100,
      "y": 500,
      "tastes": [
        1000,
        -1000
      ]
    },
    {
      "x": 200,
      "y": 1000,
      "tastes": [
        200,
        200
      ]
    },
    {
      "x": 1100,
      "y": 800,
      "tastes": [
        800,
        1500
      ]
    }
  ]
}
        "#;
        let _input: Input = serde_json::from_str(input).unwrap();
    }

    #[test]
    fn parse_solution() {
        let solution = r#"{
  "placements": [
    {
      "x": 590,
      "y": 10
    },
    {
      "x": 1100,
      "y": 100
    },
    {
      "x": 1100,
      "y": 150
    }
  ]
}"#;
        let _solution: Solution = serde_json::from_str(solution).unwrap();
    }
}
