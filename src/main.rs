use clap::{App, Arg};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng}; // Ensure rand is included in Cargo.toml
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

struct CNFFormula {
    variables: usize,
    clauses: Vec<Vec<i32>>,
    literal_map: HashMap<i32, Vec<usize>>, // Map from literal to list of clause indices
}

impl CNFFormula {
    // Initializes a new CNF Formula.
    fn new() -> CNFFormula {
        CNFFormula {
            variables: 0,
            clauses: Vec::new(),
            literal_map: HashMap::new(),
        }
    }

    // Adds a clause to the CNF Formula and updates the literal map.
    fn add_clause(&mut self, clause: Vec<i32>, index: usize) {
        for &literal in &clause {
            self.literal_map
                .entry(literal)
                .or_insert_with(Vec::new)
                .push(index);
        }
        self.clauses.push(clause);
    }
    fn initialize_nonces() -> [u64; 16] {
        let mut rng = StdRng::seed_from_u64(42);
        let mut nonces = [0u64; 16];
        for nonce in nonces.iter_mut() {
            *nonce = rng.gen::<u64>() | 1; // Ensure it's not zero
        }
        nonces
    }
    fn compute_signature(&self) -> u64 {
        let nonces = Self::initialize_nonces();
        let mut hash: u64 = 0;

        for clause in &self.clauses {
            let mut d = clause.clone();
            d.sort_unstable();
            let mut tmp = (d.len() as u64 + 1).wrapping_mul(nonces[0]);
            let mut i = 1usize;

            for &ulit in &d {
                tmp = (tmp << 4) | (tmp >> 60); // Rotating bits
                tmp = tmp.wrapping_add(ulit as u64);
                tmp = tmp.wrapping_mul(nonces[i]);
                i = (i + 1) % nonces.len();
            }

            hash = hash.wrapping_add(tmp);
        }

        hash
    }
}

// Simplifies the CNF formula (placeholder for now).
fn simplify_formula(formula: &mut CNFFormula) {
    // This will contain logic to simplify the CNF formula.
}

// Configuration structure for the application settings.
struct Config {
    input_path: String,
    output_path: String,
    verbosity: i32,
    sign: bool,
}

// Parses the CNF file and constructs a CNF formula object.
fn parse_cnf(input_path: &str) -> io::Result<CNFFormula> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut formula = CNFFormula::new();
    let mut current_clause_index = 0;

    let mut lines = reader.lines();
    let mut header_parsed = false;

    while let Some(line) = lines.next() {
        let line = line?;
        if line.starts_with('c') {
            continue; // Skip comments
        }
        if line.starts_with("p cnf") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                eprintln!("Error: Invalid header format.");
                process::exit(1);
            }
            formula.variables = parts[2].parse().unwrap_or_else(|_| {
                eprintln!("Error: Invalid number of variables.");
                process::exit(1);
            });
            let _clauses_count: usize = parts[3].parse().unwrap_or_else(|_| {
                eprintln!("Error: Invalid number of clauses.");
                process::exit(1);
            });
            header_parsed = true;
        } else if header_parsed {
            let clause: Vec<i32> = line
                .split_whitespace()
                .map(|num| {
                    num.parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid literal format.");
                        process::exit(1);
                    })
                })
                .filter(|&x| x != 0)
                .collect();
            formula.add_clause(clause, current_clause_index);
            current_clause_index += 1;
        } else {
            eprintln!("Error: CNF header not found.");
            process::exit(1);
        }
    }
    Ok(formula)
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

    // Parse the CNF file and initialize the formula variable.
    let mut formula = parse_cnf(&config.input_path).unwrap_or_else(|err| {
        eprintln!("Failed to parse CNF file: {}", err);
        process::exit(1);
    });

    // Print parsed data for verbosity or debugging.
    println!("Variables: {}", formula.variables);
    println!("Clauses:");
    for (index, clause) in formula.clauses.iter().enumerate() {
        println!("{}: {:?}", index, clause);
    }
    println!("Literal Map:");
    for (literal, indices) in &formula.literal_map {
        println!("{} appears in clauses: {:?}", literal, indices);
    }

    // Simplify the CNF formula
    simplify_formula(&mut formula);
    let signature = formula.compute_signature();
    println!("Signature: {signature}");
}
