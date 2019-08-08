extern crate clap;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate simplelog;

use clap::{App, Arg};
use reqwest::Client;
use simplelog::*;
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
        .arg(Arg::with_name("verify-tls").help("If set will verify the TLS connection."))
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
    create_logger(verbosity);
    if !Path::new(urlfile).exists() {
        error!("URLFile: '{}' does not exist!", urlfile);
        process::exit(1)
    }
    let file = File::open(urlfile).unwrap();
    let outfile = matches.value_of("output").unwrap_or("");
    for line in BufReader::new(file).lines() {
        // Test the url to see if it is valid and if so print to the screen/output file.
        let url = line.unwrap();
        trace!("Testing URL: {}", url);
        let mut client = Client::builder().build().unwrap();

        if !matches.is_present("verify-tls") {
            client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();
        }
        let mut resp = client.get(&url).send().unwrap();
        if resp.status().is_success() {
            println!("{}", url);
        }
    }
}

fn create_logger(level: u64) {
    let mut log_level = LevelFilter::Info;
    if level > 2 {
        // Set logger to trace
        log_level = LevelFilter::Trace;
    } else if level > 1 {
        // Set logger to debug
        log_level = LevelFilter::Debug;
    }
    // Set logger to info by default.
    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap()
}
