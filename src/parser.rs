use std::iter::Peekable;

use crate::lexer::{Token, TokenKind, Tokens, lex};

struct TokenStream<'source> {
    // source: &'source str,
    lexer: Peekable<Tokens<'source>>,
}

// Since this is only used internally in a limited and specific way,
// it's ok to break the 0, 1, or infinity rule for speed and clarity.
enum OneOrTwoOf<T> {
    One(T),
    Two(T, T),
}

struct ParseFailError<'source, 'b> {
    received: Option<&'b Token<'source>>,
    expected: OneOrTwoOf<TokenKind>,
}

impl<'source> TokenStream<'source> {
    fn new(source: &'source str) -> Self {
        TokenStream {
            // source,
            lexer: lex(source).peekable(),
        }
    }

    fn peek(&mut self) -> Option<&Token<'source>> {
        self.lexer.peek()
    }

    fn bump(&mut self) -> Option<Token<'_>> {
        self.lexer.next()
    }

    /// Consume a token only when it satisfies the expected [`TokenKind`],
    /// otherwise it fails and doesn't consume anything.
    fn expect<'b>(
        &'b mut self,
        expected: TokenKind,
    ) -> Result<Token<'source>, ParseFailError<'source, 'b>> {
        self.lexer
            .next_if(|v| v.satisfies(expected))
            .ok_or_else(|| ParseFailError {
                received: self.peek(),
                expected: OneOrTwoOf::One(expected),
            })
    }

    fn expect_either(
        &mut self,
        one: TokenKind,
        two: TokenKind,
    ) -> Result<Token<'_>, ParseFailError<'_, '_>> {
        self.lexer
            .next_if(|v| v.satisfies(one) || v.satisfies(two))
            .ok_or_else(|| ParseFailError {
                received: self.peek(),
                expected: OneOrTwoOf::Two(one, two),
            })
    }
}
