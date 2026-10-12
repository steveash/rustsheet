//! Excel's complexity limit on a format code, which TEXT reports as `#VALUE!` long before the
//! code reaches 255 characters (docs/excel-semantics/number-formats.md, "TEXT format code
//! complexity limit"; probes/text-format-length-limit-c/d/e, excel-graph, T-1393).
//!
//! The rule is fitted to the probed boundaries, not documented by Microsoft:
//! - at most four sections;
//! - at most 32 elements in a section, where an element is a run of digit placeholders (with
//!   their `,` and `.`), a run of literals (unquoted, escaped and quoted text merge), or any other
//!   token (`d`, `mm`, `%`, `@`, `_x`, `*x`, `[h]`, `AM/PM`, ...);
//! - a cost of at most 166 over the whole code: 2 per placeholder element, 2 per unquoted or
//!   escaped literal character (`$` 1), 1.5 per quoted character (rounded up per string), 1 per
//!   `;`. Colors and conditions cost nothing (`[Red]0` plus 80 spaces works; unprobed beyond).

use super::lexer::{Lexer, Token};

const MAX_SECTIONS: usize = 4;
const MAX_ELEMENTS: usize = 32;
const MAX_COST: usize = 166;

#[derive(PartialEq)]
enum Run {
    None,
    Digits,
    Literal,
}

/// Whether Excel's TEXT rejects `format` as too complex (see the module docs).
pub(crate) fn exceeds_complexity_limit(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    let mut sections = 1;
    let mut elements = 0;
    let mut cost = 0;
    let mut run = Run::None;
    loop {
        let token = lexer.next_token();
        let (next_run, token_cost) = match &token {
            // An unparseable code is the formatter's to report.
            Token::EOF | Token::ILLEGAL => break,
            Token::Separator => {
                sections += 1;
                elements = 0;
                run = Run::None;
                cost += 1;
                continue;
            }
            Token::Color(_) | Token::Condition(..) => continue,
            Token::Zero | Token::Sharp | Token::QuestionMark => (Run::Digits, 0),
            Token::Comma | Token::Period if run == Run::Digits => continue,
            Token::Literal('$') => (Run::Literal, 1),
            Token::Literal(_) | Token::Currency(_) => (Run::Literal, 2),
            Token::Text(text) => {
                let chars = text.chars().count();
                (Run::Literal, chars + chars.div_ceil(2))
            }
            _ => (Run::None, 2),
        };
        if next_run == Run::None || next_run != run {
            elements += 1;
            if next_run == Run::Digits {
                cost += 2;
            }
        }
        cost += token_cost;
        run = next_run;
        if sections > MAX_SECTIONS || elements > MAX_ELEMENTS || cost > MAX_COST {
            return true;
        }
    }
    sections > MAX_SECTIONS
}

#[cfg(test)]
mod tests {
    use super::exceeds_complexity_limit;

    #[test]
    fn probed_boundaries() {
        let ok = [
            "dm".repeat(16),
            format!("dd{}", " ".repeat(82)),
            "-".repeat(83),
            format!("d{}", "\\a".repeat(82)),
            format!("d{}", "$".repeat(160)),
            format!("0{};0{}", " ".repeat(40), " ".repeat(40)),
            format!("0;0;0;{}", " ".repeat(78)),
            "0.".repeat(20),
            "\"a\" ".repeat(17),
            format!("d\"{}\"", "a".repeat(90)),
            "0;;;".to_string(),
        ];
        for code in ok {
            assert!(!exceeds_complexity_limit(&code), "{code}");
        }
        let too_complex = [
            "dm".repeat(16) + "d",
            format!("d{}", " ".repeat(83)),
            "-".repeat(84),
            format!("d{}", "\\a".repeat(83)),
            format!("d{}", "$".repeat(250)),
            format!("0{};0{}", " ".repeat(41), " ".repeat(40)),
            format!("d\"{}\"", "a".repeat(150)),
            "0;;;;".to_string(),
            "0%".repeat(17),
        ];
        for code in too_complex {
            assert!(exceeds_complexity_limit(&code), "{code}");
        }
    }
}
