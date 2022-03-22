use clap::{ArgEnum, Parser};
use wordle_solver::Guesser;

const GAMES: &str = include_str!("../answers.txt");


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the wordle guesser implementation to use
    #[clap(short, long, arg_enum)]
    implementation: Implementation,

    /// max Number of games to play
    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(ArgEnum, Debug, Copy, Clone)]
enum Implementation {
    Unoptimized,
    Allocs,
    Vecrem,
    Once
}



fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Unoptimized => {
            play(|| wordle_solver::algorithms::Unoptimized::new(), args.max);
        },
        Implementation::Allocs => {
            play(|| wordle_solver::algorithms::Allocs::new(), args.max);
        },
        Implementation::Vecrem => {
            play(|| wordle_solver::algorithms::Vecrem::new(), args.max);
        },
        Implementation::Once => {
            play(|| wordle_solver::algorithms::OnceInit::new(), args.max);
        },
    }
}

fn play<G>(mut maker: impl FnMut() -> G, max: Option<usize>) where G: Guesser {
    let w = wordle_solver::Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (maker)();
        if let Some(score) = w.play(answer, guesser) {
            println!("guessed '{}' in {}", &answer, score);
        } else {
            eprintln!("failed to guess..zoinks!");
        }
    }
}