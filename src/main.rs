mod valid_guesses;
mod secret_words;

use wordle::{Board, Word};
use crate::valid_guesses::ValidGuesses;
use crate::secret_words::SecretWords;

// A TUI reconstruction of Wordle by Matt Sellick
// Randomly selects a secret word on every launch
// Saves a text file to the working directory when the game ends to store stats

fn main() {

    // check terminal size
    wordle::enforce_terminal();

    // game setup
    let valid_guesses = ValidGuesses::load().contents;
    let secret_word = match Word::try_new(SecretWords::load().choose_secret(), &valid_guesses) { // note that secret words must also be in the valid guess list
        Ok(w) => w,
        Err(e) => panic!("Error choosing secret word: {}", e), // if it can't load a secret word it should panic
    };

    // for testing:
    // println!("\nSecret word is: {}", secret_word.contents());
    // std::thread::sleep(std::time::Duration::from_secs(2));

    // initialize game board, moving into alternate screen
    let mut game_board = Board::new(secret_word);
    game_board.welcome();
    game_board.draw();

    // turn loop
    for turn in 1..=6 as usize {

        // update turn in Board
        game_board.turn = turn;

        // get user input
        loop {
            let guess = match Word::try_new(game_board.get_input(), &valid_guesses) { // asks for a guess word
                Ok(g) => {
                    if game_board.hard { // if you're in hard mode, make sure it's a legal guess before binding
                        match game_board.hard_check(&g) {
                            Ok(_) => g,
                            Err(error) => {
                                game_board.print_msg(&error);
                                continue;
                            },
                        }
                    } else { // normal mode
                        g
                    }
                },
                Err(e) => {
                    game_board.print_msg(e);
                    continue;
                },
            };
            game_board.guesses.push(guess); // game_board will own guesses.
            break;
        }
        
        // display the board
        game_board.draw();

        // check if the guess is right
        if game_board.check_guess() {
            game_board.win = true;
            break;
        }
    }

    // game end
    game_board.win_message(); // display win message and wait for key press
    game_board.stats(); // display stats and wait for key press
    drop(game_board); // return to main screen
}