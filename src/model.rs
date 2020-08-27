#[derive(Debug)]
pub enum Isbn {
    Isbn10(String),
    Isbn13(u64),
}

// ---

impl ToString for Isbn {
    fn to_string(&self) -> String {
        match self {
            Isbn::Isbn10(value) => format!("ISBN10:{}", value),
            Isbn::Isbn13(value) => format!("ISBN13:{}", value),
        }
    }
}

impl PartialEq for Isbn {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Isbn::Isbn10(a) => match other {
                Isbn::Isbn10(b) => a == b,
                _ => false,
            },

            Isbn::Isbn13(a) => match other {
                Isbn::Isbn13(b) => a == b,
                _ => false,
            },
        }
    }
}
