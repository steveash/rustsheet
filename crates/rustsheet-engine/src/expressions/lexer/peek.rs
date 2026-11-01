//! The lexer's one-token lookahead: [`Lexer::peek_token`], [`Lexer::peek`] and taking the
//! peeked token with [`Lexer::next_token`]. Split out of `mod.rs` to keep it under its size
//! ratchet (T-0499).

use super::Lexer;
use crate::expressions::token::TokenType;

/// A token read by [`Lexer::peek_token`], with the positions it was read from and up to.
#[derive(Clone)]
pub(super) struct Peeked {
    start: usize,
    end: usize,
    /// What reading it left in `next_token_position` (a peek inside the token, e.g. for the
    /// `:` of a range, sets it), so that taking it with `next_token` leaves the same state.
    inner_next: Option<usize>,
    token: TokenType,
}

impl Lexer {
    /// Checks the next token without advancing position
    /// See also [`advance_token`](Self::advance_token)
    pub fn peek_token(&mut self) -> TokenType {
        self.peek().clone()
    }

    /// The next token, borrowed: [`peek_token`](Self::peek_token) without the copy, for a
    /// caller that only looks at it (most peeks only compare it with one token, T-0499).
    pub fn peek(&mut self) -> &TokenType {
        let start = self.position;
        if let Some(peeked) = self.peeked.take_if(|p| p.start == start) {
            self.next_token_position = Some(peeked.end);
            return &self.peeked.insert(peeked).token;
        }
        self.next_token_position = None;
        let token = self.lex_token();
        let end = self.position;
        let inner_next = self.next_token_position;
        self.next_token_position = Some(end);
        self.position = start;
        &self
            .peeked
            .insert(Peeked {
                start,
                end,
                inner_next,
                token,
            })
            .token
    }

    /// Advances position. This is used in conjunction with [`peek_token`](Self::peek_token)
    /// It is a noop if the has not been a previous `peek_token`
    pub fn advance_token(&mut self) {
        if let Some(position) = self.next_token_position {
            self.position = position;
            self.next_token_position = None;
        }
    }

    /// Returns the next token
    pub fn next_token(&mut self) -> TokenType {
        self.next_token_position = None;
        if let Some(peeked) = self.peeked.take_if(|p| p.start == self.position) {
            self.position = peeked.end;
            self.next_token_position = peeked.inner_next;
            return peeked.token;
        }
        self.lex_token()
    }
}
