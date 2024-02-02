use clap::Parser;

pub mod game_of_life;

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, default_value = "4")]
    width: u32,
    #[clap(long, default_value = "4")]
    height: u32,
    #[clap(short, long, default_value = "10")]
    generations: u32,
}

fn main() {
    let args = Args::parse();
    let width = args.width;
    let height = args.height;
    let generations = args.generations;
    println!(
        "Running Game of Life with width: {:?}, height: {:?}, generations: {:?}",
        width,
        height,
        generations
    );
    game_of_life::initialize(width, height, generations);
}
