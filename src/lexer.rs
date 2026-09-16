//! Parses Metamath databases.
//!
//! A database is an input file, which may have commands to include other input files.
//! Each file is further parsed into scopes and statements.
//!
//! If [`Token`]s are like Words, {{`SyntaxProduction`}}s are like the various kinds of increasingly complex
//! phrases, sentences, paragraphs, pages, chapters, sections, volumes, and arcs that make up the story
//! that is a database.
//!
//! If there is a {{`FileInclusion`}}, the lexer will stop, and load the {{`SyntaxProduction`}}s
//! of that file in-place. It is currently unspecified, so the filepath of the {{`FileInclusion`}}
//! will be relative to whatever is convenient, probably the current-working-directory.
//!
//! [`Token`]: Token

use logos::Logos;
use std::ops::Range;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Span(Range<usize>);

impl Span {
    fn new(range: Range<usize>) -> Self {
        Self(range)
    }

    fn as_range(&self) -> &Range<usize> {
        &self.0
    }

    fn into_range(self) -> Range<usize> {
        self.0
    }

    fn start(&self) -> usize {
        self.0.start
    }

    fn end(&self) -> usize {
        self.0.end
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn contains(&self, index: usize) -> bool {
        self.0.contains(&index)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self(range)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.into_range()
    }
}

impl AsRef<Range<usize>> for Span {
    fn as_ref(&self) -> &Range<usize> {
        self.as_range()
    }
}

impl std::ops::Deref for Span {
    type Target = Range<usize>;

    fn deref(&self) -> &Self::Target {
        self.as_range()
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
struct InvalidToken(String, Span);

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(error(InvalidToken, callback = |lex| InvalidToken(lex.slice().to_owned(), Span(lex.span()))))]
#[logos(skip r"[[:space:]]+")] // Ignore ASCII whitespace
/// A space separated unit of Metamath.
///
/// # Warning
///
/// There exists lexical ambiguity:
/// (`Token`) enum variants are accepted as representing themselves OR any more general variant.
///
/// In other words, a specific token is allowed to be parsed in a general category.
///
/// See [`satisfies`].
///
/// [`satisfies`]: Token::satisfies
enum Token {
    #[token("$(", priority = 5)]
    CommentStart,

    #[token("$)", priority = 5)]
    CommentEnd,

    #[token("$[", priority = 5)]
    FileInclusionStart,

    #[token("$]", priority = 5)]
    FileInclusionEnd,

    // metamath.pdf calls this a "block",
    // but "scope" is more intuitive imo.
    #[token("${", priority = 5)]
    ScopeStart,

    #[token("$}", priority = 5)]
    ScopeEnd,

    #[token("$c", priority = 5)]
    ConstantDeclarationStart,

    #[token("$v", priority = 5)]
    VariableDeclarationStart,

    #[token("$d", priority = 5)]
    DvConditionStart,

    #[token("$f", priority = 5)]
    FloatingHypothesisStart,

    #[token("$e", priority = 5)]
    EssentialHypothesisStart,

    #[token("$a", priority = 5)]
    AxiomStart,

    #[token("$p", priority = 5)]
    ProofStart,

    #[token("$=", priority = 5)]
    ProofDetailsStart,

    #[token("$.", priority = 5)]
    ItemEnd,

    // Due to the combination of:
    // 1. "AAA" cannot be three separate tokens as `Label` matches the whole thing
    // 1. "A A" cannot be a single token because `MathSymbol` might need them to be separate
    //
    // we can only process compressed proofs in "chunks".
    //
    // # Possible interpretations
    //
    // - A valid number: `A`
    // - An invalid number: `U`
    // - Multiple numbers: `AA`
    // - A combination of these: `AAUZ`
    #[regex(r"[A-Z]+", priority = 4)]
    CompressedChunkLabelCompatible,

    #[regex(r"[A-Z?]+", priority = 3)]
    CompressedChunkLabelIncompatible,

    #[regex(r"[A-Za-z0-9._\-]+", priority = 2)]
    Label,

    #[regex("[!-#%-~]+", priority = 1)]
    MathSymbol,

    // This won't include "$)" because `CommentEnd` has higher priotity.
    #[regex("[!-~]+", priority = 0)]
    CommentPart,
}

impl Token {
    /// Return an iterator over the keywords (the `$` [`Token`]s) in a string.
    ///
    /// # Why
    ///
    /// It would be useful to give a hint like so:
    /// ```text
    /// error: expected a statement
    ///   --> file.mm:42:4
    ///    |
    /// 42 | foo$a bar
    ///    |    ^^
    ///    |
    ///    = note: `$a` appears to be attached to another token
    ///    = help: separate `$a` from the preceding text with whitespace
    /// ```
    ///
    /// [`Token`]: Token
    fn find_keywords_in(s: &str) -> impl Iterator<Item = Span> {
        s.match_indices('$').filter_map(|(i, _)| {
            let next = *s.as_bytes().get(i + 1)?;
            matches!(
                next,
                b'(' | b')'
                    | b'['
                    | b']'
                    | b'{'
                    | b'}'
                    | b'c'
                    | b'v'
                    | b'd'
                    | b'f'
                    | b'e'
                    | b'a'
                    | b'p'
                    | b'='
                    | b'.'
            )
            .then_some(Span(i..i + 2))
        })
    }

    /// Returns true if this token matches or can be interpreted/consumed as another token.
    fn satisfies(self, expected: Self) -> bool {
        self == expected
            || matches!(
                (self, expected),
                (
                    Self::CommentStart
                        | Self::FileInclusionStart
                        | Self::FileInclusionEnd
                        | Self::ScopeStart
                        | Self::ScopeEnd
                        | Self::ConstantDeclarationStart
                        | Self::VariableDeclarationStart
                        | Self::DvConditionStart
                        | Self::FloatingHypothesisStart
                        | Self::EssentialHypothesisStart
                        | Self::AxiomStart
                        | Self::ProofStart
                        | Self::ProofDetailsStart
                        | Self::ItemEnd
                        | Self::CompressedChunkLabelCompatible
                        | Self::CompressedChunkLabelIncompatible
                        | Self::Label
                        | Self::MathSymbol,
                    Self::CommentPart // Everything except CommentEnd
                ) | (
                    Self::CompressedChunkLabelCompatible
                        | Self::CompressedChunkLabelIncompatible
                        | Self::Label,
                    Self::MathSymbol
                ) | (Self::CompressedChunkLabelCompatible, Self::Label)
            )
    }
}

#[cfg(test)]
mod lexing_tests {
    use super::*;

    #[test]
    fn basic_lexing() {
        assert_eq!(
            Token::lexer("$( hi! $. $(( ZU\n AAU Zap").collect::<Vec<_>>(),
            vec![
                Ok(Token::CommentStart),                   // $(
                Ok(Token::MathSymbol),                     // hi!
                Ok(Token::ItemEnd),                        // $.
                Ok(Token::CommentPart),                    // $((
                Ok(Token::CompressedChunkLabelCompatible), // ZU
                Ok(Token::CompressedChunkLabelCompatible), // AAU
                Ok(Token::Label),                          // Zap
            ]
        );
    }

    #[test]
    fn invalid_lex() {
        assert_eq!(
            Token::lexer("miku miku biiiimu！ \n[...]").collect::<Vec<_>>(),
            vec![
                Ok(Token::Label),
                Ok(Token::Label),
                Ok(Token::Label),
                Err(InvalidToken(String::from("！"), 17..20)),
                Ok(Token::MathSymbol),
            ]
        );
    }
}
