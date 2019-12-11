//!
//! Handles parsing a Makefile, line by line
//! 


use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use lazy_static::lazy_static;
use log::*;
use regex::Regex;


use crate::types::Target;


pub struct Parser {
    targets: Vec<Target>,
    vars: HashMap<String,String>,
    match_var_def: Regex,
    match_target_def: Regex,
    match_output: Vec<Regex>,
}

impl Parser {
    /// Create a new Parser
    pub fn new() -> Self {
        Parser {
            targets: Vec::<Target>::new(),
            vars: HashMap::<String,String>::new(),
            // assume that variables have no whitespace in front of them. while this isn't strictly
            // required by Make, in reality it's often an error otherwise.
            // a make variable name can't contain whitespace, :, #, or =
            match_var_def: Regex::new(r"^(?P<name>[^\s:#=]+)(\s)*[?:]?=(\s)*(?P<value>[^\n\r#]+)").unwrap(),
            // search for lines starting with a word followed by ':'
            match_target_def: Regex::new(r"^(?P<target>[\w]+):").unwrap(),
            // a list of recognized output types
            // requires indentation under a target
            match_output: vec![
                // match a mkdir command and get the last arg passed to it
                Regex::new(r"( {4}|\t)+(mkdir)([^\n\r])*\b(?P<path>[^\n\r]+)\b").unwrap(),
                // match arbitrary stuff until -o is found
                Regex::new(r"( {4}|\t)+[^\n\r#]*-o(\s)+(?P<path>[^\s]+)").unwrap(),
                // match a specific comment with output location specifies
                Regex::new(r"( {4}|\t)+#[ \t]*Output[ \t]*:[ \t]*(?P<path>[^\s]+)").unwrap(),
            ],
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, filepath: P) -> Result<Vec<Target>, String> {
        let filepath = filepath.as_ref();

        // open the file for line-by-line reading
        let file = match File::open(filepath) {
            Ok(f) => f,
            Err(e) => return Err(format!("Couldn't open {}: {}", filepath.display(), e))
        };
        let mut reader = BufReader::new(file);

        // check each line in the file to see if it matches
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(len) => {
                    // eof
                    if len == 0 {
                        break;
                    }

                    debug!("line: '{}'", line.trim_end());

                    // resolve any variables in the line
                    let line = match self.eval_variable(&line, vec![]) {
                        Ok(evald) => evald,
                        Err(e) => return Err(format!("Line variable expansion failed: {}", e))
                    };

                    // match against makefile targets
                    if let Some(matches) = self.match_target_def.captures(&line) {
                        let mut t = Target::new(matches["target"].to_string());
                        if self.targets.is_empty() {
                            t.default = true;
                        }
                        self.targets.push(t);
                        // add a variable with the name `@` that will resolve to the current target
                        self.vars.insert("@".to_string(), matches["target"].to_string());
                    }
                    // match against variables
                    else if let Some(matches) = self.match_var_def.captures(&line) {
                        self.vars.insert(matches["name"].to_string(), matches["value"].to_string());
                    }
                    // match against output types
                    else if !self.targets.is_empty() && self.targets[self.targets.len() - 1].output.is_none() {
                        // match the first output type found
                        for (i, output) in self.match_output.iter().enumerate() {
                            if let Some(matches) = output.captures(&line) {
                                debug!("Found output match on output regex {}", i);
                                // get the value of the output
                                let val = matches["path"].to_string();
                                debug!("output: '{}'", val);

                                let idx = self.targets.len() - 1;
                                self.targets[idx].output = Some(val);
                            }
                        }
                    }
                }
                Err(e) => return Err(format!("Failed to read from file: {:?}", e))
            }
        }

        Ok(self.targets.clone())
    }

    /// Evaluate a variable recursively until the actual value is determined, using other
    ///  variables as necessary
    fn eval_variable(&mut self, value: &str, deps: Vec<&str>) -> Result<String, String> {
        // look for variable matches, and if found recursively resolve them
        lazy_static! {
            static ref SELFVAR: Regex = Regex::new(r"\$(?P<value>@)").unwrap();
            static ref PVAR: Regex = Regex::new(r"\$\((?P<value>[^\s:#={}()\[\]/\\]+)\)").unwrap();
            static ref CVAR: Regex = Regex::new(r"\$\{(?P<value>[^\s:#={}()\[\]/\\]+)\}").unwrap();
        }

        let mut new = value.to_string();
        debug!("running eval on '{}'", new.trim_end());

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
            debug!("wrapped var: '{}'", wrapped_var);

            // unwrap the variable name
            let varname = if vec!["${", "$("].contains(&&wrapped_var[0..2]) {
                &wrapped_var[2..(wrapped_var.len() - 1)]
            } else {
                &wrapped_var[1..wrapped_var.len()]
            };
            debug!("found variable named {}", varname);

            // make sure the variable doesn't already exist up the dependency chain
            if deps.contains(&varname) {
                return Err(format!("Variable {} has a recursive dependency", varname));
            }

            // get the variable value from the value map
            let value = if self.vars.contains_key(varname) {
                self.vars[varname].clone()
            } else {
                return Err(format!("No variable '{}'", varname));
            };
            debug!("variable value {}", value);

            // recusrively evaluate variable values
            match self.eval_variable(&value, {
                let mut newdeps = deps.clone();
                newdeps.push(&varname);
                newdeps
            }) {
                Ok(evald) => {
                    debug!(
                        "replacing '{}' with '{}'",
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

        debug!("eval'd line: '{}'", new.trim_end());

        Ok(new)
    }
}