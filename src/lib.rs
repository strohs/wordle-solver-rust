use std::collections::HashSet;

pub mod algorithms;

/// list of all 5 letter words
const DICTIONARY: &str = include_str!("../dictionary.txt");

// All wordle words are exactly 5 bytes (ASCII characters only)
type Word = [u8; 5];

pub struct Wordle {
    dictionary: HashSet<&'static Word>,
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
                            .as_bytes()
                            .try_into()
                            .expect("every word is exactly 5 bytes")
                    })),
        }
    }

    /// plays a game of wordle using the provided `guesser` to guess the `answer`
    /// returns `Some(round_number)` if the answer was guessed, else `None` if the guesser
    /// could not guess the answer
    pub fn play<G: Guesser>(&self, answer: Word, mut guesser: G) -> Option<usize> {

        // stores past guesses
        let mut history = Vec::new();

        // wordle only allows six guesses, we'll allow more chances in order to avoid
        // chopping off the score distribution for stats purposes
        for i in 1..=32 {
            let guess = guesser.guess(&history[..]);
            if guess == answer {
                return Some(i);
            }

            assert!(self.dictionary.contains(&guess));

            let correctness = Correctness::compute(answer, guess);
            history.push(Guess {
                word: guess,
                mask: correctness,
            })
        }
        None
    }
}

/// Correctness holds the three possible 'states' for the characters of a word when compared
/// against a wordle answer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    fn compute(answer: Word, guess: Word) -> [Self; 5] {
        let mut c = [Correctness::Wrong; 5];

        // first mark green chars
        for (i, (&a, &g)) in answer
            .iter()
            .zip(guess.iter())
            .enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }

        // mark the positions of characters that are correct
        let mut correctly_used = [false; 5];
        for (i, c) in c.iter().enumerate() {
            if *c == Correctness::Correct {
                correctly_used[i] = true;
            }
        }

        // next mark yellow (misplaced) chars
        for (i, &g) in guess.iter().enumerate() {
            if c[i] == Correctness::Correct {
                // already marked green
                continue;
            }
            // not worth optimizing away the bounds_check on correctly_used[]
            if answer.iter().enumerate().any(|(ai, &a)| {
                if a == g && !correctly_used[ai] {
                    correctly_used[ai] = true;
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
}

/// Guess holds the details of a guessed word.
/// It contains a guessed word along with the correctness mask of that word compared against
/// the actual answer
pub struct Guess {
    /// a word that was guessed
    pub word: Word,
    /// the correctness mask of each character of word when compared against the true answer
    pub mask: [Correctness; 5],
}

impl Guess {
    /// compares the given `word` against the word in this guess to see if `word` could be a
    /// plausible guess... a.k.a  a "match"
    /// returns `true` if `word` could be a plausible guess
    /// returns `false` if there is no possible way that `word` would match this Guess based on the
    /// Guesses mask data
    pub fn matches(&self, word: Word) -> bool {
        // using Correctness::compute is a 18x runtime improvement over using old matches
        return Correctness::compute(word, self.word) == self.mask;

        // assert_eq!(self.word.len(), 5);
        // assert_eq!(word.len(), 5);
        //
        // // keeps track of which chars in a word have been used thus far
        // let mut used = [false; 5];
        //
        // for (i, ((pg, &prev_guess_corr), w)) in self
        //     .word
        //     .bytes()
        //     .zip(&self.mask)
        //     .zip(word.bytes())
        //     .enumerate() {
        //     if prev_guess_corr == Correctness::Correct {
        //         if pg != w {
        //             return false;
        //         } else {
        //             used[i] = true;
        //         }
        //     }
        // }
        //
        // for (i, (w, &mask)) in word
        //     .bytes()
        //     .zip(&self.mask)
        //     .enumerate() {
        //     if mask == Correctness::Correct {
        //         // must be correct or we would've returned in the previous loop
        //         continue;
        //     }
        //     let mut plausible = true;
        //     if self.word
        //         .bytes()
        //         .zip(&self.mask)
        //         .enumerate()
        //         .any(|(j, (pg, &m))| {
        //             if pg != w {
        //                 return false;
        //             }
        //
        //             if used[j] {
        //                 return false;
        //             }
        //
        //             // we're looking at 'word_char' in 'word' and have found a 'word_char' in the
        //             // previous guess. The color of that previous 'word_char' will tell us whether
        //             // this 'word_char' might be okay
        //             match m {
        //                 Correctness::Correct => unreachable!("all correct guesses should have already returned or be used"),
        //                 Correctness::Misplaced if j == i => {
        //                     // 'w' was yellow in this same position last time around, which means that
        //                     // word cannot be the answer
        //                     plausible = false;
        //                     return false;
        //                 }
        //                 Correctness::Misplaced => {
        //                     used[j] = true;
        //                     return true;
        //                 }
        //                 Correctness::Wrong => {
        //                     plausible = false;
        //                     return false;
        //                 }
        //             }
        //         }) && plausible {
        //         // the character in 'w' was yellow in a previous match
        //     } else if !plausible {
        //         return false;
        //     } else {
        //         // we have no info on char 'w', so word might still match
        //     }
        // }
        // true
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> Word;
}

impl Guesser for fn(history: &[Guess]) -> Word {
    /// A guessing algorithm for wordle.
    /// We Need to find the 'goodness' score of each word remaining and then return the one
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
    fn guess(&mut self, history: &[Guess]) -> Word {
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
            fn guess(&mut self, $history: &[Guess]) -> $crate::Word {
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

        use crate::Guess;

        /// checks if a Guess matches a word
        /// Ex. `check!("abcde" + [C C C C C] allows "abcde");`
        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: *$prev,
                    mask: mask![$($mask )+]
                }
                .matches(*$next));
                assert_eq!($crate::Correctness::compute(*$next, *$prev), mask![$($mask )+]);
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: *$prev,
                    mask: mask![$($mask )+]
                }
                .matches(*$next));
                assert_ne!($crate::Correctness::compute(*$next, *$prev), mask![$($mask )+]);
            }
        }


        #[test]
        fn matches() {

            // checking previous guess + prev. mask, against the latest guessed word
            check!(b"abcde" + [C C C C C] allows b"abcde");
            check!(b"abcdf" + [C C C C C] disallows b"abcde");
            check!(b"abcde" + [W W W W W] allows b"fghij");
            check!(b"abcde" + [M M M M M] allows b"eabcd");
            check!(b"baaaa" + [W C M W W] allows b"aaccc");
            check!(b"baaaa" + [W C M W W] disallows b"caacc");

            check!(b"aaabb" + [C M W W W] disallows b"accaa");

            check!(b"tares" + [W M M W W] disallows b"brink");
        }
    }

    mod game {
        use crate::{Guess, Wordle};

        #[test]
        fn play_first_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { *b"right" });
            assert_eq!(w.play(*b"right", guesser), Some(1));
        }

        #[test]
        fn play_second_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return *b"right";
                }
                return *b"wrong";
            });

            assert_eq!(w.play(*b"right", guesser), Some(2));
        }

        #[test]
        fn play_third_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return *b"right";
                }
                return *b"wrong";
            });

            assert_eq!(w.play(*b"right", guesser), Some(3));
        }

        #[test]
        fn play_fourth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return *b"right";
                }
                return *b"wrong";
            });

            assert_eq!(w.play(*b"right", guesser), Some(4));
        }

        #[test]
        fn play_fifth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return *b"right";
                }
                return *b"wrong";
            });

            assert_eq!(w.play(*b"right", guesser), Some(5));
        }

        #[test]
        fn play_sixth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return *b"right";
                }
                return *b"wrong";
            });

            assert_eq!(w.play(*b"right", guesser), Some(6));
        }

        #[test]
        fn all_wrong_guesses_should_terminate() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { *b"wrong" });

            assert_eq!(w.play(*b"right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute(*b"abcde", *b"abcde"), mask!(C C C C C))
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute(*b"abcde", *b"qwxyz"), mask!(W W W W W))
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute(*b"abcde", *b"eabcd"), mask!(M M M M M))
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute(*b"aabbb", *b"aaccc"), mask!(C C W W W))
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute(*b"aabbb", *b"ccaac"), mask!(W W M M W))
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute(*b"aabbb", *b"caacc"), mask!(W C M W W))
        }

        #[test]
        fn some_green_some_yellow() {
            assert_eq!(Correctness::compute(*b"azzaz", *b"aaabb"), mask!(C M W W W))
        }

        #[test]
        fn one_green() {
            assert_eq!(Correctness::compute(*b"baccc", *b"aaddd"), mask!(W C W W W))
        }

        #[test]
        fn some_green_some_yellow2() {
            assert_eq!(Correctness::compute(*b"abcde", *b"aacde"), mask!(C W C C C))
        }
    }
}