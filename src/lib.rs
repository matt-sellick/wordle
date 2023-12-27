use std::io::{self, BufRead, Write, Stdout, stdout, stdin};
use std::collections::HashMap;
use std::time::Duration;
use std::path::Path;
use std::fs::{File, OpenOptions};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};
use termion::cursor::{self, DetectCursorPos};
use termion::{clear, color};

use colored::Colorize;

// five-letter word
#[derive(Debug)]
pub struct Word {
    contents: String, // keep it private and ensure "Words" can only be created if they're valid
}

impl Word {
    pub fn try_new(word: String, valid_options: &Vec<String>) -> Result<Word, &'static str> {

        // check the word is 5 alphabetic characters, then make them uppercase
        if word.chars().count() != 5 {
            return Err("Please choose a 5-letter word");
        }
        if !word.chars().all(|c| c.is_alphabetic()) {
            return Err("Please choose a real word");
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
            return Err("Not in word list");
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
    pub hard: bool,
    pub turn: usize, // what turn is it? (turn zero is for board setup)
    pub guesses: Vec<Word>, // all words that have been guessed
    pub secret_word: Word,
    keyboard: Keyboard, // holds info about what letters have been guessed
    screen: RawTerminal<AlternateScreen<Stdout>>, // go into alternate screen in raw mode when board is constructed
    coord: (u16, u16), // column, row of board top left corner (where W O R D L E is printed) and column of left board border
    pub win: bool, // did you win?
}

impl Board {
    pub fn new(secret_word: Word) -> Board {

        // figure out where to print the board on screen
        let (mut col, mut row) = termion::terminal_size().unwrap();
        match (col / 2).checked_sub(12) { // ensures no overflow if terminal is small
            Some(n) => col = n,
            None => col = 0,
        }
        match (row / 2).checked_sub(9) { // can change the "checked sub" arguments as a master "slider" for board position
            Some(n) => row = n,
            None => row = 0,
        }

        Board {
            hard: false,
            turn: 0,
            guesses: vec![],
            secret_word,
            keyboard: Keyboard::initialize(),
            screen: stdout().into_alternate_screen().unwrap().into_raw_mode().unwrap(),
            coord: (col, row),
            win: false,
        }
    }

    pub fn check_guess(&self) -> bool {
        if let Some(guess) = self.guesses.last() { // return true if most recent guess matches the secret word
            return guess.contents() == self.secret_word.contents();
        } else {
            return false
        }
    }

    fn check_matches(&self, guess: &Word) -> [Letter; 5] { // currently allows checking against guess you specify, not just most recent
        let mut match_counter: HashMap<char, usize> = HashMap::new();
        let mut letter_colours: [Letter; 5] = [Letter::Grey; 5];
        let mut secret_word: [char; 5] = ['_'; 5];
        let mut guess_word: [char; 5] = ['_'; 5];
        for (index, guess_letter) in guess.contents().char_indices() {
            guess_word[index] = guess_letter;
        }

        // check GREEN matches (same-index matches)
        for (index, secret_letter) in self.secret_word.contents().char_indices() {
            secret_word[index] = secret_letter;
            if secret_letter == guess_word[index] {
                letter_colours[index] = Letter::Green;
                match_counter.entry(secret_letter).and_modify(|count| *count += 1).or_insert(1);
            }
        }
        
        // check YELLOW matches (a secret word's letter exists in guess word and is still GREY)
        // AND the amount of that letter in the secret word is MORE than the number that have been logged in the map already
        for secret_letter in self.secret_word.contents().chars() {
            for (index, guess_letter) in guess.contents().char_indices() {
                if guess_letter == secret_letter
                && self.secret_word.contents().chars().filter(|s| s == &secret_letter).count() > *match_counter.get(&secret_letter).unwrap_or_else(|| &0) {
                    if let Letter::Grey = letter_colours[index] {
                        letter_colours[index] = Letter::Yellow;
                        match_counter.entry(secret_letter).and_modify(|count| *count += 1).or_insert(1);
                    }
                }
            }
        }
        letter_colours
    }

    fn format(&mut self, colours: &[Letter; 5]) -> String {

        // figures out what colours to display for the board and keyboard elements, but does not actually print to screen
        // returns a formatted String from letter colours array and also updates the keyboard colours

        let guess = self.guesses.last().unwrap(); // safe because not calling until a guess has been made
        let mut to_print = String::new();
        for (index, letter) in guess.contents().char_indices() {
            to_print = format!("{to_print}| ");
            match colours[index] {
                Letter::Green => {
                    to_print = format!("{}{} ", to_print, String::from(letter).bright_green());
                    self.keyboard.guessed_letters.insert(letter, Letter::Green); // updating keyboard colours
                },
                Letter::Yellow => {
                    to_print = format!("{}{} ", to_print, String::from(letter).bright_yellow());
                    // if the letter is not already in the keyboard as yellow or green
                    if let &Letter::Grey = self.keyboard.guessed_letters.get(&letter).unwrap_or(&Letter::Grey) {
                        self.keyboard.guessed_letters.insert(letter, Letter::Yellow);    
                    }
                },
                Letter::Grey => {
                    to_print = format!("{}{} ", to_print, String::from(letter).truecolor(10, 10, 10));
                    // if the letter is not already in the keyboard as yellow or green
                    if let &Letter::Grey = self.keyboard.guessed_letters.get(&letter).unwrap_or(&Letter::Grey) {
                        self.keyboard.guessed_letters.insert(letter, Letter::Grey);    
                    }
                },
            }
        }

        format!("{}|", to_print)
    }
    
    pub fn draw(&mut self) {
        if self.turn == 0 { // "turn zero" prints the board blank, centred

            let (col, row) = self.coord; // Goto() uses col/row

            // print game title
            write!(self.screen, "{}{}W O R D L E",
                termion::clear::All,
                cursor::Goto(col + 5, row)
            ).unwrap();

            // print board "frame"
            let board_top = row + 2; // row of top of board
            for offset in 0..=5 {
                write!(self.screen, "{}---------------------{}|   |   |   |   |   |",
                    cursor::Goto(col, board_top + offset * 2),
                    cursor::Goto(col, board_top + offset * 2 + 1)
                ).unwrap();
            }
            write!(self.screen, "{}---------------------",
                cursor::Goto(col, board_top + 12)
            ).unwrap();

            // print full keyboard
            let keyboard_top = row + 17; // row of top of keyboard
            write!(self.screen, "{}", self.keyboard.format((col, keyboard_top))).unwrap();

            // flush screen buffer
            self.screen.flush().unwrap();

        } else { // turns 1-6

            let (col, row) = self.coord;

            // check matches and format the letter colours to print
            let last_guess = self.guesses.last().unwrap(); // unwrap is safe here
            let letter_colours = self.check_matches(last_guess);
            let to_print = self.format(&letter_colours);

            // update keyboard display
            let keyboard_top = row + 17;
            write!(self.screen, "{}", self.keyboard.format((col, keyboard_top))).unwrap();

            // move cursor to appropriate board row top prep for scrolling coloured guess
            let guess_row = row + 1;
            write!(self.screen, "{}{}",
                cursor::Goto(col, guess_row + self.turn as u16 * 2), // go to start of turn row
                cursor::Show
            ).unwrap();

            // flush screen buffer
            self.screen.flush().unwrap();

            // scroll print the word
            self.scroll(&to_print, 15);
        }
    }

    pub fn get_input(&mut self) -> String {
        let (col, row) = self.coord; // to locate initial position. Shadowed later inside input loop
        let row = row + 2;
        let mut word = String::new(); // buffer for user entry

        // move cursor to appropriate board row
        write!(self.screen, "{}|   |   |   |   |   |{}{}",
            cursor::Goto(col, row + self.turn as u16 * 2 - 1), // go to turn row, reprint blanks in case of failed guess
            cursor::Goto(col + 2, row + self.turn as u16 * 2 - 1), // go to start of turn row's letters
            cursor::Show
        ).unwrap();
        self.screen.flush().unwrap();

        // user inputs guess, letters will appear on the board
        let input = stdin();
        for key in input.keys() {
            match key.unwrap() {
                Key::Char('`') => panic!("exiting program"), // for debugging
                Key::Char('1') => { // enable hard mode
                    self.print_msg(&format!("\r{}", termion::clear::CurrentLine));
                    if self.guesses.is_empty() { // only if you haven't guessed yet
                        self.hard = true;
                        self.print_msg("Hard mode enabled");
                    } else {
                        self.print_msg("Cannot enable hard mode"); // actual message is "Hard mode can only be enabled at the start of a round" but that's long and could make terminal panic
                    }
                },
                Key::Char('\n') => {
                    break; // pressing enter breaks and returns the word String to main()
                },
                Key::Char(ch) => {
                    if ch.is_alphabetic() && word.len() < 5 { // only enters up to 5 letters
                        let (cursor_col, cursor_row) = self.screen.cursor_pos().unwrap();
                        write!(self.screen, "{}{}",
                            ch.to_uppercase(),
                            cursor::Goto(cursor_col + 4, cursor_row)
                        ).unwrap();
                        word.push(ch);
                    }
                    if word.len() >= 5 {
                        write!(self.screen, "{}",
                            cursor::Hide
                        ).unwrap();
                    }
                    self.screen.flush().unwrap();
                    self.print_msg(&format!("\r{}", termion::clear::CurrentLine)); // clear any errors displayed after first keypress
                        // This gets called every time you press a key, which is unnecessary but works fine and not sure how else to do
                },
                Key::Backspace => {
                    if !word.is_empty() {
                        let (cursor_col, cursor_row) = self.screen.cursor_pos().unwrap();
                        write!(self.screen, "{} {}", // moves back, overwrites with space, then moves back again
                            cursor::Goto(cursor_col - 4, cursor_row),
                            cursor::Goto(cursor_col - 4, cursor_row),
                        ).unwrap();
                        word.pop();
                        if word.len() < 5 {
                            write!(self.screen, "{}", cursor::Show).unwrap();
                        }
                        self.screen.flush().unwrap();
                    }
                },
                _ => (),
            }
        }
        word
    }

    pub fn hard_check(&self, guess: &Word) -> (bool, Vec<char>) {
        // returns false, and the violating letter(s) (for display) if the word doesn't pass the hard mode check
        // word must be passed in because you want it to fail *before* entering the board's guess list

        let mut pass: bool = true;
        let mut violations: Vec<char> = Vec::new();

        // check passed-in word against successful hits (i.e. in keyboard)
        // make sure that the guess word contains all the "revealed" entries in the hashmap
        let mut revealed: HashMap<char, Letter> = self.keyboard.guessed_letters.clone();
        revealed.retain(|_, l| { // this will trim down revealed to contain only revealed letters, not greys
            match l {
                Letter::Green => true,
                Letter::Yellow => true,
                Letter::Grey => false,
            }
        });

        // if a revealed letter is not found in the guess, will return a test "failure" and the offending letter
        for letters in revealed.keys() {
            if !guess.contents().contains(*letters) {
                pass = false;
                violations.push(*letters);
            }
        }
        
        (pass, violations)
    }

    pub fn scroll(&mut self, print: &str, duration: u64) {
        for item in print.chars() {
            write!(self.screen, "{item}").unwrap();
            self.screen.flush().unwrap();
            std::thread::sleep(Duration::from_millis(duration));
        }
    }

    pub fn win_message(&mut self) {
        let mut message = String::new();
        if self.win {
            match self.turn {
                1 => message.push_str("Genius"),
                2 => message.push_str("Magnificent"),
                3 => message.push_str("Impressive"),
                4 => message.push_str("Splendid"),
                5 => message.push_str("Great"),
                6 => message.push_str("Phew"),
                _ => (),
            }
        } else {
            message = format!("Failure: {}", self.secret_word.contents());
        }

        // print win message under the grid, above the keyboard (same row as error messages)
        let (col, row) = self.coord;
        let message_row = row + 16;
        write!(self.screen, "{}{}",
            cursor::Hide,
            cursor::Goto(col + 10 - (message.len() as u16 / 2), message_row)
        ).unwrap();
        self.screen.flush().unwrap();
        self.scroll(&message, 70);
        std::thread::sleep(Duration::from_secs(2)); // wait a couple seconds

        // "press any key to continue"
        let exit_message = "Press any key to continue";
        let press_message_row = row + 22;
        write!(self.screen,
            "{}{}",
            cursor::Goto(col + 10 - (exit_message.len() as u16 / 2), press_message_row), // this ensures the text is centred
            exit_message
        ).unwrap();
        self.screen.flush().unwrap();

        // wait for key press
        press_to_continue();
    }

    pub fn print_msg(&mut self, error: &str) { // print errors centred under the board but restores cursor after
        let (col, row) = self.coord;
        let message_row = row + 16;
        let (return_col, return_row) = self.screen.cursor_pos().unwrap(); // cursor position before jumping
        write!(self.screen, "{}{}{}",
            cursor::Goto(col + 10 - (error.len() as u16 / 2), message_row),
            error,
            cursor::Goto(return_col, return_row),
            // note that zsh doesn't like cursor Save/Hide so needed to use Goto()
        ).unwrap();
        self.screen.flush().unwrap();
    }

    pub fn stats(&mut self) {
        /*
            the stats vector indices represent:
            0: 1s
            1: 2s
            2: 3s
            3: 4s
            4: 5s
            5: 6s
            6: failures
            7: current streak
            8: max streak

                1           100         1           1
                Played      Win %       Current     Max
                                        Streak      Streak
            | 1 | 0
            | 2 | 0
            | 3 ||||||||||||||||||||||||||||||| 15              -> this is 30 ticks (n/i first one)
            | 4 ||||||||||||||||||||||||||||||||||||||||| 20    -> this is 40 ticks
            | 5 ||||||||||| 5
            | 6 | 1

            stats graph is 48 across
        */

        let filename = "./wordle_stats.txt";
        let mut stats: Vec<u16> = Vec::new(); // to hold nine numbers representing stats
        if let Ok(lines) = read_file_lines(filename) { // will attempt to read a file but do nothing if the file does not exist
            for line in lines {
                if let Ok(value) = line {
                    if let Ok(number) = value.parse::<u16>() {
                        stats.push(number); // push the lines onto the vector as long as each one is a number
                        if stats.len() >= 9 { // and only until there are nine
                            break;
                        }
                    }
                }
            }
        }

        // check the vector is valid, and init to nine zeros if it is not
        if stats.len() != 9 {
            stats.clear();
            for _ in 1..=9 {
                stats.push(0);
            }
        }

        // update stats
        if self.win { // if you won
            if let Some(count) = stats.get_mut(self.turn - 1) {
                *count += 1; // increase wins associated with turn number
            }
            if let Some(n) = stats.get_mut(7) {
                *n += 1; // streak +1
            }
        } else { // if you failed
            if let Some(count) = stats.get_mut(6) {
                *count += 1; // failure count
            }
            if let Some(n) = stats.get_mut(7) {
                *n = 0; // reset streak
            }
        }
        let streak: u16 = *stats.get(7).unwrap(); // note that streak/max are copies of the Vec data, not references, hence re-binding them later
        let max: u16 = *stats.get(8).unwrap();
        if streak > max {
            if let Some(n) = stats.get_mut(8) {
                *n = streak; // update max streak (in the stats vector) if current streak exceeds
            }
        }

        // calculate board position (top left coordinate)
        let (mut col, mut row) = termion::terminal_size().unwrap();
        match (col / 2).checked_sub(24) { // checked subtraction ensures no possible overflow error (and crash) if terminal is small
            Some(n) => col = n,
            None => col = 0,
        }
        match (row / 2).checked_sub(5) {
            Some(n) => row = n,
            None => row = 0,
        }

        // calculate some stats
        let played: u16 = stats[..=6].iter().fold(0, |acc, x| acc + x);
        let won: u16 = stats[..=5].iter().fold(0, |acc, x| acc + x);
        let percentage: u16 = ((won as f64 / played as f64) * 100.0) as u16;
        let streak: u16 = *stats.get(7).unwrap(); // redundant shadowing? But "streak" is a copy of vector data and could have been updated, so re-bind
        let max: u16 = *stats.get(8).unwrap(); // possibly redundant shadowing but just in case

        // display the stats: played, win%, current streak, max streak
        let stats_col = col + 4;
        write!(self.screen, "{}{}{}{played}{}{percentage}{}{streak}{}{max}{}Played{}Win %{}Current{}Max{}Streak{}Streak",
            clear::All, // wipe the screen
            cursor::Hide, // hide cursor
            cursor::Goto(stats_col, row),
            cursor::Goto(stats_col + 12, row),
            cursor::Goto(stats_col + 24, row),
            cursor::Goto(stats_col + 36, row),
            cursor::Goto(stats_col, row + 1), // jump down a line
            cursor::Goto(stats_col + 12, row + 1),
            cursor::Goto(stats_col + 24, row + 1),
            cursor::Goto(stats_col + 36, row + 1),
            cursor::Goto(stats_col + 24, row + 2), // jump down and back
            cursor::Goto(stats_col + 36, row + 2),
        ).unwrap();

        // display the graph
        let graph_row = row + 4; // dropping down to graph level
        write!(self.screen, "{}| 1 |{}| 2 |{}| 3 |{}| 4 |{}| 5 |{}| 6 |",
            cursor::Goto(col, graph_row),
            cursor::Goto(col, graph_row + 1),
            cursor::Goto(col, graph_row + 2),
            cursor::Goto(col, graph_row + 3),
            cursor::Goto(col, graph_row + 4),
            cursor::Goto(col, graph_row + 5),
        ).unwrap();

        // which is the "mode guess"? (it will take up the graph width and the others will be relative)
        let big_bar: u16 = stats[..=5].iter().fold(0, |acc, x| acc.max(*x));

        // print the bars
        let bar_col = col + 5;
        for line in 0..=5 {
            let count = *stats.get(line).unwrap(); // how many times have you won off that number of guesses
            let ticks: u16 = ((count as f64 / big_bar as f64) * 40.0) as u16; // number representing the length of each bar
            let mut bar = String::new(); // the actual bar characters to print
            for _ in 1..=ticks {
                bar.push('|');
            }
            if line + 1 == self.turn && self.win { // print the "turn row" green, unless failed
                write!(self.screen, "{}{}{bar} {count}{}",
                    cursor::Goto(bar_col, graph_row + line as u16),
                    color::Fg(color::LightGreen),
                    color::Fg(color::Reset)
                ).unwrap();
            } else {
                write!(self.screen, "{}{bar} {count}",
                    cursor::Goto(bar_col, graph_row + line as u16)
                ).unwrap();
            }
        }

        // flush the output stream
        self.screen.flush().unwrap();

        // attempt to write the stats to file
        let save_message_row = row + 11;
        let file = OpenOptions::new().write(true).create(true).open(filename);
        match file {
            Ok(mut file_out) => {
                for items in stats { // we're ignoring errors but notifying the user as long as it's successful
                    if let Ok(_)= write!(file_out, "{items}\n") { // write the stats to the file buffer
                        if let Ok(_) = file_out.flush() { // flush the file output and print message if successful
                            let saved_message = "Stats saved";
                            write!(self.screen, "{}{}",
                                cursor::Goto(col + 23 - (saved_message.len() as u16 / 2), save_message_row),
                                saved_message
                            ).unwrap();
                            self.screen.flush().unwrap();
                        }
                    }
                }
            },
            Err(e) => {
                let error_message = "Could not save stats:";
                write!(self.screen, "{}{}{}{e}", // notifying if there's a problem creating/opening the file
                    cursor::Goto(col + 23 - (error_message.len() as u16 / 2), save_message_row + 2),
                    error_message, // this will print the error *below* "press any key" line
                    cursor::Goto(col + 23 - (e.to_string().len() as u16 / 2), save_message_row + 3)
                    // the above "to_string()" to get to use len() should work but I'm not sure, it's hard to test
                ).unwrap();
                self.screen.flush().unwrap();
            },
        }
        std::thread::sleep(Duration::from_secs(2)); // wait a couple seconds

        // "press any key to continue"
        let exit_message = "Press any key to continue";
        let press_message_row = row + 12;
        write!(self.screen,
            "{}{}",
            cursor::Goto(col + 23 - (exit_message.len() as u16 / 2), press_message_row),
            exit_message
        ).unwrap();
        self.screen.flush().unwrap();

        // wait for key press
        press_to_continue();
    }
}

fn read_file_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> // this is from Rust By Example for "reading lines"
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines()) //returns an iterator to the reader of the lines of the file
}

struct Keyboard {
    guessed_letters: HashMap<char, Letter>,
}

impl Keyboard {
    fn initialize() -> Keyboard {
        Keyboard {
            guessed_letters: HashMap::new(),
        }
    }

    fn format(&self, coord: (u16, u16)) -> String {
        // coord in this case is where the keyboard starts, not the game board
        let qwerty_sequence: String = String::from(" _QWERTYUIOP __ASDFGHJKL ____ZXCVBNM");
        let mut _buf = String::new();
        let (col, mut row) = coord;
        _buf = format!("{}", cursor::Goto(col, row));
        for chars in qwerty_sequence.chars() {
            if chars.is_whitespace() {
                row += 1;
                _buf = format!("{_buf}{}", cursor::Goto(col, row)); // newline
            } else if chars == '_' {
                _buf = format!("{_buf} "); // for keyboard alignment
            } else { // if it's a normal letter ...
                match self.guessed_letters.get(&chars) { // ... print it depending on its guess "status"
                    Some(Letter::Green) => _buf = format!("{_buf}{} ", chars.to_string().bright_green()),
                    Some(Letter::Yellow) => _buf = format!("{_buf}{} ", chars.to_string().bright_yellow()),
                    Some(Letter::Grey) => _buf = format!("{_buf}{} ", chars.to_string().truecolor(10, 10, 10)),
                    None => _buf = format!("{_buf}{chars} "), // if that letter has not been guessed, print it normally
                }
            }
        }
        format!("{_buf}\n");
        _buf
    }
}

pub fn press_to_continue() {
    // suspends program while waiting for user to press a key
    let input = stdin();
    for key in input.keys() {
        match key.unwrap() {
            _ => break,
        }
    }
}

pub fn check_terminal() -> Result<(), &'static str> { // checks if terminal window is big enough to accommodate game
    let (width, height) = termion::terminal_size().unwrap();
    if width < 50 || height < 22 {
        return Err("Please resize terminal to at least 50 x 22");
    } else {
        return Ok(());
    }
}