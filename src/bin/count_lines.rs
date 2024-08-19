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

use annotate_snippets::{Level, Renderer, Snippet};
use anstream::eprintln;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}

// This exists because a PathBuf is not trivially convertable to a `&str`.
// I experimented and found that the Debug conversion work decently enough, however I have to prove
// code to the snippet. The snippet surrounds the empty string I give with newlines, resulting in useless
// newlines output. So obviously, outputting it manually is better. See the same named function in `crate/diag.rs`
fn exit_on_failed_io_read(error: &std::io::Error, path: &PathBuf) -> ! {
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L24
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L35
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L55
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L91
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L60
    const ANSI_BOLD_CYAN_PREFIX: &str = "\x1B[1;36m";

    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L78
    const ANSI_SUFFIX: &str = "\x1B[0m";

    let renderer = Renderer::styled();
    let title = format!("{error:?}");
    let str_path = format!("{path:?}");
    let message = Level::Error
        .title(&title)
        .snippet(Snippet::source("").origin(&str_path));
    eprintln!("{}", renderer.render(message));
    eprintln!("{ANSI_BOLD_CYAN_PREFIX}-->{ANSI_SUFFIX} Error trying to read the file {path:?}");

    std::process::exit(1)
}

fn main() {
    let Cli { file_path } = Cli::parse();

    let contents = match std::fs::read_to_string(&file_path) {
        Ok(s) => s,
        Err(e) => exit_on_failed_io_read(&e, &file_path),
    };

    // Ignores trailing newline
    let lines = contents.lines().count();

    // Account for trailing newline
    if contents.ends_with('\n') {
        println!("The file has {lines}+1 lines");
    } else {
        println!("The file has {lines} lines");
    }
}
