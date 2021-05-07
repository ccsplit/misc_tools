use clap::{App, Arg};

use log::{error, trace};

use reqwest::blocking::Client;

use simplelog::*;

use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;
use std::process;
use std::sync::mpsc::channel;

use threadpool::ThreadPool;

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
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .value_name("THREADS")
                .help("The number of threads to use when checking the URLs.")
                .takes_value(true)
                .default_value("10"),
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
    let write_file = matches.is_present("output");
    let lines = BufReader::new(file).lines();
    let num_workers = clap::value_t!(matches.value_of("threads"), usize).unwrap_or(10);

    let (tx, rx) = channel();
    let pool = ThreadPool::with_name("check_url worker".to_owned(), num_workers);
    for line in lines {
        // Test the url to see if it is valid and if so print to the screen/output file.
        let url = line.unwrap();
        trace!("Testing URL: {}", url);
        let verify_tls = matches.is_present("verify-tls");
        let tx = tx.clone();
        pool.execute(move || {
            if check_url(&url, verify_tls).unwrap() {
                println!("{}", url);
                if write_file {
                    tx.send(url)
                        .expect("channel will be there waiting for the pool.");
                }
            } else {
                trace!("Failed to resolve URL: {}", url);
            }
        });
    }
    pool.join();
    drop(tx);
    if write_file {
        let outfile = Path::new(matches.value_of("output").unwrap());
        let display = outfile.display();

        let mut file = match File::create(&outfile) {
            Err(why) => panic!("Unable to create {}: {}", display, why),
            Ok(file) => file,
        };
        println!("Writing the results to: {}", display);
        for valid in rx {
            let _ = write!(&mut file, "{}\r\n", valid);
        }
    }
}

fn check_url(url: &str, verify: bool) -> Result<bool> {
    let client = if !verify {
        Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
    } else {
        Client::builder().build().unwrap()
    };
    let resp = client.get(url).send();
    let r = match resp {
        Ok(r) => r,
        Err(error) => {
            trace!("Failed to get URL: '{}', error:\n{}", url, error);
            return Ok(false);
        }
    };
    Ok(r.status().is_success())
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
    )])
    .unwrap();
}
