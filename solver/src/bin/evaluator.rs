use anyhow::Result;
use clap::Parser;
use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input JSON path
    #[arg(short, long)]
    input: String,
    /// Solution JSON path
    #[arg(short, long)]
    solution: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input)?;
    let input: Input = serde_json::from_str(&input_str)?;
    let solution_str = std::fs::read_to_string(args.solution)?;
    let solution: Solution = serde_json::from_str(&solution_str)?;

    match solution.score(&input, false) {
        Ok(score) => println!("Score: {}", score),
        Err(e) => {
            println!("Invalid solution: {:#}", e);
        }
    }

    Ok(())
}
