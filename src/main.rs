use clap::{App, Arg};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::process;

// TODO: Remove the unwrap calls and handle errors properly.

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

struct SATContext {
    config: Config,
    formula: CNFFormula,
    writer: BufWriter<Box<dyn Write>>,
    reader: BufReader<Box<dyn Read>>,
}

impl SATContext {
    fn new(config: Config) -> Self {
        let output: Box<dyn Write> = match config.output_path.as_str() {
            "" => Box::new(io::stdout()),
            path => Box::new(File::create(path).expect("Failed to create output file")),
        };

        let input: Box<dyn Read> = match config.input_path.as_str() {
            "" => Box::new(io::stdin()),
            path => Box::new(File::open(path).expect("Failed to open input file")),
        };

        SATContext {
            config,
            formula: CNFFormula::new(),
            writer: BufWriter::new(output),
            reader: BufReader::new(input),
        }
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

fn parse_cnf(ctx: &mut SATContext) -> io::Result<()> {
    let mut current_clause_index = 0;
    let mut header_parsed = false;

    while let Some(line) = ctx.reader.by_ref().lines().next() {
        let line = line?;
        if line.starts_with('c') {
            continue; // Skip comment lines
        }
        if line.starts_with("p cnf") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                writeln!(ctx.writer, "Error: Invalid header format.")?;
                process::exit(1);
            }
            ctx.formula.variables = parts[2].parse().unwrap_or_else(|_| {
                writeln!(ctx.writer, "Error: Invalid number of variables.").unwrap();
                process::exit(1);
            });
            let _clauses_count: usize = parts[3].parse().unwrap_or_else(|_| {
                writeln!(ctx.writer, "Error: Invalid number of clauses.").unwrap();
                process::exit(1);
            });
            header_parsed = true;
            writeln!(
                ctx.writer,
                "c parsed 'p cnf {} {}' header",
                ctx.formula.variables, _clauses_count
            )?;
        } else if header_parsed {
            let clause: Vec<i32> = line
                .split_whitespace()
                .map(|num| {
                    num.parse().unwrap_or_else(|_| {
                        writeln!(ctx.writer, "Error: Invalid literal format.").unwrap();
                        process::exit(1);
                    })
                })
                .filter(|&x| x != 0)
                .collect();
            ctx.formula.add_clause(clause, current_clause_index);
            current_clause_index += 1;
        } else {
            writeln!(ctx.writer, "Error: CNF header not found.")?;
            process::exit(1);
        }
    }
    Ok(())
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
        input_path: matches.value_of("input").unwrap_or_default().to_string(),
        output_path: matches.value_of("output").unwrap_or_default().to_string(),
        verbosity: matches.occurrences_of("verbosity") as i32
            - matches.occurrences_of("quiet") as i32,
        sign: matches.is_present("sign"),
    };

    let mut ctx = SATContext::new(config);

    writeln!(ctx.writer, "c BabySub Subsumption Preprocessor").unwrap();
    writeln!(ctx.writer, "c reading from {}", ctx.config.input_path).unwrap();

    parse_cnf(&mut ctx).unwrap_or_else(|err| {
        eprintln!("Failed to parse CNF: {}", err);
        process::exit(1);
    });

    if ctx.config.sign {
        let signature = ctx.formula.compute_signature();
        writeln!(ctx.writer, "c hash-signature: {}", signature).unwrap();
    }

    if ctx.config.verbosity > 0 {
        print_cnf(&ctx.formula);
    }
}
