// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::fs::read;
use std::hash::Hash;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn chars_to_string(chars: &Vec<char>) -> String {
    chars.into_iter().collect()
}

struct CharInWord {
    num: u32,
    guessed_num: u32,
    idxs: Vec<u32>,
}

fn word_statistics(secret_word_chars: &Vec<char>) -> HashMap<char, CharInWord> {
    let mut statistics: HashMap<char, CharInWord> = HashMap::new();
    for(idx, ch) in secret_word_chars.iter().enumerate() {
        let a = statistics.entry(*ch).or_insert(CharInWord {
            num: 0,
            guessed_num: 0,
            idxs: Vec::new(),
        });
        (*a).num += 1;
        (*a).idxs.push(idx.try_into().unwrap());
    }
    statistics
}

enum Index {
    Yes(u32),
    No,
}

fn is_char_in_word(ch:char, statistics: &mut HashMap<char, CharInWord>) -> Index {
    match statistics.get_mut(&ch) {
        Some(ciw) => {
            let ch_idx: u32;
            match ciw.idxs.get(ciw.guessed_num as usize) {
                Some(x) => ch_idx = *x,
                None => return Index::No,
            }
            ciw.guessed_num += 1;
            Index::Yes(ch_idx)
        }
        None => Index::No,
    }
}

fn read_guess_char() -> char {
    print!("Please guess a letter:");
    //Read from the stdin 
    io::stdout().flush().expect("Error flushing stdout");
    let mut guess = String::new();
    io::stdin().read_line(&mut guess).expect("Error reading line ");

    guess.as_bytes()[0] as char
}

fn hangman(secret_word_chars: &Vec<char>) {
    let mut pass = false;
    let mut num_incorrect_guesses = 0;
    let mut cur_word_chars = vec!['-'; secret_word_chars.len()];
    let mut statistics = word_statistics(secret_word_chars);
    let mut cur_guessed_chars = Vec::new();
    while num_incorrect_guesses < 5 {
        println!("The word so far is {}", chars_to_string(&cur_word_chars));
        println!("You have guessed the following letters: {}", chars_to_string(&cur_guessed_chars));
        println!("You have {} guesses left", NUM_INCORRECT_GUESSES - num_incorrect_guesses);
        let ch = read_guess_char();
        if ch.is_alphabetic() {
            match is_char_in_word(ch, &mut statistics) {
                Index::Yes(idx) => {
                    cur_word_chars[idx as usize] = ch;
                    if cur_word_chars == *secret_word_chars {
                        pass =true;
                        break;
                    }
                }
                Index::No => {
                    num_incorrect_guesses += 1;
                    println!("Sorry, that letter is not in the word");
                }
            }
            cur_guessed_chars.push(ch);
        }
        else {
            panic!("char is not alphabetic");
        }
        println!("--------------Round ends------------");
    }
    if pass {
        println!("Congratulations you guessed the secret word: {}!", chars_to_string(&cur_word_chars));
    }
    else {
        println!("Sorry, you run out of guesses!");
    }

}

fn main() {
    let secret_word = pick_a_random_word();
    //let secret_word = "ferret";
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)
    hangman(&secret_word_chars);

}
