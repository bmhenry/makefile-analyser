///
/// Parses a Makefile for targets & output information
///
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::exit;

use clap::{App, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::to_string_pretty;

pub mod types;
use types::Target;

/// Evaluate a variable using the state of other variables
// TODO: resolve ?= with env variables if they exist
// TODO: refactor to use a struct and not do everything in main
// TODO: change debug prints to debug logs
fn eval_variable(
    map: &HashMap<String, String>,
    value: &str,
    deps: Vec<&str>,
) -> Result<String, String> {
    // look for variable matches, and if found recursively resolve them
    lazy_static! {
        static ref SELFVAR: Regex = Regex::new(r"\$(?P<value>@)").unwrap();
        static ref PVAR: Regex = Regex::new(r"\$\((?P<value>[^\s:#={}()\[\]/\\]+)\)").unwrap();
        static ref CVAR: Regex = Regex::new(r"\$\{(?P<value>[^\s:#={}()\[\]/\\]+)\}").unwrap();
    }

    let mut new = value.to_string();
    println!("DEBUG: running eval on {}", new);

    // try matching against different variable types
    while let Some(range) = SELFVAR
        .find(&new)
        .or_else(|| PVAR.find(&new))
        .or_else(|| CVAR.find(&new))
    {
        // convert the regex lib's range to a rust range
        let range = range.start()..range.end();

        // get the relevant section of the value
        let wrapped_var = &new[range.clone()];
        println!("DEBUG: wrapped var: '{}'", wrapped_var);

        // unwrap the variable name
        let varname = if vec!["${", "$("].contains(&&wrapped_var[0..2]) {
            &wrapped_var[2..(wrapped_var.len() - 1)]
        } else {
            &wrapped_var[1..wrapped_var.len()]
        };
        println!("DEBUG: found variable named {}", varname);

        // make sure the variable doesn't already exist up the dependency chain
        if deps.contains(&varname) {
            return Err(format!("Variable {} has a recursive dependency", varname));
        }

        // get the variable value from the value map
        let value = if map.contains_key(varname) {
            &map[varname]
        } else {
            return Err(format!("No variable '{}'", varname));
        };
        println!("DEBUG: variable value {}", value);

        // recusrively evaluate variable values
        match eval_variable(map, &value, {
            let mut newdeps = deps.clone();
            newdeps.push(&varname);
            newdeps
        }) {
            Ok(evald) => {
                println!(
                    "DEBUG: replacing '{}' with '{}'",
                    &new[range.clone()],
                    &evald
                );
                new.replace_range(range, &evald);
            }
            Err(e) => {
                return Err(format!("Failure to parse variable: {}", e));
            }
        }
    }

    println!("DEBUG: evald line is {}", new);

    Ok(new)
}

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
        .get_matches();

    // check to see if a valid path was given
    let filepath = Path::new(matches.value_of("INPUT").unwrap());
    if !filepath.exists() {
        eprintln!("File {} doesn't exist", filepath.display());
        exit(1);
    }

    let mut vars = HashMap::<String, String>::new();

    // assume that variables have no whitespace in front of them. while this isn't strictly
    //  required by Make, in reality it's often an error otherwise
    // a make variable name can't contain whitespace, :, #, or =
    let match_variable =
        Regex::new(r"^(?P<name>[^\s:#=]+)(\s)*[?:]?=(\s)*(?P<value>[^\n\r#]+)").unwrap();

    // search for lines starting with a word followed by ':'
    let match_target = Regex::new(r"^(?P<target>[\w]+):").unwrap();

    // a list of recognized output types
    // requires indentation under a target
    let match_output = vec![
        // match a mkdir command and get the last arg passed to it
        Regex::new(r"( {4}|\t)+(mkdir)([^\n\r])*\b(?P<path>[^\n\r]+)\b").unwrap(),
        // match arbitrary stuff until -o is found
        Regex::new(r"( {4}|\t)+[^\n\r#]*-o(\s)+(?P<path>[^\s]+)").unwrap(),
        // match a specific comment with output location specifies
        Regex::new(r"( {4}|\t)+#[ \t]*Output[ \t]*:[ \t]*(?P<path>[^\s]+)").unwrap(),
    ];

    // open the file
    let file = match File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Couldn't open {}: {}", filepath.display(), e);
            exit(1);
        }
    };
    let mut reader = BufReader::new(file);

    // check each line in the file to see if it matches
    let mut targets = Vec::<Target>::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(len) => {
                // eof
                if len == 0 {
                    break;
                }

                println!("DEBUG: line = {}", line);

                // resolve any variables in the line
                let line = match eval_variable(&vars, &line, vec![]) {
                    Ok(evald) => evald,
                    Err(e) => {
                        eprintln!("Line variable expansion failed: {}", e);
                        exit(1);
                    }
                };

                // match against makefile targets
                if let Some(matches) = match_target.captures(&line) {
                    let mut t = Target::new(matches["target"].to_string());
                    if targets.is_empty() {
                        t.default = true;
                    }
                    targets.push(t);
                    // add a variable with the name `@` that will resolve to the current target
                    vars.insert("@".to_string(), matches["target"].to_string());
                }
                // match against variables
                else if let Some(matches) = match_variable.captures(&line) {
                    vars.insert(matches["name"].to_string(), matches["value"].to_string());
                }
                // match against output types
                else if !targets.is_empty() && targets[targets.len() - 1].output.is_none() {
                    // match the first output type found
                    for (i, output) in match_output.iter().enumerate() {
                        if let Some(matches) = output.captures(&line) {
                            println!("DEBUG: Found output match on output regex {}", i);
                            // get the value of the output
                            let val = matches["path"].to_string();
                            println!("DEBUG: output: '{}'", val);

                            let idx = targets.len() - 1;
                            targets[idx].output = Some(val);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from file: {:?}", e);
                exit(1);
            }
        }
    }

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
