use std::borrow::Cow;
use std::collections::HashSet;
use anyhow::anyhow;

pub mod algorithms;

/// list of all 5 letter words
const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            // step by 2 because every other token in Dictionary is a words frequency count
            dictionary: HashSet::from_iter(
                DICTIONARY
                    .lines()
                    .map(|line| {
                        line.split_once(' ')
                            .expect("every line is a word + space + occurrence_count")
                            .0
                    })),
        }
    }

    /// plays a game of wordle using the provided `guesser` to guess the `answer`
    /// returns `Some(round_number)` if the answer was guessed, else `None` if the guesser
    /// could not guess the answer
    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {

        // stores past guesses
        let mut history = Vec::new();

        // wordle only allows six guesses, we'll allow more chances in order to avoid
        // chopping off the score distribution for stats purposes
        for i in 1..=32 {
            let guess = guesser.guess(&history[..]);
            if guess == answer {
                return Some(i);
            }

            assert!(self.dictionary.contains(&*guess));

            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            })
        }
        None
    }
}

impl Default for Wordle {
    fn default() -> Self {
        Self::new()
    }
}

/// Correctness holds the three possible 'states' for the characters of a word when compared
/// against a wordle answer.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Correctness {
    /// Green
    Correct,
    /// Yellow,
    Misplaced,
    /// Gray
    Wrong,
}

impl Correctness {
    /// computes and returns the Correctness "mask" for each character of the given `guess`
    /// when compared against the characters of the given `answer`.
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];

        // mark green chars
        for (i, (a, g)) in answer
            .bytes()
            .zip(guess.bytes())
            .enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }
        // mark yellow chars
        let mut used = [false; 5];
        for (i, c) in c.iter().enumerate() {
            if *c == Correctness::Correct {
                used[i] = true;
            }
        }
        for (i, g) in guess.bytes().enumerate() {
            if c[i] == Correctness::Correct {
                // already marked green
                continue;
            }
            if answer.bytes().enumerate().any(|(ai, a)| {
                if a == g && !used[ai] {
                    used[ai] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }
        c
    }

    /// computes the Cartesian Product of all possible correctness patterns for a 5 letter word.
    /// returns an Iterator over an array containing a possible pattern
    ///
    /// There are 3 correctness patterns for each of the 5 character positions in a word, so the
    /// total patterns will be of length 3^5.
    /// Some patterns are impossible to reach so in reality this would be slightly
    /// less than 3^5, but it should not affect our calculations. We'll generate the Cartesian
    /// Product and optimize later
    pub fn patterns() -> impl Iterator<Item=[Self; 5]> {
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong]
        )
            .map(|(a, b, c, d, e)| [a, b, c, d, e])
    }

    pub fn try_from_str(s: &str) -> Result<[Correctness; 5], anyhow::Error> {
        if s.len() != 5 {
            Err(anyhow!("correctness masks must be 5 characters"))
        } else {
            let mut mask = [Correctness::Wrong; 5];
            for (i, c) in s.chars().enumerate() {
                mask[i] = Correctness::try_from(c)?
            }
            Ok(mask)
        }
    }
}

impl TryFrom<char> for Correctness {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase() {
            'c' => Ok(Correctness::Correct),
            'm' => Ok(Correctness::Misplaced),
            'w' => Ok(Correctness::Wrong),
            invalid => Err(anyhow!("invalid correctness: '{}'", invalid))
        }
    }
}


/// Guess holds the details of a guessed word.
/// It contains a guessed word along with the correctness mask of that word compared against
/// the actual answer
pub struct Guess<'a> {
    /// a word that was guessed
    pub word: Cow<'a, str>,
    /// the correctness mask of each character of word when compared against the true answer
    pub mask: [Correctness; 5],
}

impl Guess<'_> {
    /// compares the given `word` against the word in this guess to see if `word` could be a
    /// plausible guess... a.k.a  a "match"
    /// returns `true` if `word` could be a plausible guess
    /// returns `false` if there is no possible way that `word` would match this Guess based on the
    /// Guesses mask data
    pub fn matches(&self, word: &str) -> bool {
        // using Correctness::compute is a 18x runtime improvement over using old matches
        Correctness::compute(word, &self.word) == self.mask
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    /// A guessing algorithm for wordle.
    /// We need to find the 'goodness' score of each word remaining and then return the one
    /// with the highest goodness. We'll use information theory to compute the expected
    /// amount of information we would gain if a word isn't the answer, combined with
    /// the probability of words that are likely to be the answer. This is the formula we
    /// will use:
    /// `- SUM_i prob_i * log_2(prob_i)`
    ///
    /// # Example
    /// imagine we have a list of possible candidate words: [word_1, word_2, ..., word_n]
    /// and we want to determine the "goodness" score of word_i.
    /// The goodness is the sum of the goodness of each possible pattern we MIGHT see
    /// as a result of guessing it, multiplied by the likely-hood of that pattern occurring.
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

/// helper macro that returns a struct implementing the Guesser trait.
/// It allows you to pass in a closure that can be used to mock the results of the guess fn
///
/// # Example
/// `guesser!(|_history| { "moved".to_string() });`
#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

/// maps a list of C,M,W tokens into an array of Correctness variants
#[cfg(test)]
macro_rules! mask {
    (C) => { $crate::Correctness::Correct };
    (M) => { $crate::Correctness::Misplaced };
    (W) => { $crate::Correctness::Wrong };
    ($($c:tt)+) => {[
        $(mask!($c)),+
    ]}
}

#[cfg(test)]
mod tests {
    mod guess_matcher {
        use std::borrow::Cow;

        use crate::Guess;

        /// checks if a Guess matches a word
        /// Ex. `check!("abcde" + [C C C C C] allows "abcde");`
        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask )+]
                }.matches($next));
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask )+]
                }.matches($next));
            }
        }

        #[test]
        fn matches() {
            // checking previous guess + prev. mask, against the latest guessed word
            check!("abcde" + [C C C C C] allows "abcde");
            check!("abcdf" + [C C C C C] disallows "abcde");
            check!("abcde" + [W W W W W] allows "fghij");
            check!("abcde" + [M M M M M] allows "eabcd");
            check!("baaaa" + [W C M W W] allows "aaccc");
            check!("baaaa" + [W C M W W] disallows "caacc");

            check!("aaabb" + [C M W W W] disallows "accaa");

            check!("tares" + [W M M W W] disallows "brink");
        }
    }

    mod game {
        use crate::{Guess, Wordle};

        #[test]
        fn play_first_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }

        #[test]
        fn play_second_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(2));
        }

        #[test]
        fn play_third_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(3));
        }

        #[test]
        fn play_fourth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(4));
        }

        #[test]
        fn play_fifth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(5));
        }

        #[test]
        fn play_sixth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(6));
        }

        #[test]
        fn all_wrong_guesses_should_terminate() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });

            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask!(C C C C C))
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "qwxyz"), mask!(W W W W W))
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask!(M M M M M))
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask!(C C W W W))
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask!(W W M M W))
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask!(W C M W W))
        }

        #[test]
        fn some_green_some_yellow() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask!(C M W W W))
        }

        #[test]
        fn one_green() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask!(W C W W W))
        }

        #[test]
        fn some_green_some_yellow2() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask!(C W C C C))
        }
    }
}