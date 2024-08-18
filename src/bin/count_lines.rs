//! A simple CLI utility that counts the lines in a given file
//!
//! If the file ends in a newline, it is separated from the total line count;
//! i.e. `<lines excluding last newline>+1` is output.

// This exists to experiment with clap and file handling. This small binary
// helped me write the actual clap and file handling, because I am rusty _and_ a relative beginner.
//
// Discarded ideas:
// 1. Useful exit codes (rejected: Rejected from clap since the usefulness is debatable;
//    there are already codes in the message)
// 1. Read command line arguments from stdin (rejected: Off topic)

use clap::Parser;

#[derive(Parser)]
struct Cli {
    file_path: std::path::PathBuf,
}

fn main() {
    let Cli { file_path } = Cli::parse();

    // Bad error message; don't do this!
    let contents =
        std::fs::read_to_string(file_path).expect("Should have been able to read the file");

    // Ignores trailing newline
    let lines = contents.lines().count();

    // Account for trailing newline
    if contents.ends_with('\n') {
        println!("The file has {lines}+1 lines");
    } else {
        println!("The file has {lines} lines");
    }
}
