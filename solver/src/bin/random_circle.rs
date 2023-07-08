use clap::Parser;
use geo::Point;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;

use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(long, short = 'I', default_value_t = 10)]
    iteration: i32,

    #[arg(short, long)]
    timeout: Option<u32>,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();
    let now = std::time::SystemTime::now();

    let mut solution: Solution = Default::default();

    let rx = input.stage_bottom_left.x() + input.stage_width / 2.0;
    let ry = input.stage_bottom_left.y() + input.stage_height / 2.0;
    let musician_dist = 10.0 + 1e-5;
    trace!((rx, ry));

    let mut ps = vec![];

    ps.push(Point::new(rx, ry));
    let mut d: f64 = musician_dist;
    let mut dtheta: f64 = (0.5_f64).acos();
    let mut theta: f64 = 0.0;
    while d < input.stage_width || d < input.stage_height {
        let p = Point::new(rx + d * theta.cos(), ry + d * theta.sin());
        if input.in_stage(&p) {
            ps.push(p);
        }
        theta += dtheta;
        if theta + dtheta > 6.28 {
            d += musician_dist;
            theta = 0.0;
            dtheta = (1.0 - musician_dist.powf(2.0) / (2.0 * d.powf(2.0))).acos();
        }
        if ps.len() >= input.musicians.len() + 3 {
            break;
        }
    }
    trace!(ps.len());
    solution.placements = ps.iter().take(input.musicians.len()).cloned().collect();
    let mut rnd = Pcg64Mcg::new(args.rand_seed);
    let mut best_solution = solution.clone();
    let mut best_score = solution.score(&input).unwrap();
    println!("initial score: {}", best_score);
    let mut iter = 0;
    loop {
        ps.shuffle(&mut rnd);
        solution.placements = ps.iter().take(input.musicians.len()).cloned().collect();
        match solution.score(&input) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = solution.clone();
                    println!("iter {}, score: {}", iter, best_score);
                }
            }
            Err(_) => {}
        }
        if let Some(timeout_sec) = args.timeout {
            if let Ok(elapsed) = now.elapsed() {
                if elapsed.as_millis() > timeout_sec as u128 * 1000 {
                    break;
                }
            }
        } else {
            if iter > args.iteration {
                break;
            }
        }
        iter += 1;
    }

    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}

#[macro_export]
macro_rules! trace {
    ($x:expr) => {
        #[cfg(debug_assertions)]
        eprintln!(">>> {} = {:?}", stringify!($x), $x)
    };
    ($($xs:expr),*) => { trace!(($($xs),*)) }
}
