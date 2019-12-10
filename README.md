# Target Analyzer

Parses a Makefile for callable targets.


## Why?

Yeah, this would be a pretty simple bash script. 
But I felt like doing it in Rust, for no particular reason.
However, it is at least twice as fast as a bash script, so I guess there's that.


## Todo

* [ ] For each target, try to determine what (if any) output file is associated with it, and return that as well
* [ ] Determine the default argument

