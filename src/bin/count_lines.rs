//! A simple CLI utility that counts the lines in a given file
//!
//! If the file ends in a newline, it is separated from the total line count;
//! i.e. `<lines excluding last newline>+1` is output.

use clap::Parser;

#[derive(Parser)]
struct Cli {
    file_path: std::path::PathBuf,
}

fn main() {
    let Cli { file_path } = Cli::parse();

    let contents = std::fs::read_to_string(file_path).expect("Should have been able to read the file");

    let lines = contents.lines().count();

    if contents.ends_with('\n') {
        println!("The file has {lines}+1 lines");
    } else {
        println!("The file has {lines} lines");
    }
}
