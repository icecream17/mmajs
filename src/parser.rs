//! Parses Metamath databases.
//!
//! The parser starts with an input file, which may have commands to include other input files.
//! This is called a database. Each file is further parsed into scopes and statements.
//!
//! If Tokens are like Words, SyntaxProductions are like the various kinds of increasingly complex
//! phrases, sentences, paragraphs, sections, pages, chapters, volumes, and arcs that make up the story
//! that is a database.
//!
//! If there is a FileInclusion, the lexer will stop, and load the SyntaxProductions
//! of that file in-place. It is currently unspecified, but for consistency, the filepath of the {FileInclusion}
//! will be relative to the file including it.

use std::path::PathBuf;

// While holding the whole String may be worse than other options, `count_lines.rs` indicates
// that reading then holding 772780 lines in a single String can be done in about no time (wow)
// As such, there's no reason to optimize.
struct File<'a> {
    path: &'a PathBuf,
    text: String,
}

impl<'a> File<'a> {
    fn try_new(path: &'a PathBuf) -> Result<Self, std::io::Error> {
        // Long code: see bottom of file
        match std::fs::read_to_string(path) {
            Ok(text) => Ok(File { path, text }),
            Err(e) => Err(e),
        }
    }
}

enum Keyword {
    /// `$(`
    CommentStart,
    /// `$)`
    CommentEnd,
    /// `$[`
    FileInclusionStart,
    /// `$]`
    FileInclusionEnd,
    /// `${`
    ScopeStart,
    /// `$}`
    ScopeEnd,
    /// `$c`
    ConstantDeclarationStart,
    /// `$v`
    VariableDeclarationStart,
    /// `$d`
    DVConditionStart,
    /// `$a`
    AxiomaticAssertionStart,
    /// `$p`
    ProvableAssertionStart,
    /// `$=`
    ProofDetailsStart,
    /// `$.`
    End,
}

impl From<Keyword> for &'static str {
    fn from(value: Keyword) -> Self {
        match value {
            Keyword::CommentStart => "$(",
            Keyword::CommentEnd => "$)",
            Keyword::FileInclusionStart => "$[",
            Keyword::FileInclusionEnd => "$]",
            Keyword::ScopeStart => "${",
            Keyword::ScopeEnd => "$}",
            Keyword::ConstantDeclarationStart => "$c",
            Keyword::VariableDeclarationStart => "$v",
            Keyword::DVConditionStart => "$d",
            Keyword::AxiomaticAssertionStart => "$a",
            Keyword::ProvableAssertionStart => "$p",
            Keyword::ProofDetailsStart => "$=",
            Keyword::End => "$.",
        }
    }
}

// Naming: compare https://doc.rust-lang.org/stable/nightly-rustc/rustc_ast/token/enum.TokenKind.html
//
// Reasons to use "Kind" instead of "Type":
// 1. It allows me to use "kind" as a field. "type" is a keyword in Rust
enum TokenKind {
    /// Starts with `$`
    Keyword(Keyword),

    /// Appears in proofs and before `$f`, `$e`, `$a`, and `$p`
    Label(String),

    /// Appears in expressions
    MathSymbol(String),

    /// Appears in comments
    CommentedLiteral(String),

    /// `(` -- the start of the label list in a compressed proof
    CompressedProofStart,

    /// `)` -- the end of the label list in a compressed proof
    CompressedProofEnd,

    /// Unprocessed capital letter data in a compressed proof
    CompressedProofPart(String),

    /// Custom token inserted after an included file for verification purposes
    Eof,
}

impl From<TokenKind> for &'static str {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Keyword(keyword) => keyword.into(),
            TokenKind::Label(_) => todo!(),
            TokenKind::MathSymbol(_) => todo!(),
            TokenKind::CommentedLiteral(_) => todo!(),
            TokenKind::CompressedProofStart => "(",
            TokenKind::CompressedProofEnd => ")",
            TokenKind::CompressedProofPart(_) => todo!(),
            TokenKind::Eof => "",
        }
    }
}

// NOTE: Lines and Columns are 1-indexed!
// Columns are zero indexed in annotate_snippets (.span)
// Each token holds its line, and start and end column
// Each production holds its tokens, and raw text, and file

/// Semantic type for positions in files. (metamath-rs)
pub type FilePos = usize;

/// Semantic type for file-position ranges. (metamath-rs)
pub type Span = std::ops::Range<usize>;

struct Token {
    kind: TokenKind,

    /// The columns (in the line) where the token lives
    location: Span,
}

struct SyntaxProduction {
    tokens: Vec<Token>,
}

/// A speciic location in a file
struct LexerLocation<'a> {
    /// Relative path to the file
    file: &'a PathBuf,

    ///
    zi_line: usize,

    /// Zero-indexed character
    zi_column: usize,
}

impl<'a> LexerLocation<'a> {
    fn new(file: &'a PathBuf, zi_line: usize, zi_column: usize) -> Self {
        LexerLocation {
            file,
            zi_line,
            zi_column,
        }
    }
}

struct Lexer<'a> {
    /// Necessary to avoid reprocessing the same file (which can cause infinite loops)
    files_included: Vec<File<'a>>,

    /// Location of the next ASCII character to be read, for each file remaining
    progress: Vec<LexerLocation<'a>>,
}

impl<'a> Lexer<'a> {
    fn try_new(path: &'a PathBuf) -> Result<Self, std::io::Error> {
        // Long code: see bottom of file
        match File::try_new(path) {
            Ok(file) => Ok(Lexer {
                files_included: vec![file],
                progress: vec![LexerLocation::new(path, 0, 0)],
            }),
            Err(e) => Err(e),
        }
    }

    fn next_char(&'a mut self) -> u8 {
        unimplemented!();
    }
}

// Further investigation shows that using `?` would be much shorter.
// As an experiment, I generated the assembly code of two simple functions:
// one using `match` and one using `?`:
//
// 1. https://play.rust-lang.org/?version=nightly&mode=release&edition=2021&gist=6d57cbf962e225820d9284f41e567bf5
// 2. https://play.rust-lang.org/?version=nightly&mode=release&edition=2021&gist=2520f4cef3594981c399a2ddb0b8e212
//
// Using Release build + Nightly (2021), the result is that using `?` is (ever so) very slightly less optimal!
// https://www.diffchecker.com/9hUUvWVZ/
//
// I think the reason is that `match` statements can always be very optimized, while `?` is usually used more
// than once in a function (which causes `jne` and `jmp` ASM).
//
// Obviously this explanation is vague to the point of being wrong, I'll leave it as an exercise to the reader
// to give an actual explanation.
//
// I'll probably still use `?`, but I won't bother to update the match statements.
