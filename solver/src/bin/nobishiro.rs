use clap::Parser;

use geo::EuclideanDistance;
use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    sync_effect: u8,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let generator = solver::PlacementGenerator::new(&input, 0);

    let mut score = 0i64;
    let candidates = generator.honeycomb_candidates;
    for attendee in input.attendees {
        let mut musicians = input.musicians.clone();
        // 嗜好性でソート
        musicians.sort_by(|a, b| {
            attendee.tastes[*a]
                .partial_cmp(&attendee.tastes[*b])
                .unwrap()
        });
        // 昇順に並び替え
        musicians.reverse();
        let mut musician_candidates = candidates.clone();
        // musician_candidatesを観客に近い順でソートする。
        musician_candidates.sort_by(|a, b| {
            let d1 = a.euclidean_distance(&attendee.pos());
            let d2 = b.euclidean_distance(&attendee.pos());
            d1.partial_cmp(&d2).unwrap()
        });
        musician_candidates.reverse();

        // 音楽家の配置場所
        let mut assignment = vec![];

        // 音楽家の配置を決める
        for musician in musicians {
            let taste = attendee.tastes[musician];
            if taste <= 0. {
                break;
            }
            // 最も近い配置場所にいると仮定する。
            let pos = musician_candidates.pop().unwrap();
            assignment.push((musician, pos))
        }

        // スコア計算
        let volume = 10.0f64;
        for i in 0..assignment.len() {
            let (musician, pos) = assignment[i];
            let mut sync_effect = 1.0;
            if args.sync_effect > 0 {
                for j in 0..assignment.len() {
                    if i == j {
                        continue;
                    }
                    let (other_musician, other_pos) = assignment[j];
                    if musician != other_musician {
                        continue;
                    }
                    sync_effect += 1.0 / pos.euclidean_distance(&other_pos);
                }
            }
            let d = pos.euclidean_distance(&attendee.pos());
            score +=
                (sync_effect * volume * f64::ceil(1000000.0 * attendee.tastes[musician] / (d * d)))
                    as i64;
        }
    }
    println!("Score: {}", score);
}
