Message Handling (message, verbose, die, error):
message and verbose functions are used for printing output based on verbosity levels.
die prints an error message and exits the program, typically used for fatal errors.
error is similar but used specifically for parsing errors.

Logging (LOG):
Provided under conditional compilation (#ifdef LOGGING). It logs detailed debug information if the verbosity level is set to maximum, which is useful for development or detailed debugging.

Formula Simplification (simplify):
Copies original clauses to a simplified list and populates a matrix for further processing.

Signature Calculation (signature):
If enabled, calculates a hash-based signature of the simplified formula to ensure integrity or for other verification purposes.

Output Handling (print, print function):
Manages opening the output file or setting it to stdout.
Outputs the simplified CNF formula and, if requested, the signature.

Performance Reporting (report):
Reports statistics like the number of subsumed clauses and processing time.

Dynamic Memory: Uses dynamic memory allocation for storing clauses and literals, which it properly cleans up to prevent memory leaks.

Performance Metrics: Utilizes system-specific calls to measure and report on computation time and efficiency.

Logging and Debugging: Supports detailed logging for debugging purposes, though this is optional and depends on compile-time settings.


# Done

# 1

Usage Information (usage variable):
Describes how to use the program, including its options and expected input and output formats.

Includes and Global Variables:
Includes various headers necessary for handling I/O, data manipulation, and system-specific functionality.
Defines global variables for managing program state, such as input/output files, verbosity, statistics, and data structures for clauses.

Command Line Options Parsing (options):
Parses and handles command-line options to configure the program’s behavior (verbosity, logging, input/output files, and signature).

Main Function (main):
Orchestrates the flow of the program by parsing options, initializing the environment, processing the input, and handling outputs.

# 2

Parsing Input (parse):
Handles opening the input file or setting it to stdin.
Reads and parses the CNF header and each clause, storing them in global structures.
Errors in format or content during parsing lead to immediate termination with an error message.

Initialization and Cleanup (init, reset):
init sets up data structures needed for processing the CNF formula.
reset cleans up these structures to prevent memory leaks.
