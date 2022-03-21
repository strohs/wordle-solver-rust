use std::collections::HashMap;
use crate::{Guesser, Guess, DICTIONARY, Correctness, Word};

/// A "naive", i.e. unoptimized, wordle solver algorithm
pub struct Unoptimized {
    /// a map containing all possible words that could be a possible solution
    /// it maps a `word` -> `occurrence count`, where occurrence_count is the number of times
    /// that word appeared in books
    remaining: HashMap<Word, usize>,
}

impl Unoptimized {

    /// Creates a new algorithm for solving wordle
    pub fn new() -> Self {
        Self {
            remaining: HashMap::from_iter(
                DICTIONARY
                    .lines()
                    .map(|line| {
                        let (word, count) = line
                            .split_once(' ')
                            .expect("every line is a word + space + occurrence_count");
                        let count: usize = count.parse().expect("every count is a number");
                        let word = word.as_bytes().try_into().expect("every dictionary word is 5 characters");
                        (word, count)
                    })),
        }
    }
}

/// Holds the details of a potential best guess
#[derive(Debug, Copy, Clone)]
struct Candidate {
    /// the candidate word
    word: Word,
    /// the candidates 'goodness' score, or entropy 'bits'. Higher is better
    goodness: f64,
}

impl Guesser for Unoptimized {

    fn guess(&mut self, history: &[Guess]) -> Word {

        // prune the dictionary by only keeping words that could be a possible match
        if let Some(last) = history.last() {
            self.remaining.retain(|&word, _| last.matches(word));
        }

        // hardcode the first guess to "tares"
        if history.is_empty() {
            return *b"tares";
        }

        // the sum of the counts of all the remaining words in the dictionary
        let remaining_count: usize = self.remaining.iter().map(|(_, &c)| c).sum();
        // the best word
        let mut best: Option<Candidate> = None;

        for (&word, _) in &self.remaining {
            let mut sum = 0.0;

            for pattern in Correctness::patterns() {
                // total of the count(s) of words that match a pattern
                let mut in_pattern_total: usize = 0;

                // given a particular candidate word, if we guess this word, what
                // are the probabilities of getting each pattern. We sum together all those
                // probabilities and use that to determine the entropy information amount from
                // guessing that word
                for (&candidate, &count) in &self.remaining {
                    // considering a "world" where we did guess "word" and got "pattern" as the
                    // correctness. Now compute what _then_ is left
                    let g = Guess {
                        word: word,
                        mask: pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    continue;
                }
                let prob_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += prob_of_this_pattern * prob_of_this_pattern.log2()
            }
            // negate the sum to get the final goodness amount, a.k.a the entropy "bits"
            let goodness = -sum;

            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }
        }
        best.unwrap().word
    }
}