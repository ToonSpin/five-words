use clap::Parser;
use core::hash::{Hash, Hasher};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to a file with a list of words
    #[clap(short, long, value_parser)]
    input_file: std::path::PathBuf,

    /// Show a progress indicator on standard error
    #[clap(short, long, action)]
    progress: bool,

    /// Add extra output to standard error
    #[clap(short, long, action)]
    verbose: bool,
}

#[derive(Debug)]
struct Word {
    word: [u8; 5],
    original_word: String,
}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
    }
}

impl PartialEq for Word {
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

    fn is_disjoint_with(self: &Self, other: &Self) -> bool {
        let mut a = 0;
        let mut b = 0;

        loop {
            if a == 5 || b == 5 {
                break;
            }
            if self.word[a] == other.word[b] {
                return false;
            } else if self.word[a] < other.word[b] {
                a = a + 1;
            } else {
                b = b + 1;
            }
        }
        true
    }
}

fn all_characters_unique(word: &[u8]) -> bool {
    for i in 1..word.len() {
        if word[i - 1] == word[i] {
            return false;
        }
    }
    return true;
}

fn get_disjoint_indices(
    word_list: &Vec<Word>,
    sequence_length: usize,
    args: &Args,
) -> Vec<Vec<usize>> {
    let bar = if args.verbose || !args.progress {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(word_list.len().try_into().unwrap())
    };
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{elapsed_precise} {wide_bar} {percent}%")
            .unwrap(),
    );
    let mut result = Vec::new();
    for i in 0..word_list.len() {
        bar.inc(1);
        if args.verbose && args.progress {
            eprint!("{}/{}\r", i, word_list.len());
        }
        result.append(&mut get_disjoint_indices_partial(
            &word_list,
            sequence_length,
            args.verbose,
            vec![],
            vec![i],
            &(0..word_list.len()).collect(),
        ));
    }
    if args.verbose && args.progress {
        eprintln!("");
    }
    bar.finish();
    result
}

fn get_disjoint_indices_partial(
    word_list: &Vec<Word>,
    sequence_length: usize,
    verbose: bool,
    mut partial: Vec<Vec<usize>>,
    mut state: Vec<usize>,
    valid_indices: &Vec<usize>,
) -> Vec<Vec<usize>> {
    if state.len() == sequence_length {
        if verbose {
            eprint!("Found:");
            for i in state.iter() {
                eprint!(" {}", word_list[*i].original_word);
            }
            eprintln!("");
        }
        partial.push(state);
        return partial;
    }

    let last_index = *state.last().expect("state must not be empty");
    let new_valid_indices: Vec<usize> = valid_indices
        .iter()
        .filter(|&i| word_list[last_index].is_disjoint_with(&word_list[*i]))
        .cloned()
        .collect();

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
    let mut word_list = String::new();
    input_reader.read_to_string(&mut word_list)?;
    for line in word_list.lines().filter(|l| l.len() == 5) {
        let mut bytes = line.as_bytes().to_vec();
        bytes.sort();
        if all_characters_unique(&bytes) {
            let word = Word::new(bytes.clone().try_into().unwrap(), String::from(line));
            if word_set.contains(&word) {
                let existing = word_set.get(&word).unwrap();
                if args.verbose {
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
    }
    Ok(word_set.into_iter().collect())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let input_file = File::open(args.input_file.clone())?;
    let word_list = get_words(input_file, &args)?;
    for sequence in get_disjoint_indices(&word_list, 5, &args).iter() {
        for i in 0..5 {
            if i > 0 {
                print!("\t");
            }
            print!("{}", word_list[sequence[i]].original_word);
        }
        println!("");
    }
    Ok(())
}
