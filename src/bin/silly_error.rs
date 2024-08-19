//! Provides a user-friendly error if the given file has the string `tab`
//!
//! A silly utility to test [`annotate_snippets`](https://crates.io/crates/annotate-snippets)
//! (and [`anstream`](https://crates.io/crates/anstream))
//!
//! # Example
//!
//! If you've cloned the repo, you can try:
//!
//! ```bash
//! cargo run --bin silly_error -- README.md
//! ```

use annotate_snippets::{Level, Renderer, Snippet};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    file_path: std::path::PathBuf,
}

fn main() {
    let Cli { file_path } = Cli::parse();
    let path_string = file_path.to_string_lossy().to_string();

    // Bad error message; don't do this!
    let contents =
        std::fs::read_to_string(file_path).expect("Should have been able to read the file");

    let renderer = Renderer::styled();
    let message = if let Some(index) = contents.find("tab") {
        Level::Error
            .title("no tabs allowed!")
            .snippet(
                Snippet::source(&contents)
                    .line_start(1)
                    .origin(&path_string)
                    .fold(true)
                    .annotation(
                        Level::Error
                            .span(index..(index + 3))
                            .label("Found `tab` here"),
                    )
                    .annotation(Level::Help.span(index..(index + 3)).label("remove `tab`"))
                    .annotation(Level::Warning.span(index..(index + 3)).label("Warning"))
                    .annotation(Level::Info.span(index..(index + 3)).label("you're great"))
                    .annotation(Level::Note.span(index..(index + 3)).label("a")),
            )
            .footer(Level::Info.title("This utility is silly"))
            .footer(Level::Note.title("!"))
    } else {
        Level::Info.title("No \"tabs\" found, congrats!")
    };
    anstream::println!("{}", renderer.render(message));
}
