//! A wordle solver algorithm that prunes correctness patterns that can no
//! longer be valid at each iteration of a guess
//!
use std::borrow::Cow;
use once_cell::sync::OnceCell;
use crate::{Guesser, Guess, DICTIONARY, Correctness};

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Prune {
    /// a `Vec<(word, count)>` containing all possible words (and their occurrence count) that
    /// could be a possible solution.
    // Cow is used because we are either going to be borrowing a Dictionary or we are going to
    // own a dictionary once we start pruning words
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
    /// holds all possible wordle correctness patterns, 3^5 elements
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Prune {

    /// creates a new Prune algo, loads the word dictionary if not already loaded
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
            patterns: Cow::Borrowed(PATTERNS.get_or_init(|| Vec::from_iter(Correctness::patterns()))),
        }
    }

    /// prune the list of remaining words by only keeping words that could be a possible match
    /// with the `last_guess`
    fn prune_remaining(&mut self, last_guess: &Guess) {
        if matches!(self.remaining, Cow::Owned(_)) {
            // if the remaining Vec is already owned, mutate it to retain the matching words
            self.remaining
                .to_mut()
                .retain(|(word, _)| last_guess.matches(word));
        } else {
            // else, create a new owned Vec, but first, filter the matching words
            self.remaining = Cow::Owned(self.remaining
                .iter()
                .filter(|(word, _)| last_guess.matches(word))
                .copied()
                .collect());
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

impl Guesser for Prune {

    fn guess(&mut self, history: &[Guess]) -> String {

        if let Some(last) = history.last() {
            self.prune_remaining(last);
        }

        // hardcode the first guess to "tares"
        if history.is_empty() {
            self.patterns = Cow::Borrowed(PATTERNS.get().unwrap());
            return "tares".to_string();
        } else {
            // there should be patterns left if we are still guessing
            assert!(!self.patterns.is_empty());
        }

        // the sum of the counts of all the remaining words in the dictionary
        let remaining_word_count: usize = self.remaining
            .iter()
            .map(|&(_, c)| c).sum();
        // the best candidate so far
        let mut best: Option<Candidate> = None;

        for &(word, count) in &*self.remaining {
            let mut sum = 0.0;

            // lambda that checks if the given pattern matches
            let check_pattern = |pattern: &[Correctness; 5]| {
                // sum of the count(s) of all words that match the pattern
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
                        mask: *pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    return false;
                }
                let prob_of_this_pattern = in_pattern_total as f64 / remaining_word_count as f64;
                sum += prob_of_this_pattern * prob_of_this_pattern.log2();
                return true;
            };

            // if patterns is already owned, then start pruning with owned vec, else
            // create a new owned vec but filter valid patterns first
            if matches!(self.patterns, Cow::Owned(_)) {
                self.patterns.to_mut().retain(check_pattern);
            } else {
                self.patterns = Cow::Owned(self.patterns
                    .iter()
                    .copied()
                    .filter(check_pattern)
                    .collect());
            }
            // compute the probability of the current word using its occurrence count
            let p_word = count as f64 / remaining_word_count as f64;
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