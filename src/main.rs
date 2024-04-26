use clap::{Arg, ArgAction, Command};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::process;
use std::time::Instant;

// TODO: Add IDs to clauses. For that, promtp ChatGPT and ask it to discuss a separate clause
// struct vs having what we have now.

struct CNFFormula {
    variables: usize,
    clauses: Vec<Vec<i32>>,
    literal_map: HashMap<i32, Vec<usize>>,
}

impl CNFFormula {
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
            "<stdout>" => Box::new(io::stdout()),
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

macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!("babysub: error: {}", format!($($arg)*));
        process::exit(1);
    }}
}

macro_rules! message {
    ($ctx:expr, $($arg:tt)*) => {{
        if $ctx.config.verbosity >= 0 {
            use std::io::Write;
            if let Err(e) = writeln!($ctx.writer, "{}", format!("c {}", format_args!($($arg)*))) {
                die!("Failed to write message: {}", e);
            }
            if let Err(f) = $ctx.writer.flush() {
                die!("Failed to flush writer: {}", f);
        }}
    }}
}

macro_rules! raw_message {
    ($ctx:expr, $($arg:tt)*) => {{
        if $ctx.config.verbosity >= 0 {
            use std::io::Write;
            if let Err(e) = writeln!($ctx.writer, "{}", format_args!($($arg)*)) {
                die!("Failed to write message: {}", e);
            }
            if let Err(e) = $ctx.writer.flush() {
                die!("Failed to flush writer: {}", e);
            }
        }
    }}
}

macro_rules! verbose {
    ($ctx:expr, $level:expr, $($arg:tt)*) => {{
        if $ctx.config.verbosity == $level {
            use std::io::Write;
            if let Err(e) = writeln!($ctx.writer, "{}", format!("c {}", format_args!($($arg)*))) {
                die!("Failed to write message: {}", e);
            }
            if let Err(f) = $ctx.writer.flush() {
                die!("Failed to flush writer: {}", f);
        }}
    }}
}

#[cfg(feature = "logging")]
macro_rules! LOG {
    ($($arg:tt)*) => {{
        println!("c LOG {}", format_args!($($arg)*));
    }};
}

#[cfg(not(feature = "logging"))]
macro_rules! LOG {
    ($($arg:tt)*) => {{}};
}

fn compute_signature(ctx: &mut SATContext) -> u64 {
    verbose!(ctx, 1, "computing hash-signature");
    let nonces = [
        71876167, 708592741, 1483128881, 907283241, 442951013, 537146759, 1366999021, 1854614941,
        647800535, 53523743, 783815875, 1643643143, 682599717, 291474505, 229233697, 1633529763,
    ];
    let mut hash: u64 = 0;

    for clause in &ctx.formula.clauses {
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

fn simplify(ctx: &mut SATContext) {
    verbose!(ctx, 1, "starting to simplify formula");
}

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
    let input: Box<dyn Read> = if input_path == "<stdin>" {
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
    verbose!(ctx, 1, "parsed {} clauses", ctx.stats.parsed);
    Ok(())
}

fn print(ctx: &mut SATContext) {
    verbose!(ctx, 1, "writing to '{}'", ctx.config.output_path);
    if ctx.config.sign {
        let signature = compute_signature(ctx);
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
    let app = Command::new("BabySub")
        .version("1.0")
        .author("Bernhard Gstrein")
        .about("Processes and simplifies logical formulae in DIMACS CNF format.")
        .arg(
            Arg::new("input")
                .help("Sets the input file to use")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .help("Sets the output file to use")
                .index(2),
        )
        .arg(
            Arg::new("verbosity")
                .short('v')
                .action(ArgAction::Count)
                .help("Increases verbosity level"),
        )
        .arg(Arg::new("quiet").short('q').help("Suppresses all output"))
        .arg(
            Arg::new("sign")
                .short('s')
                .help("Computes and adds a hash signature to the output"),
        );

    #[cfg(feature = "logging")]
    let app = app.arg(
        Arg::new("logging")
            .short('l')
            .help("Enables detailed logging for debugging")
            .action(ArgAction::SetTrue),
    );

    let matches = app.get_matches();

    #[cfg(not(feature = "logging"))]
    let verbosity = if matches.is_present("quiet") {
        -1
    } else {
        *matches.get_one::<u8>("verbosity").unwrap_or(&0) as i32
    };

    #[cfg(feature = "logging")]
    let verbosity = if quiet {
        -1
    } else if matches.get_flag("logging") {
        999
    } else {
        *matches.get_one::<u8>("verbosity").unwrap_or(&0) as i32
    };

    let config = Config {
        input_path: matches.value_of("input").unwrap_or("<stdin>").to_string(),
        output_path: matches.value_of("output").unwrap_or("<stdout>").to_string(),
        verbosity,
        sign: matches.is_present("sign"),
    };

    let mut ctx = SATContext::new(config);
    message!(&mut ctx, "BabySub Subsumption Preprocessor");

    if let Err(e) = parse_cnf(ctx.config.input_path.clone(), &mut ctx) {
        die!("Failed to parse CNF: {}", e);
    }

    print(&mut ctx);
    report_stats(&mut ctx);
}
