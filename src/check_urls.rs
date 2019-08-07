extern crate clap;
extern crate reqwest;

use clap::{App, Arg};
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::process;

fn main() {
    let matches = App::new("Check Urls")
        .version("0.1.0")
        .author("ccsplit")
        .about("Takes a file of urls and checks to see if they are valid.")
        .arg(
            Arg::with_name("URLFILE")
                .help("The file with URLs which you need to check.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Place the valid URLs within the given filename.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();
    // Check if the URLFILE exists if not exit with error message.
    let urlfile = matches.value_of("URLFILE").unwrap();
    let verbosity = matches.occurrences_of("v");
    if !Path::new(urlfile).exists() {
        println!("'{}' does not exist!", urlfile);
        process::exit(1)
    }
    log_verbose("Verbose test", verbosity);
    log_debug("Debug test", verbosity);
    log_info("Should always display", verbosity);
}

fn log_info(msg: &str, level: u64) {
    println!("[INFO] {}", msg);
}

fn log_debug(msg: &str, level: u64) {
    if level > 0 {
        println!("[DEBUG] {}", msg);
    }
}

fn log_verbose(msg: &str, level: u64) {
    if level > 1 {
        println!("[VERBOSE] {}", msg);
    }
}
