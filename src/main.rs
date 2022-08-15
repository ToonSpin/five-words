use clap::Parser;
use core::hash::{Hash, Hasher};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to a file with a list of words
    #[clap(short, long, value_parser)]
    input_file: std::path::PathBuf,
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
        let mut a = self.word.iter().peekable();
        let mut b = other.word.iter().peekable();

        loop {
            if let None = a.peek() {
                break;
            }
            if let None = b.peek() {
                break;
            }
            if a.peek() == b.peek() {
                return false;
            } else if a.peek() < b.peek() {
                a.next();
            } else {
                b.next();
            }
        }
        true
    }

    fn is_pairwise_disjoint(word_list: &Vec<Self>) -> bool {
        for a in 0..word_list.len() {
            for b in a + 1..word_list.len() {
                if !word_list[a].is_disjoint_with(&word_list[b]) {
                    return false;
                }
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

fn get_disjoint_indices(word_list: &Vec<Word>, sequence_length: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    for i in 0..word_list.len() {
        println!("{}/{}", i, word_list.len());
        result.append(&mut get_disjoint_indices_partial(
            &word_list,
            sequence_length,
            vec![],
            vec![i],
        ));
    }
    result
}

fn get_disjoint_indices_partial(
    word_list: &Vec<Word>,
    sequence_length: usize,
    mut partial: Vec<Vec<usize>>,
    mut state: Vec<usize>,
) -> Vec<Vec<usize>> {
    if state.len() == sequence_length {
        print!("Found:");
        for i in state.iter() {
            print!(" {}", word_list[*i].original_word);
        }
        println!("");
        partial.push(state);
        return partial;
    }

    let last_index = state.last().expect("state must not be empty");

    'next_index: for next_index in last_index + 1..word_list.len() {
        for index in state.iter() {
            if !word_list[*index].is_disjoint_with(&word_list[next_index]) {
                continue 'next_index;
            }
        }
        state.push(next_index);
        partial = get_disjoint_indices_partial(word_list, sequence_length, partial, state.clone());
        state.pop();
    }
    partial
}

fn get_words<T: Read>(mut input_reader: T) -> std::io::Result<Vec<Word>> {
    let mut word_set: HashSet<Word> = HashSet::new();
    let mut word_list = String::new();
    input_reader.read_to_string(&mut word_list)?;
    for line in word_list.lines().filter(|l| l.len() == 5) {
        let mut bytes = line.as_bytes().to_vec();
        bytes.sort();
        if all_characters_unique(&bytes) {
            let word = Word::new(bytes.clone().try_into().unwrap(), String::from(line));
            word_set.insert(word);
        }
    }
    Ok(word_set.into_iter().collect())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let input_file = File::open(args.input_file)?;
    let word_list = get_words(input_file)?;
    get_disjoint_indices(&word_list, 5);
    Ok(())
}
