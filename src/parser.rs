//! Parses text into statements.

// https://github.com/rust-lang/rust/blob/2699985190ef9e02089bb0f611c7ae0913085cc6/library/core/src/iter/adapters/peekable.rs
//
// The following are adapted from rustlib's core/src/iter/adapters/peekable.rs:
// - TokenStream::peeked and its comment
// - TokenStream::peek
// - TokenStream::bump
//
// which is Copyright (c) The Rust Project Contributors.
// Licensed under either the MIT or Apache-2.0 license, at your option.

use crate::lexer::{InvalidToken, Token, TokenKind, Tokens, has_duplicate, lex};

// Since this is only used internally in a limited and specific way,
// it's ok to break the 0, 1, or infinity rule for speed and clarity.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OneOrTwoOf<T> {
    One(T),
    Two(T, T),
}

struct UnexpectedTokenError<'source, 'b> {
    received: Option<&'b Token<'source>>,
    expected: OneOrTwoOf<TokenKind>,
}

struct TokenStream<'source> {
    // source: &'source str,
    lexer: Tokens<'source>,

    #[expect(clippy::option_option, reason = "Copied from rustc")]
    /// Remember a peeked value, even if it was None.
    peeked: Option<Option<Token<'source>>>,
}

impl<'source> TokenStream<'source> {
    fn new(source: &'source str) -> Self {
        TokenStream {
            // source,
            lexer: lex(source),
            peeked: None,
        }
    }

    // Unfortunately, we cannot use [`Iterator::peekable`] because we need
    // access to the collected [`InvalidToken`]s from [`Tokens::take_invalid`].
    //
    // So the methods come **copied from rust source** (see top of file for license).
    #[inline]
    fn peek(&mut self) -> Option<&Token<'source>> {
        let iter = &mut self.lexer;

        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    #[inline]
    fn bump(&mut self) -> Option<Token<'source>> {
        match self.peeked.take() {
            Some(v) => v,
            None => self.lexer.next(),
        }
    }

    /// Consume a token only when it satisfies the expected [`TokenKind`],
    /// otherwise it fails and doesn't consume anything.
    fn expect<'b>(
        &'b mut self,
        expected: TokenKind,
    ) -> Result<Token<'source>, UnexpectedTokenError<'source, 'b>> {
        match self.bump() {
            Some(v) if v.satisfies(expected) => Ok(v),
            _ => Err(UnexpectedTokenError {
                received: self.peek(),
                expected: OneOrTwoOf::One(expected),
            }),
        }
    }

    fn expect_either(
        &mut self,
        one: TokenKind,
        two: TokenKind,
    ) -> Result<Token<'_>, UnexpectedTokenError<'_, '_>> {
        match self.bump() {
            Some(v) if v.satisfies(one) || v.satisfies(two) => Ok(v),
            _ => Err(UnexpectedTokenError {
                received: self.peek(),
                expected: OneOrTwoOf::Two(one, two),
            }),
        }
    }

    fn finish(self) -> Vec<InvalidToken<'source>> {
        self.lexer.take_invalid()
    }
}

/// <explain what exactly this is>
///
/// Each state either applies a match expression to a peeked token
/// or just expects a single possibility or two.
///
/// - Upon success it transitions into another [`ParserState`]
/// - Upon failure it transitions into a [`FailedParserState`]
///
/// The key unit that a parser returns is a `Statement`. A `Statement` has multiple
/// [`Token`]s or something. [`ParserState`]s relations with a `Statement` are categorized:
///
/// - Begin `Statement`
/// - Add token to be given to the `Statement`
/// - Give the `Statement` the information, because it ended (can fail)
///
/// The `Parser` will handle this because it can fail. `Statement` is presumably a trait.
/// [`ParserState::Label`] shows that information can start collecting even before the
/// `Statement` is begun. `Statement`s presumably belong to a certain `Scope`; the global
/// scope by default.
///
/// I guess since this is meant not only as a verifier but as a proof assistant,
/// we cannot process and drop stuff, we must save `Statement` and `Scope` information.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ParserState {
    /// Not in the middle of anything.
    ///
    /// Note that a `ConstantDeclaration` or `FileDeclaration` can only be global.
    ///
    /// So it seems the actual Parser should pass a `ParserContext` where one of the
    /// fields is the `Scope` depth.
    Global,

    /// We are in a scope.
    ///
    /// This is still basically not in the middle of anything.
    ///
    /// This is kinda like [`Global`] but a bit more restrictive.
    Scope,

    /// We are in a comment.
    ///
    /// The `ParserContext` will note which `ParserState` to return to.
    Comment,

    /// We are in a file inclusion
    FileInclusion,

    // Both `CommentDeclaration` and `FileInclusion`s do not use labels so
    // we don't have to split this.
    /// We have just read a label and are expecting some keyword (`$` token).
    Label,

    /// We are reading a bunch of [`MathSymbol`]s
    MathSymbol,

    /// This is [`ParserState::MathSymbol`] but the ending token is `$=`
    ProofMathSymbol,

    /// We don't know if this is an uncompressed or compressed proof
    ProofDetailsStart,

    UncompressedProof,

    CompressedProofLabelList,

    CompressedProofChunks,
}

#[derive(Debug, Clone, Copy)]
enum StatementKind {
    ConstantDeclaration,
}

enum InitializeStatementError<'source> {
    EmptyConstantDeclaration,
    DuplicateConstantDeclaration(Vec<Token<'source>>),
}

impl<'source> StatementKind {
    /// Verify data and construct a `Statement`.
    fn finalize(
        self,
        data: Vec<Token<'source>>,
    ) -> Result<Statement<'source>, InitializeStatementError<'source>> {
        match self {
            StatementKind::ConstantDeclaration => {
                if data.is_empty() {
                    Err(InitializeStatementError::EmptyConstantDeclaration)
                } else if has_duplicate(data.iter().map(Token::kind)) {
                    Err(InitializeStatementError::DuplicateConstantDeclaration(data))
                } else {
                    Ok(Statement::ConstantDeclaration(data))
                }
            }
        }
    }
}

enum Statement<'source> {
    ConstantDeclaration(Vec<Token<'source>>),
}

// We're gonna use std::mem::take to avoid copying stuff!
struct Parser {
    state: ParserState,
    statement: Option<StatementKind>,
    // TODO: scope stuff
}

/*
MetamathSpec.md #Syntax summary

- `$[` {MathSymbol} `$]`
- `${` {{Statement}}<sup>*</sup> `$}`
- `$c` {MathSymbol}<sup>+</sup> `$.`
- `$v` {MathSymbol}<sup>+</sup> `$.`
- `$d` {MathSymbol}<sup>2+</sup> `$.`
- {Label} `$f` {MathSymbol} {MathSymbol} `$.`
- {Label} `$e` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`

where {{ProofDetails}} is

- ({Label} | `?`)<sup>+</sup>
- `(` {Label}<sup>*</sup> `)` {CompressedProofNumber}<sup>+</sup>
*/
