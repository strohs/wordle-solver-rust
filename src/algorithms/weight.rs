//! A wordle solver algorithm that weights more common words better
//! than less common words when computing the goodness score. This results in
//! winning games in fewer turns since Wordle actually uses more common 5-letter words.
//!
use std::borrow::Cow;
use std::sync::OnceLock;
use crate::{Guesser, Guess, DICTIONARY, Correctness};

static INITIAL: OnceLock<Vec<(&'static str, usize)>> = OnceLock::new();

pub struct Weight {
    /// a map containing all possible words that could be a possible solution
    /// it maps a `word` -> `occurrence count`, where occurrence_count is the number of times
    /// that word appeared in books
    // Cow is used because we are either going to be borrowing a Dictionary or we are going to
    // own a dictionary once we start pruning words
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
}

impl Weight {

    /// Creates a new Weight algorithm for solving wordle
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                Vec::from_iter(
                    DICTIONARY
                        .lines()
                        .map(|line| {
                            let (word, count) = line
                                .split_once(' ')
                                .expect("every line is a word + space + occurrence_count");
                            let count: usize = count.parse().expect("every count is a number");
                            (word, count)
                        }))
            })),
        }
    }
}

/// Holds the details of a potential best guess
#[derive(Debug, Copy, Clone)]
struct Candidate {
    /// the candidate word
    word: &'static str,
    /// the candidates 'goodness' score, or entropy 'bits'. Higher is better
    goodness: f64,
}

impl Guesser for Weight {

    fn guess(&mut self, history: &[Guess]) -> String {

        // prune the dictionary by only keeping words that could be a possible match
        if let Some(last) = history.last() {
            if matches!(self.remaining, Cow::Owned(_)) {
                // if the remaining Vec is already owned, just retain the matching words
                self.remaining
                    .to_mut()
                    .retain(|(word, _)| last.matches(word));
            } else {
                // else, create a new owned Vec from filtering the matching words
                self.remaining = Cow::Owned(self.remaining
                    .iter()
                    .filter(|(word, _)| last.matches(word))
                    .copied()
                    .collect());
            }

        }

        // hardcode the first guess to "tares"
        if history.is_empty() {
            return "tares".to_string();
        }

        // the sum of the counts of all the remaining words in the dictionary
        let remaining_count: usize = self.remaining
            .iter()
            .map(|&(_, c)| c).sum();
        // the best word
        let mut best: Option<Candidate> = None;

        for &(word, count) in &*self.remaining {
            let mut sum = 0.0;

            for pattern in Correctness::patterns() {
                // total of the count(s) of words that match a pattern
                let mut in_pattern_total: usize = 0;

                // given a particular candidate word, if we guess this word, what
                // are the probabilities of getting each pattern. We sum together all those
                // probabilities and use that to determine the entropy information amount from
                // guessing that word
                for &(candidate, count) in &*self.remaining {
                    // considering a "world" where we did guess "word" and got "pattern" as the
                    // correctness. Now compute what _then_ is left
                    let g = Guess {
                        word: Cow::Borrowed(word),
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
            // compute the probability of the current word using its occurrence count
            let p_word = count as f64 / remaining_count as f64;
            // negate the sum to get the final goodness amount, a.k.a the entropy "bits"
            // factor in the p_word when computing goodness
            let goodness = p_word * -sum;

            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }
        }
        best.unwrap().word.to_string()
    }
}