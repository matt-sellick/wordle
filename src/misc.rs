use std::fs::File;
use std::path::Path;
use std::io::BufRead;

// functions written for reading words from file ... but I'd rather hard-code the words.
// left here so you can still see the logic you wrote!

fn read_file_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> // this is from Rust By Example for "reading lines"
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines()) //returns an iterator to the reader of the lines of the file
}

pub fn init_valid_guesses() -> Vec<String> {
    let mut valid_guesses: Vec<String> = Vec::new();
    match read_file_lines("./wordle_guesses.txt") {
        Ok(lines) => {
            for line in lines {
                if let Ok(word) = line {
                    valid_guesses.push(word.to_uppercase());
                }
            }
        },
        Err(error) => panic!("Error loading valid guesses: {}", error),
    }
    valid_guesses
}

pub fn get_secret() -> String {
    let mut secret_options: Vec<String> = Vec::new();
    
    /* 
        option for generating absolute path to the .txt, if you have it in the same directory as the executable:
        let mut path = std::env::current_exe().unwrap();
        path.push("wordle_answers.txt");
     */
    
    match read_file_lines("./wordle_answers.txt") {
        Ok(lines) => {
            for line in lines {
                if let Ok(word) = line {
                    secret_options.push(word);
                }
            }
        }
        Err(error) => panic!("Error loading secret words list: {}", error),
    }
    let rand_position = rand::thread_rng().gen_range(0..secret_options.len());
    let word = secret_options[rand_position].clone().to_uppercase();
    word
}

// trying to use termion to rewrite existing lines
pub fn print_to_screen(&self, turn: usize) {

    if turn == 0 { // "turn zero" print the board blank
        println!("---------------------");
        for _ in 1..=6 {
            println!("|   |   |   |   |   |\n---------------------");
        }
        self.keyboard.draw_keyboard();
    } else {
        // take last guess and scroll it across the row corresponding to the length of the guess list
        // looks like first letter of first row is 5 rows down and 3 columns over
        // then each letter is 4 columns later
        // then each new guess is two rows down

        // so the goto math is a letter is:
        // ROW: turn/guess.len() * 2 + 5
        // COL: letter position * 2 + 1 ... ish?

        let last_guess = self.guesses.last().unwrap().contents();
        let mut row = String::new();
        for (index, letter) in last_guess.char_indices() {
            row = format!("{row}{}{letter}", cursor::Goto(((index + 1) * 2 + 1) as u16, (self.guesses.len() * 4 + 1) as u16));
        }
        scroll(&row, 70);



    }
}

// old implementation, pre-TUI
pub fn draw(&self) {
    if self.turn == 0 {
        println!("\n     W O R D L E");
    }
    println!("\n---------------------");

    // for each guess row on the board ... future impl might not do this and would just redraw specific lines
    for guess in &self.guesses {

        let letter_colours = self.check_matches(guess);
        let to_print = self.format_row(guess, &letter_colours);

        // print/scroll row
        if guess.contents() == self.guesses.last().unwrap().contents() {
            scroll(&to_print, 50);
            scroll("\n---------------------\n", 20);
        } else {
            println!("{}", to_print);
            println!("---------------------");
        }
    }

    // print empty rows
    let total_rows = self.guesses.len() as u32;
    for _ in total_rows..6 {
        println!("|   |   |   |   |   |\n---------------------");
    }

    // print keyboard
    // println!("{}", self.keyboard.format());
}

// no longer using since you moved to TUI
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

// pub function version. moved to Board impl.
pub fn scroll(print: &str, dur: u64) {
    for item in print.chars() {
        print!("{item}");
        io::stdout().flush().expect("Flush should succeed");
        std::thread::sleep(Duration::from_millis(dur));
    }
}


pub fn update_keyboard(&mut self) { // formerly part of Board impl
    // logs/counts last guessed letters so you can hide the letters
    for letters in self.guesses.last().unwrap().contents().chars() {
        self.keyboard.guessed_letters.entry(letters).and_modify(|count| *count += 1).or_insert(1);
    }
}
