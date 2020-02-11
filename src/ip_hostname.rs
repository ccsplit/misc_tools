use clap::{App, Arg};

use dns_lookup::lookup_addr;

use log::{error, info, trace};

use ipnet::{IpAddrRange, IpNet, Ipv4AddrRange, Ipv6AddrRange};

use simplelog::*;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::IpAddr;
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::string::String;
use std::sync::mpsc::channel;
use std::vec::Vec;

use threadpool::ThreadPool;

fn main() {
    let matches = App::new("Ip => Hostname")
        .version("0.1.0")
        .author("ccsplit")
        .about(
            "A utility to grab the hostnames for the specified IPs/CIDRs/Ranges
            using reverse DNS lookups.",
        )
        .arg(
            Arg::with_name("IPS")
                .help("An IP, CIDR, IPRange, or file if the (-f|--file) flag is specified.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .help("The passed in value is a file."),
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .value_name("THREADS")
                .help("The number of threads to use when getting hostnames for the IPs.")
                .takes_value(true)
                .default_value("10"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Place the valid IPs and hostnames within the given filename.")
                .takes_value(true),
        )
        .get_matches();
    let verbosity = matches.occurrences_of("v");
    create_logger(verbosity);
    let mut ips = Vec::new();
    let write_file = matches.is_present("output");
    let ips_value = matches.value_of("IPS").unwrap();
    let num_workers = clap::value_t!(matches.value_of("threads"), usize).unwrap_or(10);
    let is_file = matches.is_present("file");
    let pool = ThreadPool::with_name("IP => Hostname worker".to_owned(), num_workers);
    let (tx, rx) = channel();
    if is_file {
        if !Path::new(ips_value).exists() {
            error!("IPS File: '{}' does not exist!", ips_value);
            process::exit(1)
        }
        let file = File::open(ips_value).unwrap();
        let lines = BufReader::new(file).lines();
        for line in lines {
            ips.push(line.unwrap());
        }
    } else {
        ips.push(String::from(ips_value));
    }
    for ip_obj in &ips {
        if ip_obj.find('/').is_some() {
            // Handle if the string is a CIDR..
            let net = IpNet::from_str(ip_obj).unwrap();
            trace!("Parsed: '{:?}", net);
            for ip in net.hosts() {
                let tx = tx.clone();
                pool.execute(move || {
                    trace!("Creating thread for: '{}'", ip);
                    if let Some(i) = resolve_ip(ip) {
                        let msg = format!("{} => {}", ip, i);
                        info!("{}", msg);
                        if write_file {
                            tx.send(msg)
                                .expect("channel will be there waiting for the pool.");
                        }
                    }
                });
            }
        } else if ip_obj.find('-').is_some() {
            // Handle if the string is an IPRange.
            let ips: Vec<&str> = ip_obj.split('-').collect();
            let ip_range: IpAddrRange;
            if ips[0].find(':').is_some() {
                // IPv6
                ip_range = IpAddrRange::from(Ipv6AddrRange::new(
                    ips[0].trim().parse().unwrap(),
                    ips[1].trim().parse().unwrap(),
                ));
            } else {
                ip_range = IpAddrRange::from(Ipv4AddrRange::new(
                    ips[0].trim().parse().unwrap(),
                    ips[1].trim().parse().unwrap(),
                ));
            }
            for ip in ip_range {
                let tx = tx.clone();
                pool.execute(move || {
                    trace!("Creating thread for: '{}'", ip);
                    if let Some(i) = resolve_ip(ip) {
                        let msg = format!("{} => {}", ip, i);
                        info!("{}", msg);
                        if write_file {
                            tx.send(msg)
                                .expect("channel will be there waiting for the pool.");
                        }
                    }
                });
            }
        } else {
            let ip = ip_obj.parse::<IpAddr>().unwrap();
            let tx = tx.clone();
            pool.execute(move || {
                trace!("Creating thread for: '{}'", ip);
                if let Some(i) = resolve_ip(ip) {
                    let msg = format!("{} => {}", ip, i);
                    info!("{}", msg);
                    if write_file {
                        tx.send(msg)
                            .expect("channel will be there waiting for the pool.");
                    }
                }
            });
        }
        pool.join();
    }
    drop(tx);
    if write_file {
        let outfile = Path::new(matches.value_of("output").unwrap());
        let display = outfile.display();

        let mut file = match File::create(&outfile) {
            Err(why) => panic!("Unable to create {}: {}", display, why.description()),
            Ok(file) => file,
        };
        println!("Writing the results to: {}", display);
        for valid in rx {
            let _ = write!(&mut file, "{}\r\n", valid);
        }
    }
}

fn resolve_ip(ip: IpAddr) -> Option<String> {
    trace!("Attempting to resolve: '{}", ip);
    match lookup_addr(&ip) {
        Ok(r) => {
            trace!("Resolved ip address '{}' to '{}", ip, r);
            if ip.to_string() == r {
                return None;
            }
            Some(r)
        }
        Err(err) => {
            trace!("Unable to resolve: '{:?}'. Error occurred:\n{}", ip, err);
            None
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
