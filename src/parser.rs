//! Parses Metamath databases.
//!
//! A database is an input file, which may have commands to include other input files.
//! Each file is further parsed into scopes and statements.
//!
//! If {`Token`}s are like Words, {{`SyntaxProduction`}}s are like the various kinds of increasingly complex
//! phrases, sentences, paragraphs, pages, chapters, sections, volumes, and arcs that make up the story
//! that is a database.
//!
//! If there is a {{`FileInclusion`}}, the lexer will stop, and load the {{`SyntaxProduction`}}s
//! of that file in-place. It is currently unspecified, so the filepath of the {{`FileInclusion`}}
//! will be relative to whatever is convenient, probably the current-working-directory.

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[[:space:]]+")] // Ignore ASCII whitespace
enum Token {
    #[token("$(", priority = 4)]
    CommentStart,

    #[token("$)", priority = 4)]
    CommentEnd,

    #[token("$[", priority = 4)]
    FileInclusionStart,

    #[token("$]", priority = 4)]
    FileInclusionEnd,

    // metamath.pdf calls this a "block",
    // but "scope" is more intuitive imo.
    #[token("${", priority = 4)]
    ScopeStart,

    #[token("$}", priority = 4)]
    ScopeEnd,

    #[token("$c", priority = 4)]
    ConstantDeclarationStart,

    #[token("$v", priority = 4)]
    VariableDeclarationStart,

    #[token("$d", priority = 4)]
    DVConditionStart,

    #[token("$f", priority = 4)]
    FloatingHypothesisStart,

    #[token("$e", priority = 4)]
    EssentialHypothesisStart,

    #[token("$a", priority = 4)]
    AxiomStart,

    #[token("$p", priority = 4)]
    ProofStart,

    #[token("$=", priority = 4)]
    ProofDetailsStart,

    #[token("$.", priority = 4)]
    ItemEnd,

    #[token("?", priority = 4)]
    MissingCompressedStep,

    // Thankfully, this does not capture "Z" in the midst of a word,
    // see the `basic_lexing` test below.
    #[token("Z", priority = 4)]
    RecallCompressedStep,

    // Instead of a callback which interrupts blazing-fast lexing,
    // we convert this into a number upon actually parsing, for speed.
    #[regex(r"[U-Y[:space:]]*[A-T]", priority = 3)]
    CompressedStep,

    #[regex(r"[A-Za-z0-9._\-]+", priority = 2)]
    Label,

    #[regex("[!-#%-~]+", priority = 1)]
    MathSymbol,

    #[regex("[!-~]+", priority = 0)]
    CommentPart,
}

#[cfg(test)]
mod lexing_tests {
    use super::*;

    #[test]
    fn basic_lexing() {
        assert_eq!(
            Token::lexer("$( hi! $. $(( Z U\n A Zap").collect::<Vec<_>>(),
            vec![
                Ok(Token::CommentStart),         // $(
                Ok(Token::MathSymbol),           // hi!
                Ok(Token::ItemEnd),              // $.
                Ok(Token::CommentPart),          // $((
                Ok(Token::RecallCompressedStep), // Z
                Ok(Token::CompressedStep),       // U\n A
                Ok(Token::Label),                // Zap
            ]
        )
    }
}
