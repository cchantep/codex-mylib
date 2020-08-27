use std::io::{Error, ErrorKind, Result, Write};

use csv::Writer;

use reqwest::blocking::Client;

use crate::codex::Book;
use crate::model::Isbn;

pub const DEFAULT_COVER_DIRECTORY: &str = "/MyLibrary/Images/Books";

const DEFAULT_COVER_CONTENT_TYPE: &str = "image/jpeg";

pub fn write<A: Write, B: Write>(
    csv_writer: &mut Writer<A>,
    img_writer: &mut B,
    book: &Book,
    cover_dir: &str,
    http: &Client,
) -> Result<()> {
    let authors: Vec<String> =
        book.authors.iter().map(|a| a.name.to_string()).collect();

    let pubdate = book.pubdate.
        map(|d| d.format("%d/%m/%Y")).unwrap_or("".to_string());

    let isbn = book.isbn.iter().find(|i| match i {
        Isbn::Isbn13(_) => true,
        _ => false
    }).or(book.isbn.first()).map_or_else(
        || "".to_string(),
        |i| match i {
            Isbn::Isbn13(i13) => i13.to_string(),
            Isbn::Isbn10(i10) => i10.to_string(),
        });

    let cover: Result<Option<String>> = {
        if !book.cover.is_empty() {
            let url: &String = &book.cover;

            log::info!(target: "mylib", "Cover URL: {}", url);

            book_hashcode(&book.title, &authors).
                and_then(|h| resolve_cover(http, url, h, img_writer).
                         map_err(|cause| {
                             log::warn!("Fails to resolve cover '{}': {}",
                                        url, cause);

                             cause
                         }).
                         map(|c| {
                             let file_ext = match c.as_str() {
                                 "image/png" => "png",
                                 _ => "jpg"
                             };

                             let id: String = {
                                 let i = &isbn;

                                 if i.is_empty() {
                                     h.to_string()
                                 } else {
                                     i.to_string()
                                 }
                             };

                             Some(format!("{}/{}.{}", cover_dir, id, file_ext))
                         }))

        } else {
            Ok(None)
        }
    };

    let cover_path: String = cover.map_or_else(
        |_| "_invalid_".to_string(),
        |url| url.map_or_else(|| "".to_string(), |u| u.to_string()));

    match csv_writer.write_record(&[
        book.title.to_string(),
        authors.join(", "),
        "".to_string(), // serie
        book.kind.join(", "),
        pubdate,
        book.publisher.to_string(),
        book.pages.to_string(),
        isbn,
        "".to_string(), // lu
        "".to_string(), // period
        "".to_string(), // comment
        book.summary.to_string(),
        cover_path,
    ]) {
        Ok(_) => csv_writer.flush(),
        Err(cause) => Err(Error::new(ErrorKind::Other, cause)),
    }
}

use base64::write::EncoderWriter;
use reqwest::header::CONTENT_TYPE;

fn resolve_cover<'a, A: Write>(
    http: &'a Client,
    url: &'a String,
    hashcode: i32,
    img_writer: &'a mut A,
) -> Result<String> {
    http.get(url).send().map_or_else(
            |cause| Err(Error::new(ErrorKind::Interrupted, cause)),
            |mut r| {
                if !r.status().is_success() {
                    Err(Error::new(
                        ErrorKind::Interrupted,
                        format!("Fails to get cover: {}", url)))

                } else {
                    let mut img_buf: Vec<u8> = vec![];

                    let c: Result<u64> = {
                        let mut cover_writer =
                            EncoderWriter::new(&mut img_buf, base64::STANDARD);
                        
                        std::io::copy(&mut r, &mut cover_writer)
                    };

                    if c? <= 0 {
                        Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Missing cover data: {}", url)))

                    } else {
                        let json_cover =
                            std::str::from_utf8(&img_buf).map_or_else(
                                |cause| Err(Error::new(
                                    ErrorKind::Interrupted,
                                    format!("Fails to encode cover image '{}': {}", url, cause))),
                                |b64img| Ok(json::object!{
                                    base64Image: b64img,
                                    elementHashcode: hashcode,
                                    imageOrientation: 0,
                                    type: "BOOK",
                                }))?;

                        let tpe = r.headers()[CONTENT_TYPE].
                            to_str().map_or_else(
                                |cause| {
                                    log::warn!("Fails to determine type for cover '{}': {}", url, cause);
                                    
                                    DEFAULT_COVER_CONTENT_TYPE.to_string()
                                },
                                |s| s.to_string());

                        let line = json::stringify(json_cover) + &"\r\n";

                        img_writer.write(line.as_bytes()).map(|_| tpe)
                    }
                }
            })
}

const JAVA_HASH_SEED: i32 = 31;

fn java_hashcode(str: String) -> i32 {
    let mut h: i32 = 0;
    
    for ch in str.chars() {
        h = JAVA_HASH_SEED.wrapping_mul(h).wrapping_add(ch as i32);
    }

    return h;
}

fn book_hashcode(title: &String, authors: &Vec<String>) -> Result<i32> {
    let string_repr = authors.first().map_or_else(
        || Err(Error::new(ErrorKind::Other, "Missing author")),
        |author| Ok(format!("{}{}", title, author)));

    string_repr.map(|repr| java_hashcode(repr))
}

// ---

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_java_hashcode() {
        assert_eq!((2457 as i32), java_hashcode("Le".to_string()));
        assert_eq!((65282059 as i32), java_hashcode("Codex".to_string()));
        assert_eq!((816537616 as i32), java_hashcode("Guta-Sintram".to_string()));
        assert_eq!((100742 as i32), java_hashcode("est".to_string()));
        assert_eq!((3737 as i32), java_hashcode("un".to_string()));
        assert_eq!((109904722 as i32), java_hashcode("manuscrit".to_string()));
        assert_eq!((1974693833 as i32), java_hashcode("enluminé".to_string()));
        assert_eq!((3076146 as i32), java_hashcode("daté".to_string()));
        assert_eq!((3201 as i32), java_hashcode("de".to_string()));
        assert_eq!((1508543 as i32), java_hashcode("1154".to_string()));
        assert_eq!((-1197664643 as i32), java_hashcode("exécuté".to_string()));
        assert_eq!((224 as i32), java_hashcode("à".to_string()));
        assert_eq!((-1926557657 as i32), java_hashcode("l'abbaye".to_string()));
        assert_eq!((3201 as i32), java_hashcode("de".to_string()));
        assert_eq!((-1791335646 as i32), java_hashcode("Marbach".to_string()));
        assert_eq!((3241 as i32), java_hashcode("en".to_string()));
        assert_eq!((759338387 as i32), java_hashcode("Alsace.".to_string()));
        assert_eq!((76995002 as i32), java_hashcode("Peint".to_string()));
        assert_eq!((3075842 as i32), java_hashcode("dans".to_string()));
        assert_eq!((3449 as i32), java_hashcode("le".to_string()));
        assert_eq!((109780401 as i32), java_hashcode("style".to_string()));
        assert_eq!((-925389361 as i32), java_hashcode("roman,".to_string()));
        assert_eq!((3363 as i32), java_hashcode("il".to_string()));
        assert_eq!((100742 as i32), java_hashcode("est".to_string()));
        assert_eq!((-28034847 as i32), java_hashcode("actuellement".to_string()));
        assert_eq!((-568241327 as i32), java_hashcode("conservé".to_string()));
        assert_eq!((224 as i32), java_hashcode("à".to_string()));
        assert_eq!((3445 as i32), java_hashcode("la".to_string()));
        assert_eq!((1977356260 as i32), java_hashcode("Bibliothèque".to_string()));
        assert_eq!((3217 as i32), java_hashcode("du".to_string()));
        assert_eq!((69062892 as i32), java_hashcode("Grand".to_string()));
        assert_eq!((1713345047 as i32), java_hashcode("séminaire".to_string()));
        assert_eq!((3201 as i32), java_hashcode("de".to_string()));
        assert_eq!((2049634778 as i32), java_hashcode("Strasbourg".to_string()));
        assert_eq!((-347316087 as i32), java_hashcode("(Ms.37).".to_string()));
    }

    #[test]
    fn test_book_hashcode() {
        assert_eq!(1663365717, book_hashcode(
            &"Japon, Miscellanées".to_string(),
            &vec![
                "Chantal DELTENRE".to_string(),
                "Maximilien DAUBER".to_string()
            ]).unwrap());

        assert_eq!(49491434, book_hashcode(
            &"Ainsi Parlait Zarathoustra".to_string(),
            &vec![ "Friedrich Wilhelm Nietzsche".to_string() ]).unwrap());

        assert_eq!(1763833006, book_hashcode(
            &"Alien Earth".to_string(),
            &vec![ "Robin Hobb".to_string() ]).unwrap());

        assert_eq!(-1061641663, book_hashcode(
            &"Allez les mages !".to_string(),
            &vec![ "Terry Pratchett".to_string() ]).unwrap());

        assert_eq!(-1648148861, book_hashcode(
            &"Ally".to_string(),
            &vec![ "Karen Traviss".to_string() ]).unwrap());

        assert_eq!(1397285980, book_hashcode(
            &"Va-t-en-guerre".to_string(),
            &vec![ "Terry Pratchett".to_string() ]).unwrap());

        assert_eq!(411805042, book_hashcode(
            &"Vision aveugle".to_string(),
            &vec![ "Peter Watts".to_string() ]).unwrap());

        assert_eq!(2094868745, book_hashcode(
            &"Vulture Peak".to_string(),
            &vec![ "John Burdett".to_string() ]).unwrap());

        assert_eq!(-1547775297, book_hashcode(
            &"Échopraxie".to_string(),
            &vec![ "Peter Watts".to_string() ]).unwrap());

        assert_eq!(-1492802074, book_hashcode(
            &"Élévation".to_string(),
            &vec![ "David Brin".to_string() ]).unwrap());

    }
}
