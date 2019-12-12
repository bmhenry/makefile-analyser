//!
//! Creates and applies filters to a vector of targets
//! 

use std::process::exit;
use clap::Values;
use crate::types::Target;
use regex::Regex;
use log::*;

pub fn filter_targets(
	targets: Vec<Target>,
	strict_mode: bool,
	filters: Option<Values>,
	includes: Option<Values>
) -> Vec<Target> 
{
	// make a list of all the filters
	let filters: Option<Vec<Regex>> = match filters {
		Some(filters) => {
		    let res = filters.filter_map(|filter| {
		    	match Regex::new(filter) {
	                Ok(re) => Some(re),
	                Err(e) => {
	                    error!("Failed to apply user filter '{}': {}", filter, e);
	                    if strict_mode {
	                        exit(1);
	                    } else {
	                    	None
	                    }
	                }
	            }
	        }).collect();
	        Some(res)
		},
		None => None
	};

	let includes: Option<Vec<Regex>> = match includes {
		Some(includes) => {
        	let res = includes.filter_map(|include| {
        		match Regex::new(include) {
	                Ok(re) => Some(re),
	                Err(e) => {
	                    error!("Failed to apply user include '{}' : {}", include, e);
	                    if strict_mode {
	                        exit(1);
	                    } else {
	                        None
	                    }
	                }
	            }
	        }).collect();
	        Some(res)
		},
		None => None
	};


	// filter the targets based on the possible filters
	targets.into_iter()
		.filter(|target| {
			if let Some(filters) = &filters {
				// don't keep any target that matches a filter
				!filters.iter().any(|re| re.is_match(&target.name)) 
			} else { true }
		})
		.filter(|target| {
			if let Some(includes) = &includes {
				// only keep targets that match any include filter
				includes.iter().any(|re| re.is_match(&target.name))
			} else { true }
		})
		.collect()
}