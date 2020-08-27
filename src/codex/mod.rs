use std::option::Option;

use std::fmt::{Display,Formatter};

use time::Date;

use super::model::Isbn;

pub mod util;

#[derive(Debug)]
pub struct Book {
    pub title: String,
    pub authors: Vec<Author>,
    pub kind: Vec<String>,
    pub pubdate: Option<Date>,
    pub publisher: String,
    pub pages: u16,
    pub isbn: Vec<Isbn>,
    pub summary: String,
    pub cover: String,
}

pub fn empty_book() -> Book {
    Book {
        title: "".to_string(),
        authors: vec![],
        kind: vec![],
        pubdate: None,
        publisher: "".to_string(),
        pages: 0,
        isbn: vec![],
        summary: "".to_string(),
        cover: "".to_string(),
    }
}

#[derive(Debug)]
pub struct Author {
    pub first_name: String,
    pub last_name: String,
    pub name: String,
}

pub fn empty_author() -> Author {
    Author {
        first_name: "".to_string(),
        last_name: "".to_string(),
        name: "".to_string(),
    }
}

// ---

impl ToString for Author {
    fn to_string(&self) -> String {
        format!("Author[ {}, {}, {}, ]",
                self.first_name, self.last_name, self.name)
    }
}

impl Display for Book {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let isbns: Vec<String> =
            self.isbn.iter().map(|i| i.to_string()).collect();

        let authors: Vec<String> =
            self.authors.iter().map(|a| a.to_string()).collect();

        write!(
            formatter,
            "Book[ #title[{}], #authors[{}], #kind[{}], {}, {}, {}, {}, #summary[{}], #cover[{}] ]",
            ellipsis(&self.title, 30),
            authors.join(", "),
            self.kind.join(", "),
            self.pubdate.map(|d| d.to_string()).unwrap_or("".to_string()),
            self.publisher,
            self.pages,
            isbns.join(", "),
            ellipsis(&self.summary, 30),
            self.cover)
    }
}

fn ellipsis(text: &String, max: usize) -> String {
    let str = text.as_str();

    if text.len() > max {
        let mut prepared: String = str.chars().into_iter().take(max).collect();

        prepared.push_str("...");

        return prepared;
    }

    return String::from(str);
}

impl PartialEq for Author {
    fn eq(&self, other: &Self) -> bool {
        self.first_name == other.first_name &&
            self.last_name == other.last_name &&
            self.name == other.name
    }
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title &&
            self.authors == other.authors &&
            self.kind == other.kind &&
            self.pubdate == other.pubdate &&
            self.publisher == other.publisher &&
            self.pages == other.pages &&
            self.isbn == other.isbn &&
            self.summary == other.summary &&
            self.cover == other.cover
    }
}
