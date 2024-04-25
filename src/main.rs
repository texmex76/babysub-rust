use clap::{App, Arg};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

struct CNFFormula {
    variables: usize,
    clauses: Vec<Vec<i32>>,
    literal_map: HashMap<i32, Vec<usize>>,
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
        [
            71876167, 708592741, 1483128881, 907283241, 442951013, 537146759, 1366999021,
            1854614941, 647800535, 53523743, 783815875, 1643643143, 682599717, 291474505,
            229233697, 1633529763,
        ]
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

// fn simplify_formula(formula: &mut CNFFormula) {
//     // This will contain logic to simplify the CNF formula.
// }

struct Config {
    input_path: String,
    output_path: String,
    verbosity: i32,
    sign: bool,
}

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
            continue;
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
            println!(
                "c parsed 'p cnf {} {}' header",
                formula.variables, _clauses_count
            );
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

fn print_cnf(formula: &CNFFormula) {
    println!("p cnf {} {}", formula.variables, formula.clauses.len());
    for clause in &formula.clauses {
        for literal in clause {
            print!("{} ", literal);
        }
        println!("0");
    }
}

fn main() {
    let matches = App::new("BabySub")
        .version("1.0")
        .author("Bernhard Gstrein")
        .about("Processes and simplifies logical formulae in DIMACS CNF format.")
        .arg(
            Arg::with_name("input")
                .help("Sets the input file to use")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("Sets the output file to use")
                .required(false)
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

    println!("c BabySub Subsumption Preprocessor");
    println!("c reading from {}", config.input_path);

    let formula = parse_cnf(&config.input_path).unwrap_or_else(|err| {
        eprintln!("Failed to parse CNF file: {}", err);
        process::exit(1);
    });

    if config.sign {
        let signature = formula.compute_signature();
        println!("c hash-signature: {}", signature);
    }

    if config.verbosity > 0 {
        print_cnf(&formula);
    }
}
