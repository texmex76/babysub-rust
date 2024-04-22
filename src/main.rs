use clap::{App, Arg};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

struct Config {
    input_path: String,
    output_path: String,
    verbosity: i32,
    sign: bool,
}

fn read_input_file(input_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);

    for line in reader.lines() {
        let line = line?;
        println!("{line}");
    }
    Ok(())
}

fn main() {
    let matches = App::new("BabySub")
        .version("1.0")
        .author("Bernhard Gstrein")
        .about("Processes and simplifies logical formulae in DIMACS CNF format.")
        .arg(
            Arg::with_name("input")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("Sets the output file to use")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("verbosity")
                .short('v')
                .multiple(true)
                .takes_value(true)
                .help("Increases verbosity level"),
        )
        .arg(
            Arg::with_name("quiet")
                .short('q')
                .help("Suppresses all output"),
        )
        .arg(
            Arg::with_name("sign")
                .short('s')
                .help("Computes and adds a hash signature to the output"),
        )
        .get_matches();

    let config = Config {
        input_path: matches.value_of("input").unwrap().to_string(),
        output_path: matches.value_of("output").unwrap().to_string(),
        verbosity: matches.occurrences_of("verbosity") as i32
            - matches.occurrences_of("quiet") as i32,
        sign: matches.is_present("sign"),
    };

    println!("input_path: {}", config.input_path);
    println!("output_path: {}", config.output_path);
    println!("verbosity: {}", config.verbosity);
    println!("sign: {}", config.sign);
    if let Err(e) = read_input_file(&config.input_path) {
        eprintln!("Error reading input file: {}", e);
    }
}
