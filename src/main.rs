mod valid_guesses;
mod secret_words;

use wordle::get_input;
use wordle::{Board, Word};
use std::process;
use crate::valid_guesses::ValidGuesses;
use crate::secret_words::SecretWords;

// a command-line reconstruction of Wordle.
// tried to be faithful to original game! Only thing is it does not do is track wins.
// randomly selects a secret word on every launch.

fn main() {

    // game setup
    println!("\n     W O R D L E");
    let valid_guesses = ValidGuesses::load().contents;
    let secret_word = match Word::try_new(SecretWords::load().choose_secret(), &valid_guesses) { // note that secret words must also be in the valid guess list
        Ok(w) => w,
        Err(e) => panic!("Error choosing secret word: {}", e), // if it can't load a secret word it should panic
    };

    // for testing:
    // println!("Secret word is: {:?}", secret_word);

    // init game board
    let mut game_board = Board::new(secret_word); // board owns secret word
    game_board.draw(0);

    // turn loop
    for turn in 1..=6 as usize {
        // get user input
        println!("\nEnter your guess:");
        loop {
            let guess = match Word::try_new(get_input(), &valid_guesses) { // asks for a guess word
                Ok(g) => g,
                Err(e) => {
                    println!("{}", e);
                    continue;
                },
            };
            game_board.guesses.push(guess); // game_board will own guesses.
            break;
        }
        
        // display the board
        game_board.draw(turn);

        // check if the guess is right
        if game_board.check_guess() {
            game_board.win_message(turn);
            process::exit(0);
        }
    }
    
    // if you finish six turns without getting it right ...
    game_board.win_message(7);
}