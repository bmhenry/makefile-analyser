///
/// Parses a Makefile for targets & output information
///
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use clap::{App, Arg};
use log::*;
use serde_json::to_string_pretty;
use simplelog::*;

use makeparse::parser::Parser;

// TODO: resolve ?= with env variables if they exist
// TODO: handle included makefiles

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
        .arg(Arg::with_name("debug")
                .help("Enable debug logging")
                .long("debug"))
        .arg(Arg::with_name("logfile")
                .help("Specify a file to write error messages (stderr by default) & debug logs (stdout by default) to")
                .long("log")
                .value_name("FILE")
                .takes_value(true))
        .get_matches();

    // initialize the logger
    let loglevel = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    let mut try_term_log = !matches.is_present("logfile");

    // if a logfile was specified, try to init a log writer for that file
    if !try_term_log {
        match File::create(matches.value_of("logfile").unwrap()) {
            Ok(f) => {
                if let Err(e) = WriteLogger::init(loglevel, Config::default(), f) {
                    eprintln!(
                        "Failed to initialize file logger ({}); writing to terminal instead",
                        e
                    );
                    try_term_log = true;
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to create logfile ({}); writing to terminal instead",
                    e
                );
                try_term_log = true;
            }
        }
    }

    // if no logfile was specified or one couldn't be created, try to log to the terminal
    if try_term_log {
        if let Err(e) = TermLogger::init(loglevel, Config::default(), TerminalMode::Mixed)
            .or_else(|_| SimpleLogger::init(LevelFilter::Error, Config::default()))
        {
            eprintln!("Failed to initialize a logger: {}", e);
            // exit(1);
        }
    }

    // check to see if a valid path was given
    let filepath = Path::new(matches.value_of("INPUT").unwrap());
    if !filepath.exists() {
        error!("File {} doesn't exist", filepath.display());
        exit(1);
    }

    let mut parser = Parser::new();
    let targets = match parser.parse_file(filepath, matches.is_present("strict")) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to parse {}: {}", filepath.display(), e);
            exit(1);
        }
    };

    let ser_output = to_string_pretty(&targets).unwrap();

    // save the output to a file if specified, otherwise write to stdout
    if let Some(path) = matches.value_of("output") {
        let mut file = match File::create(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create output file: {}", e);
                exit(1);
            }
        };

        if let Err(e) = file.write_all(ser_output.as_bytes()) {
            error!("Failed to write to output file: {}", e);
            exit(1);
        }
    } else {
        println!("{}", ser_output);
    }
}
