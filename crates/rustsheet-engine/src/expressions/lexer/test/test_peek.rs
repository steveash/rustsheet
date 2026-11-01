#![allow(clippy::unwrap_used)]

//! The peek cache (T-0499): a peeked token, taken or advanced past, leaves the lexer where a
//! fresh read leaves it, and a setting change drops it.

use crate::expressions::lexer::{Lexer, LexerMode};
use crate::expressions::token::TokenType;
use crate::language::get_language;
use crate::locale::get_locale;

const FORMULAS: [&str; 6] = [
    "=IF($E100<>\"\",COUNTIFS('XA Data Pull'!$J:$J,\">=\"&AA$1)+1,\"\")",
    "3:5",
    "SUM(A1:B2 B1:C3)%",
    "-+5^2&\"x\"",
    "{1,2;3,4}",
    "R[1]C[-2]+1",
];

fn lexer(formula: &str, mode: LexerMode) -> Lexer {
    Lexer::new(
        formula,
        mode,
        get_locale("en").unwrap(),
        get_language("en").unwrap(),
    )
}

/// Every token with the position after it, read with `next_token` alone.
fn fresh_reads(formula: &str, mode: LexerMode) -> Vec<(TokenType, i32)> {
    let mut lx = lexer(formula, mode);
    let mut reads = Vec::new();
    loop {
        let token = lx.next_token();
        reads.push((token.clone(), lx.get_position()));
        if token == TokenType::EOF || reads.len() > 100 {
            return reads;
        }
    }
}

#[test]
fn cached_peeks_leave_the_state_of_fresh_reads() {
    for mode in [LexerMode::A1, LexerMode::R1C1] {
        for formula in FORMULAS {
            let expected = fresh_reads(formula, mode.clone());
            // Peek twice, then take the token: by `next_token` on even steps, by
            // `advance_token` on odd ones.
            let mut lx = lexer(formula, mode.clone());
            for (step, (token, position)) in expected.iter().enumerate() {
                assert_eq!(&lx.peek_token(), token, "{formula} step {step}");
                assert_eq!(lx.peek(), token, "{formula} step {step}");
                if step % 2 == 0 {
                    assert_eq!(&lx.next_token(), token, "{formula} step {step}");
                } else {
                    lx.advance_token();
                }
                assert_eq!(lx.get_position(), *position, "{formula} step {step}");
            }
        }
    }
}

#[test]
fn a_setting_change_drops_the_peeked_token() {
    let mut lx = lexer("R1C1", LexerMode::A1);
    let a1 = lx.peek_token();
    lx.set_lexer_mode(LexerMode::R1C1);
    let r1c1 = lx.next_token();
    assert_ne!(a1, r1c1);
    assert_eq!(r1c1, fresh_reads("R1C1", LexerMode::R1C1)[0].0);
    lx.set_formula("A1");
    lx.set_lexer_mode(LexerMode::A1);
    assert_eq!(lx.peek_token(), fresh_reads("A1", LexerMode::A1)[0].0);
}
