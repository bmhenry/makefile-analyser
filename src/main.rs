///
/// Parses a Makefile for targets & output information
///

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use clap::{App, Arg};
use serde_json::to_string_pretty;

use makeparse::parser::Parser;


// TODO: resolve ?= with env variables if they exist
// TODO: refactor to use a struct and not do everything in main
// TODO: change debug prints to debug logs
// TODO: handle included makefiles
// TODO: implement strict handler

fn main() {
    // parse command line arguments
    let matches = App::new("makefile-analyzer")
        .version("0.1.0")
        .author("Brandon Henry <brandon@bhenry.dev>")
        .about("Analyzes a Makefile's targets and outputs")
        .arg(
            Arg::with_name("INPUT")
                .help("Makefile to be parsed")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .help("Output file to write JSON results to (stdout by default)")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(Arg::with_name("strict")
                .help("Fail on any parser error")
                .short("s")
                .long("strict"))
        .get_matches();

    // check to see if a valid path was given
    let filepath = Path::new(matches.value_of("INPUT").unwrap());
    if !filepath.exists() {
        eprintln!("File {} doesn't exist", filepath.display());
        exit(1);
    }

    let mut parser = Parser::new();
    let targets = match parser.parse_file(filepath) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to parse {}: {}", filepath.display(), e);
            exit(1);
        }
    };

    let ser_output = to_string_pretty(&targets).unwrap();

    // save the output to a file if specified, otherwise write to stdout
    if let Some(path) = matches.value_of("output") {
        let mut file = match File::create(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create output file: {}", e);
                exit(1);
            }
        };

        if let Err(e) = file.write_all(ser_output.as_bytes()) {
            eprintln!("Failed to write to output file: {}", e);
            exit(1);
        }
    } else {
        println!("{}", ser_output);
    }
}
