# Rust Wordle Solver
A command line driven world solver that uses information theory to solve a wordle challenge. 
It's inspired [3Blue1Brown](https://www.youtube.com/watch?v=v68zYyaEmEA) and
[Jon Gjenset's](https://www.youtube.com/watch?v=doFowk4xj7Q) youtube videos on this topic.

## Running
This solver is a separate command line program that is used at the same time you are making guesses into the
[Wordle web application](https://www.nytimes.com/games/wordle/index.html).
You make your first guess into the web app, and then enter that guess along with its corrsponding "correctness" pattern
into the solver, separated by a space. 

The correctness pattern is a five character string that represents the... correctness of each letter of your guess as reported by wordle.
The correctness string is composed of the characters `c`,`m`, or `w`.
- `c` a letter of the guess is in the **correct** position, the wordle web app shows these with a green background
- `m` a letter of the guess is in the answer but is **misplaced**, or in the incorrect position, wordle shows these with a yellow background
- `w` a letter of the guess is **wrong** and not in the answer at all, wordle shows these with a grey background

For example, suppose your first guess is "event" and wordle displayed the following info for each letter (assume the real wordle answer is "depot"):
- `e` with a yellow background, this character is `misplaced`. It is in the answer but in the wrong position
- `v` with a grey background, this character is `wrong`
- `e` with a grey background, this character is `wrong` becuase there is only one 'e' in depot
- `n` with a grey backgroung, this character is `wrong` as well
- `t` with a green background, this character is `correct`

Therefore your correctness pattern is `mwwwe`

### Example
to run the solver
> cargo run --release

- go to the wordle web application and enter your first guess. i.e. "event".
- enter your guess and correctness pattern into the solver:
> event mwwwc

- the solver will return the next, "best", guess you should try
> Try... least
- type `least` into the wordle web application
- enter `least` and it's pattern into the solver
- repeast until the correct word is guessed





## Worldle Solver Algorithm
In general, the wordle solver implemented here uses the formula for
[Entropy of an information source](https://en.wikipedia.org/wiki/Information_theory#Entropy_of_an_information_source) to
compute the "best" word to guess.
This formula lets you compute the quality of a given guess, expressed in "bits".
The solver will select a word that returns the greatest amount "bits" during each turn, until the mystery word 
is guessed correctly.


### Data files used
- `dictionary.txt` This file contains all the five-letter words used by wordle, merged with
all five-letter found in books that where scanned by Google Books. Then each of those words is followed by a count of 
the number of times that word has appeared across all books (as scanned by google). 
This count data is provided by Google Books' [Ngram Viewer](https://storage.googleapis.com/books/ngrams/books/datasetsv3.html)

- `answers.txt` all five-letter words that wordle will accept as valid guesses, in the order that wordle used them,
with most recently used words at the top of the file.
