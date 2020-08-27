use std::path::Path;
use std::fs::File;

use std::io::{Error, ErrorKind, BufReader, BufWriter, Write};

use reqwest::blocking::Client;

use clap::{Arg, App};

mod model;
mod codex;

fn main() {
    let matches = App::new("Codex-Mylib").
        about("Converts Codex XML to Mylib").
        arg(Arg::with_name("INPUT_FILE").
            short("i").
            long("input").
            help("Path to Codex XML file").
            takes_value(true).
            required(true)).
        arg(Arg::with_name("OUTPUT_DIR").
            short("o").
            long("output").
            help("Path to directory where to write Mylib files").
            takes_value(true).
            required(true)).
        arg(Arg::with_name("COVER_TARGET_DIR").
            short("ct").
            long("cover-target").
            help(&format!("Path to directory where cover images are imported (default: {})", mylib::DEFAULT_COVER_DIRECTORY)).
            takes_value(true).
            required(false)).
        get_matches();

    let input = matches.value_of("INPUT_FILE").expect("Missing input");

    log::info!(target: "cli", "Input file = {}", input);

    let out_basepath = || {
        return matches.value_of("OUTPUT_DIR").
            and_then(|dir| {
                return Path::new(input).with_extension("").
                    file_name().and_then(|os| os.to_str()).
                    map(|basename| format!("{}{}", dir, basename));
            }).map_or_else(
                || Err(Error::new(ErrorKind::NotFound, "Output not found")),
                |o| Ok(o));
    };

    let res = File::open(input).
        and_then(|f| out_basepath().and_then(|out| {
            let csv_path = format!("{}-mylib.csv", out);
            let img_path = format!("{}-mylib-images.txt", out);

            println!("Will write CSV to '{}' and images to '{}'",
                     csv_path, img_path);

            return File::create(csv_path).
                and_then(|of| File::create(img_path).map(|imf| (of, imf))).
                and_then(|st| {
                    let (of, imf) = st;
                    
                    let http = Client::builder().
                        timeout(std::time::Duration::from_secs(30)).build().
                        map_err(|cause| Error::new(ErrorKind::Other, cause));

                    http.map(|h| (f, of, imf, h))
                })
        }));

    match res {
        Err(cause) => {
            println!("Fails to convert from '{}': {}", input, cause);
        }

        Ok((inf, out, imf, http)) => {
            let r = BufReader::new(inf);
            let csv = BufWriter::new(out);
            let mut img = BufWriter::new(imf);
            let cover_dir = matches.value_of("COVER_TARGET_DIR").
                unwrap_or_else(|| mylib::DEFAULT_COVER_DIRECTORY);

            codex::util::parse(r, on_book(csv, &http, cover_dir, &mut img));
        }
    };
}

// ---

mod mylib;

fn on_book<'a, A: Write + 'a, B: Write>(
    csv_writer: A,
    http: &'a Client,
    cover_dir: &'a str,
    img_writer: &'a mut B,
) -> impl FnMut(&codex::Book) -> () + 'a {
    let mut cw = csv::WriterBuilder::new().
        delimiter(b';').
        quote_style(csv::QuoteStyle::NonNumeric).
        from_writer(csv_writer);

    return move |book| {
        match mylib::write(&mut cw, img_writer, book, cover_dir, http) {
            Err(cause) => log::warn!("Fails to write book as CSV: {}", cause),
            _ => ()
        }
    };
}
