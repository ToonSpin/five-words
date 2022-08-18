use clap::Parser;
use core::hash::{Hash, Hasher};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// This program reads a list of lowercase ASCII words, and produces a list of
/// tab-separated combinations of words that don't have any characters in
/// common. Any anagrams of words in the list are not considered. The word list
/// is read from standard input, or it can be specified with the -i option. It
/// is inspired by this video: https://www.youtube.com/watch?v=_-AfhLQfb6w
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to a file with a list of words
    #[clap(short, long, value_parser)]
    input_file: Option<std::path::PathBuf>,

    /// Show a progress indicator on standard error
    #[clap(short, long, action, conflicts_with = "verbose")]
    progress: bool,

    /// Add extra output to standard error, can't be used with a progress bar
    #[clap(short, long, action, conflicts_with = "progress")]
    verbose: bool,
}

struct Word {
    word: [u8; 5],
    original_word: String,
}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Only consider the sorted bytes when hashing the Word, because we're
        // also storing the original word. It's not desirable for that to be
        // part of the hash, because otherwise we would be storing all the
        // anagrams of this Word in the set, too.
        self.word.hash(state);
    }
}

impl PartialEq for Word {
    // Only consider the sorted bytes when comparing Words, because it's
    // desireable for anagrams of the original words to be equal, not
    // different.
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word
    }
}

impl Eq for Word {}

impl Word {
    fn new(word: [u8; 5], original_word: String) -> Self {
        Word {
            word,
            original_word,
        }
    }

    /// Returns `true` if the two `Word`s do not have any characters in common.
    /// This function assumes that `word` is sorted for both `Word`s.
    #[allow(clippy::comparison_chain)]
    fn is_disjoint_with(&self, other: &Self) -> bool {
        let mut a = 0;
        let mut b = 0;

        loop {
            if a == 5 || b == 5 {
                break;
            }
            if self.word[a] == other.word[b] {
                return false;
            } else if self.word[a] < other.word[b] {
                a += 1;
            } else {
                b += 1;
            }
        }
        true
    }
}

/// Returns `true` if the array has no duplicate values. This function assumes
/// that `word` is sorted.
fn all_characters_unique(word: &[u8]) -> bool {
    for i in 1..word.len() {
        if word[i - 1] == word[i] {
            return false;
        }
    }
    true
}

fn get_disjoint_indices(
    word_list: &Vec<Word>,
    sequence_length: usize,
    args: &Args,
) -> Vec<Vec<usize>> {
    let word_list_len = word_list.len();
    let bar = if args.progress {
        ProgressBar::new(word_list_len.try_into().unwrap()).with_style(
            ProgressStyle::default_bar()
                .template("{elapsed_precise} {wide_bar} {percent}%")
                .unwrap(),
        )
    } else {
        ProgressBar::hidden()
    };

    let result = (0..word_list_len)
        .into_par_iter()
        .progress_with(bar)
        .map(|i| {
            get_disjoint_indices_partial(
                word_list,
                sequence_length,
                args.verbose,
                vec![],
                vec![i],
                &(0..word_list_len).collect(),
            )
        });

    result.flatten().collect()
}

#[allow(clippy::ptr_arg)]
fn get_disjoint_indices_partial(
    word_list: &Vec<Word>,
    sequence_length: usize,
    verbose: bool,
    mut partial: Vec<Vec<usize>>,
    mut state: Vec<usize>,
    valid_indices: &Vec<usize>,
) -> Vec<Vec<usize>> {
    // Found a match. Further down this function, all the combinations of words
    // that are not disjoint are filtered out. This means that non-disjoint
    // combinations are not considered at all, so if the state has the desired
    // length then it is guaranteed to be pairwise disjoint.
    if state.len() == sequence_length {
        if verbose {
            eprint!("Found:");
            for i in state.iter() {
                eprint!(" {}", word_list[*i].original_word);
            }
            eprintln!();
        }
        partial.push(state);
        return partial;
    }

    // First, prune all words in the valid indices that are disjoint with the
    // last word in the state. This is done here so the calling function
    // get_disjoint_indices doesn't have to do it.
    let last_index = *state.last().expect("state must not be empty");
    let new_valid_indices: Vec<usize> = valid_indices
        .iter()
        .filter(|&i| word_list[last_index].is_disjoint_with(&word_list[*i]))
        .cloned()
        .collect();

    // The filter is here because otherwise there would be duplicate results.
    for next_index in new_valid_indices.iter().filter(|&i| *i >= last_index) {
        state.push(*next_index);
        partial = get_disjoint_indices_partial(
            word_list,
            sequence_length,
            verbose,
            partial,
            state.clone(),
            &new_valid_indices,
        );
        state.pop();
    }

    partial
}

fn get_words<T: Read>(mut input_reader: T, args: &Args) -> std::io::Result<Vec<Word>> {
    let mut word_set: HashSet<Word> = HashSet::new();
    let mut input = String::new();

    input_reader.read_to_string(&mut input)?;

    for line in input.lines().filter(|l| l.len() == 5) {
        let mut bytes = line.as_bytes().to_vec();
        bytes.sort();

        if !all_characters_unique(&bytes) {
            continue;
        }

        let word = Word::new(bytes.clone().try_into().unwrap(), String::from(line));

        // This check is not strictly necessary to insert the Word, but it's
        // here because of the verbose output, to debug the anagram logic.
        if word_set.contains(&word) {
            if args.verbose {
                let existing = word_set.get(&word).unwrap();
                eprintln!(
                    "An anagram of the word {} is already in the list ({}).",
                    word.original_word, existing.original_word
                );
            }
        } else {
            if args.verbose {
                eprintln!("Adding the word {} to the list.", word.original_word);
            }
            word_set.insert(word);
        }
    }
    Ok(word_set.into_iter().collect())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let word_list = if args.input_file.is_none() || args.input_file == Some(PathBuf::from("-")) {
        get_words(std::io::stdin(), &args)?
    } else {
        let input_file = File::open(args.input_file.as_ref().unwrap().clone())?;
        get_words(input_file, &args)?
    };

    for sequence in get_disjoint_indices(&word_list, 5, &args).iter() {
        for i in 0..5 {
            if i > 0 {
                print!("\t");
            }
            print!("{}", word_list[sequence[i]].original_word);
        }
        println!();
    }
    Ok(())
}
