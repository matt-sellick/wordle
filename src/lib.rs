use std::io::{self, Write};
use colored::Colorize;
use std::collections::HashMap;
use std::time::Duration;

// five-letter word
#[derive(Debug)]
pub struct Word {
    contents: String, // keep it private and ensure "Words" can only be created if they're valid
}

impl Word {
    pub fn try_new(word: String, valid_options: &Vec<String>) -> Result<Word, &'static str> {

        // check the word is 5 alphabetic characters, then make them uppercase
        if word.chars().count() != 5 {
            return Err("Please choose a 5-letter word.");
        }
        if !word.chars().all(|c| c.is_alphabetic()) {
            return Err("Please choose a real word.");
        }
        let word = word.to_uppercase();

        // check it is a legal word
        let mut found: bool = false;
        for opt in valid_options.iter() {
            if *opt == word {
                found = true;
                break;
            }
        }
        if !found {
            return Err("Not in word list.");
        }
        Ok(Word{contents: word})
    }

    pub fn contents(&self) -> &String { // getter
        &self.contents
    }
}

// represents letter colours, for use in array
#[derive(Copy, Clone)]
enum Letter {
    Green,
    Yellow,
    Grey,
}

pub struct Board {
    pub guesses: Vec<Word>,
    pub secret_word: Word,
}

impl Board {
    pub fn new(secret_word: Word) -> Board {
        Board {
            guesses: vec![],
            secret_word,
        }
    }

    pub fn check_guess(&self) -> bool {
        if let Some(guess) = self.guesses.last() { // return true if most recent guess matches the secret word
            return guess.contents() == self.secret_word.contents();
        } else {
            return false
        }
    }
    
    pub fn draw(&self, turn: usize) {
        
        println!("\n---------------------");
        let mut guess_counter: usize = 0;

        // for each guess row on the board ...
        for guess in &self.guesses {

            // init letter tally and colour trackers
            let mut match_tracker: HashMap<char, usize> = HashMap::new();
            let mut row_colours: [Letter; 5] = [Letter::Grey; 5]; // default them all to Grey
            guess_counter += 1;

            // so you can check the secret word and guesses by indexing directly ...
            let mut secret_array: [char; 5] = ['a'; 5];
            for (i, c) in self.secret_word.contents().char_indices() {
                secret_array[i] = c;
            }
            let mut guess_array: [char; 5] = ['a'; 5];
            for (i, c) in guess.contents().char_indices() {
                guess_array[i] = c;
            }

            // check GREEN matches (same-index matches)
            for (idx_secret, secret_letter) in self.secret_word.contents().char_indices() {
                if secret_letter == guess_array[idx_secret] {
                    row_colours[idx_secret] = Letter::Green;
                    let count = match_tracker.entry(secret_letter).or_insert(0);
                    *count += 1; // increase that letter's count in the map
                }
            }

            // check YELLOW matches (a secret word's letter exists in guess word and is still GREY)
            // AND the amount of that letter in the secret word is MORE than the number that have been logged in the map already
            for secret_letter in self.secret_word.contents().chars() {
                for (idx_guess, guess_letter) in guess.contents().char_indices() {
                    if guess_letter == secret_letter
                    && self.secret_word.contents().chars().filter(|x| x == &secret_letter).count() > *match_tracker.get(&secret_letter).unwrap_or_else(|| &0) {
                        if let Letter::Grey = row_colours[idx_guess] {
                            row_colours[idx_guess] = Letter::Yellow;
                            let count = match_tracker.entry(secret_letter).or_insert(0);
                            *count += 1;
                        }
                    }
                }
            }

            // format the guess row ...
            let mut row_print: String = String::new();
            for (idx, letter) in guess.contents().char_indices() {
                row_print = format!("{}| ", row_print);
                match row_colours[idx] {
                    Letter::Green => row_print = format!("{}{} ", row_print, String::from(letter).bright_green()),
                    Letter::Yellow => row_print = format!("{}{} ", row_print, String::from(letter).bright_yellow()),
                    Letter::Grey => row_print = format!("{}{} ", row_print, String::from(letter)),
                }
            }
            row_print = format!("{}|", row_print);

            // print/scroll row
            if turn == guess_counter {
                scroll(&row_print, 50);
                scroll("\n---------------------\n", 20);
            } else {
                println!("{}", row_print);
                println!("---------------------");
            }
        }

        // print empty rows
        let total_rows = self.guesses.len() as u32;
        for _ in total_rows..6 {
            println!("|   |   |   |   |   |\n---------------------");
        }       
    }

    pub fn win_message(&self, turns: usize) {
        match turns {
            1 => scroll("\n       Genius\n", 70),
            2 => scroll("\n     Magnificent\n", 70),
            3 => scroll("\n     Impressive\n", 70),
            4 => scroll("\n      Splendid\n", 70),
            5 => scroll("\n       Great\n", 70),
            6 => scroll("\n        Phew\n", 70),
            7 => {
                scroll("\n       Failure:\n", 70);
                println!("        {}", self.secret_word.contents());
            },
            _ => (),
        }
        println!("");
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn get_input() -> String {
    // returns an input string. If there's an input error, will reject and try again.
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_) => break,
            Err(error) => {
                println!("Error: {}, try again", error);
                continue;
            },
        };
    }
    let word: String = input.trim().to_string(); // trim whitespace
    word
}

pub fn scroll(print: &str, dur: u64) {
    for item in print.chars() {
        print!("{item}");
        io::stdout().flush().expect("Flush should succeed");
        std::thread::sleep(Duration::from_millis(dur));
    }
}