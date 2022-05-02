# Rust Wordle Solver
A command line wordle solver that uses information theory to solve a game of [wordle](https://www.nytimes.com/games/wordle/index.html). 
It's inspired by [3Blue1Brown](https://www.youtube.com/watch?v=v68zYyaEmEA) and
[Jon Gjenset's](https://www.youtube.com/watch?v=doFowk4xj7Q) YouTube videos on this topic.

## Running
This solver is a command line program that is used at the same time you are making guesses into the wordle web application.
You make your first guess into the web app, and then enter that guess along with its corresponding "correctness" pattern
into the wordle solver, separated by a space. 

The correctness pattern is a five character string that represents the correctness of each letter of your guess as reported by wordle.
It consists of the characters `c`,`m`, or `w`.
- `c` a letter of the guess is in the **correct** position, the wordle web app shows these with a green background
- `m` a letter of the guess is in the answer but is **misplaced**, or in the incorrect position, wordle shows these with a yellow background
- `w` a letter of the guess is **wrong** and not in the answer at all, wordle shows these with a grey background

For example, suppose your first guess is "event" and wordle displayed the following info for each letter (assume the real wordle answer is "depot"):
- `e` with a yellow background, this character is `misplaced`. It is in the answer but in the wrong position
- `v` with a grey background, this character is `wrong`
- `e` with a grey background, this character is `wrong` because there is only one 'e' in 'depot'
- `n` with a grey background, this character is also `wrong`
- `t` with a green background, this character is `correct`

The final correctness pattern is `mwwwe`

### Example
to run the solver you'll need to have installed Rust 1.59 and Cargo.
From the project's root directory run:

> cargo run --release

- go to the wordle web application and enter your first guess. i.e. "tares".
- enter your guess and correctness pattern into the solver:
> tares mwwwc

- the solver will recommend the next, "best", guess you should try
> Try... least
- type `least` into the wordle web application
- enter `least` and it's correctness pattern into the solver
- repeat until the correct word is guessed



## The Wordle Solver Algorithm
In general, the wordle solver implemented here uses the formula for
[Entropy of an information source](https://en.wikipedia.org/wiki/Information_theory#Entropy_of_an_information_source) to
compute the "best" word to guess. 
On any given turn, the best guess is the word that yields the highest amount of **information**, called "bits" in information theory.
To put it another way, which guess would reduce the space of possibilities the most (i.e. eliminate the most words).
The algorithm also needs to take into account a word's frequency data, or how common it is in everyday use. 
This is because not all five-letter words are equally possible in a game of wordle. 
Wordle will use more common words like: `jelly` or `shark` versus more obscure words like: `iller` or `jeely`.


So the final formula to compute if a word, `w`, is the best guess is: 

`BestWord = Pw * -Sum( Ppat * log2(Ppat) )`
- `Pw` is the probability of a word occurring in general, (based on its frequency count data)
- `Ppat` is the probability of a wordle correctness pattern occurring (that could still potentially match `w`)


After each guess, the algorithm removes any words and correctness patterns that could not possibly be a match based on all the guesses that
have been made so far. This pruning step boosts performance the most as you could potentially be reducing your search space in half.


### Data files used
`dictionary.txt` This file contains all the five-letter words used by wordle along with the "occurrence count" of that word.
This data is taken from Google Books' [Ngram Viewer](https://storage.googleapis.com/books/ngrams/books/datasetsv3.html).


`answers.txt` contains past worlds answers, in the order that wordle used them, with most recently used words at the top of the file.
