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
