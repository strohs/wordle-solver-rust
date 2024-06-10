use std::borrow::Cow;
use std::io::Write;
use anyhow::anyhow;
use clap::{ArgEnum, Parser};
use wordle_solver::{Correctness, Guess, Guesser};

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

/// various Wordle guesser implementations
#[derive(ArgEnum, Debug, Copy, Clone)]
enum Implementation {
    Unoptimized,
    Allocs,
    Vecrem,
    Once,
    Precalc,
    Weight,
    Prune
}

fn main() -> Result<(), anyhow::Error> {
    // use the Prune algorithm as it is the fastest so far
    let mut guesser = wordle_solver::algorithms::Prune::new();
    let mut guess_history: Vec<Guess> = Vec::new();

    println!("Enter a guess and its resulting correctness mask separated by a space then press ENTER, example:'tares ccwmm'");
    for turn in 1.. {
        print!("Turn {} Guess and Pattern:", turn);
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)?;
        let (word, mask) = input
            .trim_end()
            .split_once(' ')
            .ok_or_else(|| anyhow!("guess and mask must be separated by one space"))?;

        let correctness = Correctness::try_from_str(mask)?;
        let guess = Guess {
            word: Cow::Owned(word.to_string()),
            mask: correctness,
        };
        guess_history.push(guess);
        let best_word = guesser.guess(&guess_history);
        println!("try this guess... {}", &best_word);
    }
    Ok(())
}
