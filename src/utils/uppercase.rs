pub trait FirstLetterToUpperCase {
    fn first_to_uppercase(self) -> String;
}

impl FirstLetterToUpperCase for String {
    fn first_to_uppercase(self) -> String {
        let mut c = self.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}