# Target Analyzer

Parses a Makefile for callable targets and their respective outputs.


## Why?

It can be useful to programmatically determine what targets are available in a Makefile, and what potential
outputs are associated with each of those targets.
* You followed some build instructions, but don't know where to find the built product
* You want to build something in particular, but don't know which `make` target to call to get that output
* You're interested in all potential outputs of a Makefile

Originally, this was simple enough to be done in a Bash script (and probably could still be done that way),
but there's enough parsing included that it's worth writing with Rust's powerful & fast regex engine.


## Parsing Requirements

* Variables should be defined at the start of a line, with no whitespace before the variable name.
* The parser will expect one output per target, which may be either a file or a directory
	* The parser will attempt to automatically determine the output
	* An output may be specified manually with a `# Output: <path>` comment line as the first line of a target,
		for targets which output multiple items into a directory that was created in a different step
	* The first output found will be the one returned
* This parser expects output paths to be simple, i.e. no string concatenation or other tricks. The parser doesn't implement Bash.
