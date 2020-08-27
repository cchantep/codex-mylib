use log::{info, warn};

use std::io::Read;

use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use super::{Book, empty_author, empty_book};
use crate::model::Isbn;

pub fn parse<A: Read, F: FnMut(&Book) -> ()>(r: A, mut f: F) {
    let parser = EventReader::new(r);

    let name_name = OwnedName::local("name");
    let type_name = OwnedName::local("type");
    let value_name = OwnedName::local("value");

    let first_name = OwnedName::local("firstName");
    let last_name = OwnedName::local("lastName");

    let book_name = OwnedName::local("book");
    let book_title = OwnedName::local("title");
    let book_publisher = OwnedName::local("publisher");
    let book_identifiers = OwnedName::local("identifiers");
    let book_identifier = OwnedName::local("identifier");
    let book_page_count = OwnedName::local("pageCount");
    let book_publish_date = OwnedName::local("publishDate");
    let book_description = OwnedName::local("description");
    let book_categories = OwnedName::local("categories");
    let book_category = OwnedName::local("category");
    let book_cover_url = OwnedName::local("coverUrl");
    let book_authors = OwnedName::local("authors");
    let book_author = OwnedName::local("author");

    let isbn_10 = "ISBN_10".to_string();
    let isbn_13 = "ISBN_13".to_string();
    let google_id = "GOOGLE_ID".to_string();

    let mut in_book = false;
    let mut in_book_title = false;
    let mut in_book_page_count = false;
    let mut in_book_publish_date = false;
    let mut in_book_description = false;
    let mut in_book_publisher: u8 = 0;
    let mut in_book_identifier: u8 = 0;
    let mut in_book_category: u8 = 0;
    let mut in_book_cover = false;
    let mut in_book_author = 0;

    // Accumulated book properties
    let mut book = empty_book();
    let mut id_type = 0;
    let mut author = empty_author();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) if (
                name == book_name) => {
                in_book = true;
            }

            Ok(XmlEvent::EndElement { name }) if (name == book_name) => {
                f(&book);

                in_book = false;

                book = empty_book();
                id_type = 0;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_title) => {
                in_book_title = true;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_title) => {
                in_book_title = false;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_cover_url) => {
                in_book_cover = true;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_cover_url) => {
                in_book_cover = false;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_publisher) => {
                in_book_publisher = 1;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_publisher) => {
                in_book_publisher = 0;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                (in_book_publisher == 1) && name == name_name) => {
                in_book_publisher = 2;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_categories) => {
                in_book_category = 1;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_categories) => {
                in_book_category = 0;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                (in_book_category == 1) && name == book_category) => {
                in_book_category = 2;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_identifiers) => {
                in_book_identifier = 1;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_identifiers) => {
                in_book_identifier = 0;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                (in_book_identifier == 1) && name == book_identifier) => {
                in_book_identifier = 2;
            }

            Ok(XmlEvent::EndElement { name }) if (
                (in_book_identifier == 2) && name == book_identifier) => {
                in_book_identifier = 1;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                (in_book_identifier == 2) && name == type_name) => {
                in_book_identifier = 3;
            }

            Ok(XmlEvent::EndElement { name }) if (
                (in_book_identifier == 3) && name == type_name) => {
                in_book_identifier = 2;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                (in_book_identifier == 2) && name == value_name) => {
                in_book_identifier = 4;
            }

            Ok(XmlEvent::EndElement { name }) if (
                (in_book_identifier == 4) && name == value_name) => {
                in_book_identifier = 2;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                name == book_authors) => {
                in_book_author = 1;
            }

            Ok(XmlEvent::EndElement { name }) if (name == book_authors) => {
                in_book_author = 0;

                book.authors.push(author);
                
                author = empty_author();
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book_author == 1 && name == book_author) => {
                in_book_author = 2;
            }

            Ok(XmlEvent::EndElement { name }) if (name == book_author) => {
                in_book_author = 1;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book_author >= 2 && name == first_name) => {
                in_book_author = 3;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book_author >= 2 && name == last_name) => {
                in_book_author = 4;
            }

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book_author >= 2 && name == name_name) => {
                in_book_author = 5;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_page_count) => {
                in_book_page_count = true;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_page_count) => {
                in_book_page_count = false;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_publish_date) => {
                in_book_publish_date = true;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_publish_date) => {
                in_book_publish_date = false;
            }

            // ---

            Ok(XmlEvent::StartElement { name, .. }) if (
                in_book && name == book_description) => {
                in_book_description = true;
            }

            Ok(XmlEvent::EndElement { name }) if (
                in_book && name == book_description) => {
                in_book_description = false;
            }

            // ---

            Ok(XmlEvent::Characters(value)) if (in_book_title) => {
                book.title = value;
            }

            Ok(XmlEvent::Characters(value)) if (in_book_cover) => {
                book.cover = value;
            }

            Ok(XmlEvent::Characters(value)) if (in_book_publisher == 2) => {
                book.publisher = value;
            }

            Ok(XmlEvent::Characters(value)) if (in_book_category == 2) => {
                book.kind.push(value);
            }

            Ok(XmlEvent::Characters(value)) if (in_book_author == 3) => {
                author.first_name = value;
            }

            Ok(XmlEvent::Characters(value)) if (in_book_author == 4) => {
                author.last_name = value;
            }

            Ok(XmlEvent::Characters(value)) if (in_book_author == 5) => {
                author.name = value;
            }

            Ok(XmlEvent::Characters(value)) if (
                in_book_identifier == 3) => {
                if value == isbn_10 {
                    id_type = 10;
                } else if value == isbn_13 {
                    id_type = 13;
                } else if value == google_id {
                    info!(target: "xml", "Ignore Google identifier");
                    id_type = 0;
                } else {
                    warn!("Invalid ISBN type: {}", value);
                    id_type = 0;
                }
            }

            Ok(XmlEvent::Characters(value)) if (
                in_book_identifier == 4 && id_type > 0) => {
                match id_type {
                    10 => book.isbn.push(Isbn::Isbn10(value)),

                    13 => {
                        match value.parse::<u64>() {
                            Err(cause) => {
                                warn!("Invalid ISBN13 '{}': {}", value, cause);
                            }

                            Ok(num) => {
                                book.isbn.push(Isbn::Isbn13(num))
                            }
                        }
                    }
                    
                    _ => warn!("Unexpected ISBN: {}", value),
                }

            }

            Ok(XmlEvent::Characters(value)) if (in_book_publish_date) => {
                match time::parse(value.to_string(), "%F") {
                    Ok(date) => book.pubdate = Some(date),

                    Err(cause) => {
                        warn!("Invalid publication date '{}': {}",
                              value, cause);
                    }
                }
            }

            Ok(XmlEvent::Characters(value)) if (in_book_description) => {
                book.summary = value;
            }

            Ok(XmlEvent::Characters(value)) if (in_book_page_count) => {
                match value.parse() {
                    Err(cause) => {
                        warn!("Invalid pageCount '{}': {}",
                              value, cause);
                    }

                    Ok(p) => {
                        book.pages = p;
                    }
                }
            }

            // ---

            Ok(..) => {
                //println!("Ok");
            }

            Err(cause) => warn!("Error = {}", cause),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_parse() {
        let input = "<?xml version='1.0' encoding='UTF-8' standalone='yes' ?><books version=\"2\" itemsCount=\"1\">
  <book>
    <title>Accros du roc</title>
    <authors>
      <author>
        <firstName>Terry</firstName>
        <lastName>Pratchett</lastName>
        <name>Terry Pratchett</name>
      </author>
    </authors>
    <publisher>
      <name>Pocket</name>
    </publisher>
    <identifiers>
      <identifier>
        <type>ISBN_13</type>
        <value>9782266211963</value>
      </identifier>
      <identifier>
        <type>ISBN_10</type>
        <value>226621196X</value>
      </identifier>
      <identifier>
        <type>GOOGLE_ID</type>
        <value>4iuTtwAACAAJ</value>
      </identifier>
    </identifiers>
    <publishDate>2012-07-10</publishDate>
    <description>Suzanne est une jeune étudiante discrète ...</description>
    <language>français</language>
    <pageCount>411</pageCount>
    <coverUrl>http://bks0.books.google.fr/books?id=fwIHPwAACAAJ&amp;printsec=frontcover&amp;img=1&amp;zoom=1&amp;imgtk=AFLRE711A4q0LqeTgMfMz76VFvw0yiHbNPQOTK-8nFhitUSbS8At14EQS6gzXwN1w2phGjskOqburPHmt_5LiFZQHufvU2KZ9GCB_JyQ6LeZdKysJY6gPuQ&amp;source=gbs_api</coverUrl>
    <categories>
      <category>
        <name>General</name>
      </category>
      <category>
        <name>Science Fiction</name>
      </category>
      <category>
        <name>Fiction</name>
      </category>
    </categories>
  </book>
</books>";

        parse(
            input.as_bytes(),
            |book| assert_eq!(*book, Book {
                title: "Accros du roc".to_string(),
                authors: vec![ crate::codex::Author {
                    first_name: "Terry".to_string(),
                    last_name: "Pratchett".to_string(),
                    name: "Terry Pratchett".to_string(),
                } ],
                kind: vec![
                    "General".to_string(),
                    "Science Fiction".to_string(),
                    "Fiction".to_string(),
                ],
                pubdate: Some(time::date!(2012-07-10)),
                publisher: "Pocket".to_string(),
                pages: 411,
                isbn: vec![
                    Isbn::Isbn13(9782266211963),
                    Isbn::Isbn10("226621196X".to_string()),
                ],
                summary: "Suzanne est une jeune étudiante discrète ...".to_string(),
                cover: "http://bks0.books.google.fr/books?id=fwIHPwAACAAJ&printsec=frontcover&img=1&zoom=1&imgtk=AFLRE711A4q0LqeTgMfMz76VFvw0yiHbNPQOTK-8nFhitUSbS8At14EQS6gzXwN1w2phGjskOqburPHmt_5LiFZQHufvU2KZ9GCB_JyQ6LeZdKysJY6gPuQ&source=gbs_api".to_string(),
            }));
    }
}
