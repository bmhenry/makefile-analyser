//!
//! Stores data relevant to a single make target
//!

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    // name of the target
    pub name: String,
    // whether this is the default (first) target or not
    pub default: bool,
    // output path associated with the target (may be a file or folder)
    pub output: Option<String>,
}

impl Target {
    pub fn new(name: String) -> Self {
        Target {
            name,
            default: false,
            output: None,
        }
    }
}
