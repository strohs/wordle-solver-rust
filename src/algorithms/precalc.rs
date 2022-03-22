//! A wordle solver algorithm that pre-calculates all possible combinations of:
//! word + word + mask
//!
use std::borrow::Cow;
use std::collections::{BTreeMap};
use once_cell::sync::OnceCell;
use crate::{Guesser, Guess, DICTIONARY, Correctness};

// holds the initial list of (word, count) from the dictionary, loaded only once
static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static MATCH: OnceCell<BTreeMap<(&'static str, &'static str, [Correctness; 5]), bool>> = OnceCell::new();

pub struct PreCalc {
    /// a map containing all possible words that could be a possible solution
    /// it maps a `word` -> `occurrence count`, where occurrence_count is the number of times
    /// that word appeared in books
    // Cow is used because we are either going to be borrowing a Dictionary or we are going to
    // own a dictionary once we start pruning words
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
}

impl PreCalc {

    /// Creates a new Once algorithm for solving wordle
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                // sort initial words in DESCSENDING order
                let mut words = Vec::from_iter(
                    DICTIONARY
                        .lines()
                        .map(|line| {
                            let (word, count) = line
                                .split_once(' ')
                                .expect("every line is a word + space + occurrence_count");
                            let count: usize = count.parse().expect("every count is a number");
                            (word, count)
                        }));
                words.sort_unstable_by_key(|&(_, c)| std::cmp::Reverse(c));
                words
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

impl Guesser for PreCalc {

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

        for &(word, _) in &*self.remaining {
            let mut sum = 0.0;

            // todo don't consider correctness patterns that had no candidates in the previous
            // iteration
            for pattern in Correctness::patterns() {
                // total of the count(s) of words that match a pattern
                let mut in_pattern_total: usize = 0;

                // given a particular candidate word, if we guess this word, what
                // are the probabilities of getting each pattern. We sum together all those
                // probabilities and use that to determine the entropy information amount from
                // guessing that word
                for &(candidate, count) in &*self.remaining {
                    // pre-computing matches HashMap here
                    let matches = MATCH.get_or_init(|| {
                        // considering a "world" where we did guess "word" and got "pattern" as the
                        // correctness. Now compute what _then_ is left

                        let initial_words = &INITIAL.get().expect("INITIAL vec of dictionary words is loaded")[..1024];
                        dbg!(&initial_words[..10]);
                        let mut out = BTreeMap::new();

                        for &(word1, _) in initial_words {
                            for &(word2, _) in initial_words {
                                if word2 < word1 { break; }
                                for pattern in Correctness::patterns() {
                                    let g = Guess {
                                        word: Cow::Borrowed(word),
                                        mask: pattern,
                                    };
                                    out.insert((word1, word2, pattern), g.matches(candidate));
                                }
                            }
                        }
                        out
                    });
                    // compute the map key so that it is in the correct order for lookup
                    let key = if word < candidate {
                        (word, candidate, pattern)
                    } else {
                        (candidate, word, pattern)
                    };
                    if matches.get(&key).copied().unwrap_or_else(|| {
                        let g = Guess {
                            word: Cow::Borrowed(word),
                            mask: pattern,
                        };
                        g.matches(candidate)
                    }) {
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
            // TODO weight this by prob_word
            let goodness = -sum;

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