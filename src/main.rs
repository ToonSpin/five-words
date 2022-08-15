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

fn get_disjoint_indices(word_list: &Vec<Word>, sequence_length: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    for i in 0..word_list.len() {
        println!("{}/{}", i, word_list.len());
        result.append(&mut get_disjoint_indices_partial(
            &word_list,
            sequence_length,
            vec![],
            vec![i],
            &(0..word_list.len()).collect()
        ));
    }
    result
}

fn get_disjoint_indices_partial(
    word_list: &Vec<Word>,
    sequence_length: usize,
    mut partial: Vec<Vec<usize>>,
    mut state: Vec<usize>,
    valid_indices: &Vec<usize>
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

    let last_index = *state.last().expect("state must not be empty");
    let new_valid_indices: Vec<usize> = valid_indices.iter().filter(|&i| word_list[last_index].is_disjoint_with(&word_list[*i])).cloned().collect();

    // if state.len() < 3 {
    //     for _i in 0..state.len() {
    //         print!("  ")
    //     }
    //     println!("length of new_valid_indices: {:?}", new_valid_indices.len());
    // }

    for next_index in new_valid_indices.iter().filter(|&i| *i >= last_index) {
        state.push(*next_index);
        partial = get_disjoint_indices_partial(word_list, sequence_length, partial, state.clone(), &new_valid_indices);
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
