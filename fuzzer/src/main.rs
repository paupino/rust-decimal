#[macro_use]
extern crate clap;
extern crate decimal;
extern crate rand;
extern crate rust_decimal;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod command;

use clap::{App, Arg, SubCommand};

use std::path::Path;
use std::str::FromStr;

fn main() -> std::result::Result<(), String> {
    // Command line:
    //  1. Generate fuzz file of size (this generates expected)
    //  2. Take fuzz file and check results
    let matches = App::new("fuzzer")
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generates an input file for fuzz testing")
                .arg(
                    Arg::with_name("SAMPLE_SIZE")
                        .long("size")
                        .short("s")
                        .required(true)
                        .takes_value(true)
                        .help("The sample size to generate"),
                )
                .arg(
                    Arg::with_name("OUTPUT")
                        .long("output")
                        .short("o")
                        .required(true)
                        .takes_value(true)
                        .help("The file to output fuzz inputs"),
                ),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs the tests for the specified input file")
                .arg(
                    Arg::with_name("INPUT")
                        .required(true)
                        .help("The input file of fuzz tests to execute against"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("generate", Some(generate)) => {
            let sample_size = match u32::from_str(generate.value_of("SAMPLE_SIZE").unwrap()) {
                Ok(o) => o,
                Err(_) => panic!("invalid sample size provided"),
            };
            let output = Path::new(generate.value_of("OUTPUT").unwrap());
            command::generate(sample_size, &output)
        }
        ("run", Some(run)) => {
            let input = Path::new(run.value_of("INPUT").unwrap());
            command::run(&input)
        }
        _ => panic!("Unknown subcommand"),
    }
}
