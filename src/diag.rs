//! Datatypes to represent diagnostics emitted by various checks (metamath-rs)
//!
//! Uses [`annotate_snippets`](https://crates.io/crates/annotate-snippets) for helpful human-readable outputs.

use annotate_snippets::{Level, Renderer};
use anstream::eprintln;

// How to output a message:
// anstream::println!();

// Start a message with level and title:
// Lever::Error.title("message title")

// From C-WORD-ORDER, error types should be consistant, so:
// adjective-noun and verb-noun-error order.

// This exists because a PathBuf is not trivially convertable to a `&str`.
// I experimented and found that the Debug conversion work decently enough, however I have to prove
// code to the snippet. The snippet surrounds the empty string I give with newlines, resulting in useless
// newlines output. So obviously, outputting it manually is better.
pub(crate) fn exit_on_failed_io_read(error: &std::io::Error, path: &std::path::PathBuf) -> ! {
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L24
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L35
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L55
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L91
    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L60
    const ANSI_BOLD_CYAN_PREFIX: &str = "\x1B[1;36m";

    // https://github.com/ogham/rust-ansi-term/blob/ff7eba98d55ad609c7fcc8c7bb0859b37c7545cc/src/ansi.rs#L78
    const ANSI_SUFFIX: &str = "\x1B[0m";

    let renderer = Renderer::styled();
    let title = format!("{error}");
    let message = Level::Error.title(&title);
    eprintln!("{}", renderer.render(message));
    eprintln!("{ANSI_BOLD_CYAN_PREFIX}-->{ANSI_SUFFIX} Could not read the file {path:?}");
    eprintln!("Some possible reasons why:");
    eprintln!("\"The system cannot find the file specified.\" --> The file does not exist. Maybe you made a typo or it was deleted.");
    eprintln!("\"Access is denied.\" --> Self explanatory. Note that this error also happens when you type in a folder instead.");
    eprintln!("\"stream did not contain valid UTF-8\" --> The file is probably some binary file instead of a file that contains text.");

    std::process::exit(1)
}

pub(crate) enum Diagnostic {}
