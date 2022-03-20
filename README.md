# Rust Wordle Solver
Uses information theory to solve a wordle challenge. To do this, it uses the formula for
[Entropy of an information source](https://en.wikipedia.org/wiki/Information_theory#Entropy_of_an_information_source).
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

### Benchmarking tools
- [hyperfine](https://crates.io/crates/hyperfine) to benchmark the solver from the command line
- [cargo flamegraph]() for performance analysis 