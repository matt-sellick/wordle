#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn word_creator_works() {
        let name = match Word::try_new(String::from("Mateo"), &vec![String::from("MATEO")]) {
            Ok(word) => word,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(name.contents, String::from("MATEO"));
    }

    #[test]
    #[should_panic]
    fn too_long() {
        let name = match Word::try_new(String::from("Matheo"), &vec![String::from("MATHEO")]) {
            Ok(word) => word,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(name.contents, String::from("MATHEO"));
    }

    #[test]
    #[should_panic]
    fn non_alphabetic() {
        let name = match Word::try_new(String::from("Mat3o"), &vec![String::from("MAT3O")]) {
            Ok(word) => word,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(name.contents, String::from("MAT3O"));
    }
}