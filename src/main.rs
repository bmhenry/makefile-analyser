///
/// Parses a Makefile for targets & output information
/// 


use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;

use regex::Regex;


fn main() -> () {
    let args: Vec<String> = std::env::args().collect();

    // argument parsing
    if args.len() < 2 {
        eprintln!("Makefile path is required");
        exit(1);
    }

    let filepath = Path::new(&args[1]);
    if !filepath.exists() {
        eprintln!("File {} doesn't exist", filepath.display());
        exit(1);
    }

    // search for lines starting with a word followed by ':'
    let re = Regex::new(r"^(?P<target>[\w]+):").unwrap();

    // open the file
    let file = match File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Couldn't open {}: {}", &args[0], e);
            exit(1);
        }
    };
    let mut reader = BufReader::new(file);

    // check each line in the file to see if it matches
    let mut targets = Vec::<String>::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(len) => {
                // eof
                if len == 0 { break; }

                if let Some(matches) = re.captures(&line) {
                    targets.push(matches["target"].to_string());
                }
            }
            Err(e) => {
                eprintln!("Failed to read from file: {:?}", e);
                exit(1);
            }
        }
    }

    // print the targets, newline separated
    for t in targets {
        println!("{}", t);
    }
}
