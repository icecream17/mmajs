//! Converts source text into tokens.

use logos::Logos;
use std::ops::Range;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Span(Range<usize>);

impl Span {
    const fn new(range: Range<usize>) -> Self {
        Self(range)
    }

    const fn as_range(&self) -> &Range<usize> {
        &self.0
    }

    const fn into_range(self) -> Range<usize> {
        self.0
    }

    const fn start(&self) -> usize {
        self.0.start
    }

    const fn end(&self) -> usize {
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
pub(crate) struct InvalidToken(String, Span);

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(error(InvalidToken, callback = |lex| InvalidToken(lex.slice().to_owned(), Span(lex.span()))))]
#[logos(skip r"[[:space:]]+")] // Ignore ASCII whitespace
/// A category of lexical unit in Metamath source text.
///
/// All tokens / token kinds are ASCII.
/// Whitespace is always skipped in the lexer, and is never part of a token.
///
/// # Note
///
/// Some variants are intentionally broad: a token kind may satisfy the requirements
/// of a more general category through [`TokenKind::satisfies`].
///
/// The name of a variant is more specific than the possibilities for its semantic meaning.
///
/// [`TokenKind::satisfies`]: TokenKind::satisfies
pub(crate) enum TokenKind {
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

impl TokenKind {
    /// Return an iterator over the keywords (the `$` [`TokenKind`]s) in a string.
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
    /// [`TokenKind`]: TokenKind
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

#[derive(Debug, PartialEq)]
/// A token occurrence together with its location in the source text.
pub(crate) struct Token<'source> {
    kind: Result<TokenKind, InvalidToken>,

    /// The source text occupied by this token.
    slice: &'source str,

    /// The token's byte range in the source text.
    span: Span,
}

impl Token<'_> {
    // pub(crate) fn kind(&self) -> &Result<TokenKind, InvalidToken> {
    //     &self.kind
    // }

    // pub(crate) fn span(&self) -> &Span {
    //     &self.span
    // }

    pub(crate) fn satisfies(&self, expected: TokenKind) -> bool {
        self.kind.as_ref().is_ok_and(|k| k.satisfies(expected))
    }
}

// This exists so that `lex` returns an explicit type instead of an opaque one,
// which would cause `TokenStream` to have an unwieldy generic <I: Iterator<...>>.
/// The lazy token stream produced from a source string.
pub(crate) struct Tokens<'source> {
    lexer: logos::Lexer<'source, TokenKind>,
}

impl<'source> Iterator for Tokens<'source> {
    type Item = Token<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Token {
            kind: self.lexer.next()?,
            slice: self.lexer.slice(),
            span: Span(self.lexer.span()),
        })
    }
}

/// Lex a source string.
pub(crate) fn lex(source: &str) -> Tokens<'_> {
    Tokens {
        lexer: TokenKind::lexer(source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_lexing() {
        assert_eq!(
            TokenKind::lexer("$( hi! $. $(( ZU\n AAU Zap").collect::<Vec<_>>(),
            vec![
                Ok(TokenKind::CommentStart),                   // $(
                Ok(TokenKind::MathSymbol),                     // hi!
                Ok(TokenKind::ItemEnd),                        // $.
                Ok(TokenKind::CommentPart),                    // $((
                Ok(TokenKind::CompressedChunkLabelCompatible), // ZU
                Ok(TokenKind::CompressedChunkLabelCompatible), // AAU
                Ok(TokenKind::Label),                          // Zap
            ]
        );
    }

    #[test]
    fn invalid_lex() {
        assert_eq!(
            TokenKind::lexer("miku miku biiiimu！ \n[...]").collect::<Vec<_>>(),
            vec![
                Ok(TokenKind::Label),
                Ok(TokenKind::Label),
                Ok(TokenKind::Label),
                Err(InvalidToken(String::from("！"), Span(17..20))),
                Ok(TokenKind::MathSymbol),
            ]
        );
    }

    #[test]
    fn lex_includes_slices_and_spans() {
        assert_eq!(
            lex("$c wff $.").collect::<Vec<_>>(),
            vec![
                Token {
                    kind: Ok(TokenKind::ConstantDeclarationStart),
                    slice: "$c",
                    span: Span(0..2),
                },
                Token {
                    kind: Ok(TokenKind::Label),
                    slice: "wff",
                    span: Span(3..6),
                },
                Token {
                    kind: Ok(TokenKind::ItemEnd),
                    slice: "$.",
                    span: Span(7..9),
                },
            ]
        );
    }

    #[test]
    fn finds_attached_keywords() {
        assert_eq!(
            TokenKind::find_keywords_in("foo$a bar").collect::<Vec<_>>(),
            vec![Span(3..5)]
        );
    }

    #[test]
    fn token_compatibility_is_explicit() {
        assert!(TokenKind::Label.satisfies(TokenKind::MathSymbol));
        assert!(TokenKind::CompressedChunkLabelCompatible.satisfies(TokenKind::Label));
        assert!(!TokenKind::CommentEnd.satisfies(TokenKind::CommentPart));
    }
}
