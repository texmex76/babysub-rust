use clap::{App, Arg, ArgAction};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::process;
use std::time::Instant;

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

struct Stats {
    // added: usize, // In Prof. Biere's code, this does not do anything, so I ommited it.
    checked: usize,
    parsed: usize,
    subsumed: usize,
    start_time: Instant,
}

struct SATContext {
    config: Config,
    formula: CNFFormula,
    writer: BufWriter<Box<dyn Write>>,
    stats: Stats,
}

impl SATContext {
    fn new(config: Config) -> Self {
        let output: Box<dyn Write> = match config.output_path.as_str() {
            "" => Box::new(io::stdout()),
            path => Box::new(File::create(path).expect("Failed to create output file")),
        };

        SATContext {
            config,
            formula: CNFFormula::new(),
            writer: BufWriter::new(output),
            stats: Stats {
                // added: 0,
                checked: 0,
                parsed: 0,
                subsumed: 0,
                start_time: Instant::now(),
            },
        }
    }
}

macro_rules! message {
    ($ctx:expr, $($arg:tt)*) => {{
        if $ctx.config.verbosity >= 0 {
            use std::io::Write;  // Import the Write trait to access the flush method

            // Write the formatted message to the writer
            writeln!($ctx.writer, "{}", format!("c {}", format_args!($($arg)*))).unwrap();

            // Flush the writer to ensure the output is immediately visible
            $ctx.writer.flush().unwrap();
        }
    }}
}

macro_rules! raw_message {
    ($ctx:expr, $($arg:tt)*) => {{
        if $ctx.config.verbosity >= 0 {
            use std::io::Write;  // Import the Write trait to access the flush method

            // Write the formatted message directly to the writer without prefix
            writeln!($ctx.writer, "{}", format_args!($($arg)*)).unwrap();

            // Flush the writer to ensure the output is immediately visible
            $ctx.writer.flush().unwrap();
        }
    }}
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

fn average(a: usize, b: usize) -> f64 {
    if b != 0 {
        a as f64 / b as f64
    } else {
        0.0
    }
}

fn percent(a: usize, b: usize) -> f64 {
    100.0 * average(a, b)
}

fn report_stats(ctx: &mut SATContext) {
    let elapsed_time = ctx.stats.start_time.elapsed().as_secs_f64();
    message!(
        ctx,
        "{:<20} {:>10}    clauses {:.2} per subsumed",
        "checked:",
        ctx.stats.checked,
        average(ctx.stats.subsumed, ctx.stats.subsumed)
    );
    message!(
        ctx,
        "{:<20} {:>10}    clauses {:.0}%",
        "subsumed:",
        ctx.stats.subsumed,
        percent(ctx.stats.subsumed, ctx.stats.parsed)
    );
    message!(ctx, "{:<20} {:13.2} seconds", "process-time:", elapsed_time);
}

macro_rules! parse_error {
    ($ctx:expr, $msg:expr, $line:expr) => {{
        eprintln!(
            "babysub: parse error: at line {} in '{}': {}",
            $line, $ctx.config.input_path, $msg
        );
        process::exit(1);
    }};
}

fn parse_cnf(input_path: String, ctx: &mut SATContext) -> io::Result<()> {
    let input: Box<dyn Read> = if input_path.is_empty() {
        message!(ctx, "reading from '<stdin>'");
        Box::new(io::stdin())
    } else {
        message!(ctx, "reading from '{}'", input_path);
        Box::new(File::open(&input_path)?)
    };

    let reader = BufReader::new(input);
    let mut current_clause_index = 0;
    let mut header_parsed = false;
    let mut clauses_count = 0;
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        if line.starts_with('c') {
            continue; // Skip comment lines
        }
        if line.starts_with("p cnf") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                parse_error!(ctx, "Invalid header format.", line_number);
            }
            ctx.formula.variables = parts[2].parse().unwrap_or_else(|_| {
                parse_error!(ctx, "Could not read number of variables.", line_number);
            });
            clauses_count = parts[3].parse().unwrap_or_else(|_| {
                parse_error!(ctx, "Could not read number of clauses.", line_number);
            });
            header_parsed = true;
            message!(
                ctx,
                "parsed 'p cnf {} {}' header",
                ctx.formula.variables,
                clauses_count
            );
        } else if header_parsed {
            let clause: Vec<i32> = line
                .split_whitespace()
                .map(|num| {
                    num.parse().unwrap_or_else(|_| {
                        parse_error!(ctx, "Invalid literal format.", line_number);
                    })
                })
                .filter(|&x| x != 0)
                .collect();
            ctx.formula.add_clause(clause, current_clause_index);
            current_clause_index += 1;
            ctx.stats.parsed += 1;
        } else {
            parse_error!(ctx, "CNF header not found.", line_number);
        }
    }
    if clauses_count != ctx.stats.parsed {
        parse_error!(
            ctx,
            format!(
                "Mismatch in declared and parsed clauses: expected {}, got {}",
                clauses_count, ctx.stats.parsed
            ),
            line_number
        );
    }
    Ok(())
}

fn print_cnf(ctx: &mut SATContext) {
    if ctx.config.sign {
        let signature = ctx.formula.compute_signature();
        message!(ctx, "hash-signature: {}", signature);
    }

    raw_message!(
        ctx,
        "p cnf {} {}",
        ctx.formula.variables,
        ctx.formula.clauses.len()
    );
    for clause in &ctx.formula.clauses {
        let clause_string = clause
            .iter()
            .map(|&lit| lit.to_string())
            .collect::<Vec<String>>()
            .join(" ")
            + " 0";
        raw_message!(ctx, "{}", clause_string);
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
                .action(ArgAction::Count)
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

    let quiet = matches.is_present("quiet");
    let verbosity = if quiet {
        -1
    } else {
        *matches.get_one::<u8>("verbosity").unwrap_or(&0) as i32
    };
    let config = Config {
        input_path: matches.value_of("input").unwrap_or_default().to_string(),
        output_path: matches.value_of("output").unwrap_or_default().to_string(),
        verbosity,
        sign: matches.is_present("sign"),
    };

    let mut ctx = SATContext::new(config);
    message!(&mut ctx, "BabySub Subsumption Preprocessor");

    if let Err(e) = parse_cnf(ctx.config.input_path.clone(), &mut ctx) {
        eprintln!("Failed to parse CNF: {}", e);
        process::exit(1);
    }

    print_cnf(&mut ctx);
    report_stats(&mut ctx);
}
