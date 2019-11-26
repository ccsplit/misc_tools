extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;

use clap::{App, Arg};
use simplelog::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::net::IpAddr;
use std::path::Path;
use std::process;
use std::str::FromStr;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver;

fn main() {
    let matches = App::new("IP => Hostnames")
        .version("0.1.0")
        .author("ccsplit")
        .about("Takes a file of IPs, CIDRs and ranges to try and get a list of hostnames.")
        .arg(
            Arg::with_name("IPFILE")
                .help("The file with IP/CIDRs/Ranges which you need to check.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Place the valid hostnames within the given filename.")
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
    let ipfile = matches.value_of("IPFILE").unwrap();
    let verbosity = matches.occurrences_of("v");
    create_logger(verbosity);
    if !Path::new(ipfile).exists() {
        error!("IPFile: '{}' does not exist!", ipfile);
        process::exit(1)
    }
    let file = File::open(ipfile).unwrap();
    let outfile = matches.value_of("output").unwrap_or("");
    let mut resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    for line in BufReader::new(file).lines() {
        // Test the url to see if it is valid and if so print to the screen/output file.
        let value = line.unwrap();
        trace!("Attempting to resolve hostname for IP: {}", value);
        let response = resolver
            .reverse_lookup(IpAddr::from_str(&value).unwrap())
            .unwrap();
        println!("{:?}", response);
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
