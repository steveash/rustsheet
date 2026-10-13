//! Excel's complexity limit on a format code, which TEXT reports as `#VALUE!` long before the
//! code reaches 255 characters (docs/excel-semantics/number-formats.md, "TEXT format code
//! complexity limit"; probes/text-format-length-limit-c to -i, excel-graph, T-1393, T-1434).
//!
//! The rule is fitted to the probed boundaries, not documented by Microsoft. The weights look like
//! the size in bytes of a compiled code (a quoted character 2, a UTF-16 unit; an unquoted one 3):
//! - at most four sections, and a fourth section may not hold a digit placeholder or a date or
//!   time token (`"0;0;0;0"` and `"d;d;d;d"` are `#VALUE!` whatever the value);
//! - at most one condition in a section (`"[>5][<9]0"` is `#VALUE!` for any value);
//! - at most 32 elements in a section, where an element is a run of digit placeholders (with
//!   their `,` and `.`), a run of literals (unquoted, escaped and quoted text merge), or any other
//!   token (`d`, `mm`, `%`, `@`, `_x`, `*x`, `[h]`, `AM/PM`, ...);
//! - a cost of at most 250 over the whole code: 5 per placeholder run plus 2 per `.` in it, 3 per
//!   unquoted or escaped literal character (`$` 1; a run of colons counts once), 2 per quoted
//!   character, 2 per other token, 1 per color, 7 per condition, 0 per `;`.
//!
//! Known misses: a date or text token followed by a quoted string at the string's limit
//! (`d"<125 a's>"` works in Excel, costs 252 here), and quoted text followed by more text or a
//! token, which costs about 4 more than modeled (T-1455).

use super::lexer::{Lexer, Token};

const MAX_SECTIONS: usize = 4;
const MAX_ELEMENTS: usize = 32;
const MAX_COST: usize = 250;
const PLACEHOLDER_RUN_COST: usize = 5;
const DECIMAL_POINT_COST: usize = 2;
const LITERAL_COST: usize = 3;
const DOLLAR_COST: usize = 1;
const QUOTED_CHAR_COST: usize = 2;
const TOKEN_COST: usize = 2;
const COLOR_COST: usize = 1;
const CONDITION_COST: usize = 7;

#[derive(PartialEq)]
enum Run {
    None,
    Digits,
    Literal,
    /// A literal colon. It belongs to the literal run for element counting, and the next colon
    /// joins it at no cost: colon runs print as one (probes/text-format-length-limit-f, -g).
    Colon,
}

/// Whether `token` is a number or date/time token, which a fourth section may not hold. A positive
/// list (not `Token::is_date`, which leaves out time tokens), so a new variant is not caught by default.
fn is_number_or_date(token: &Token) -> bool {
    matches!(
        token,
        Token::Zero
            | Token::Sharp
            | Token::QuestionMark
            | Token::Scientific
            | Token::ScientificMinus
            | Token::Day
            | Token::DayPadded
            | Token::DayNameShort
            | Token::DayName
            | Token::Month
            | Token::MonthPadded
            | Token::MonthNameShort
            | Token::MonthName
            | Token::MonthLetter
            | Token::YearShort
            | Token::Year
            | Token::Hour
            | Token::HourPadded
            | Token::Second
            | Token::SecondPadded
            | Token::ElapsedHour
            | Token::ElapsedMinute
            | Token::ElapsedSecond
            | Token::ElapsedHourPadded
            | Token::ElapsedMinutePadded
            | Token::ElapsedSecondPadded
            | Token::AMPM
    )
}

/// Whether Excel's TEXT rejects `format` as too complex (see the module docs).
pub(crate) fn exceeds_complexity_limit(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    let mut sections = 1;
    let mut elements = 0;
    let mut conditions = 0;
    let mut cost = 0;
    let mut run = Run::None;
    loop {
        let token = lexer.next_token();
        if sections == MAX_SECTIONS && is_number_or_date(&token) {
            return true;
        }
        let (next_run, token_cost) = match &token {
            // An unparseable code is the formatter's to report.
            Token::EOF | Token::ILLEGAL => break,
            Token::Separator => {
                sections += 1;
                elements = 0;
                conditions = 0;
                run = Run::None;
                if sections > MAX_SECTIONS {
                    return true;
                }
                continue;
            }
            Token::Color(_) => (Run::None, COLOR_COST),
            Token::Condition(..) => {
                conditions += 1;
                if conditions > 1 {
                    return true;
                }
                (Run::None, CONDITION_COST)
            }
            Token::Zero | Token::Sharp | Token::QuestionMark => (Run::Digits, 0),
            Token::Comma if run == Run::Digits => continue,
            Token::Period if run == Run::Digits => (Run::Digits, DECIMAL_POINT_COST),
            Token::Literal(':') if run == Run::Colon => continue,
            Token::Literal(':') => (Run::Colon, LITERAL_COST),
            Token::Literal('$') => (Run::Literal, DOLLAR_COST),
            Token::Literal(_) | Token::Currency(_) => (Run::Literal, LITERAL_COST),
            Token::Text(text) => (Run::Literal, QUOTED_CHAR_COST * text.chars().count()),
            _ => (Run::None, TOKEN_COST),
        };
        let counts_as_element = !matches!(token, Token::Color(_) | Token::Condition(..));
        let joins_run = match next_run {
            Run::None => false,
            Run::Digits => run == Run::Digits,
            Run::Literal | Run::Colon => matches!(run, Run::Literal | Run::Colon),
        };
        if counts_as_element && !joins_run {
            elements += 1;
            if next_run == Run::Digits {
                cost += PLACEHOLDER_RUN_COST;
            }
        }
        cost += token_cost;
        if counts_as_element {
            run = next_run;
        }
        if elements > MAX_ELEMENTS || cost > MAX_COST {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::exceeds_complexity_limit;

    fn quoted(n: usize) -> String {
        format!("\"{}\"", "a".repeat(n))
    }

    #[test]
    fn probed_boundaries() {
        let ok = [
            "dm".repeat(16),
            format!("dd{}", " ".repeat(81)),
            format!("d{}", " ".repeat(82)),
            "-".repeat(83),
            format!("d{}", "\\a".repeat(82)),
            format!("d{}", "$".repeat(160)),
            format!("d{}", ":".repeat(85)),
            format!("0{};0{}", " ".repeat(40), " ".repeat(40)),
            format!("0;0;0;{}", " ".repeat(78)),
            "0.".repeat(20),
            "\"a\" ".repeat(17),
            quoted(125),
            format!("0{}", quoted(122)),
            format!("0.0{}", quoted(121)),
            format!("[Red]{}", quoted(124)),
            format!("d{}{}", quoted(50), " ".repeat(48)),
            format!("{}{}", " ".repeat(50), quoted(50)),
            format!("{}{}", quoted(100), "-".repeat(15)),
            "0;;;".to_string(),
            "0;0;0;@".to_string(),
            "d;d;d".to_string(),
            format!("[Red]0{}", " ".repeat(80)),
            format!("[>5]0{}", " ".repeat(79)),
            "[Red][>5]0".to_string(),
            "[>5]0;[<9]0".to_string(),
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
            quoted(126),
            format!("d{}", quoted(126)),
            format!("0{}", quoted(123)),
            format!("0.0{}", quoted(122)),
            format!("[Red]{}", quoted(125)),
            format!("d{}{}", quoted(50), " ".repeat(50)),
            format!("{}{}", " ".repeat(52), quoted(50)),
            format!("{}{}", quoted(100), "-".repeat(18)),
            "0;;;;".to_string(),
            "0;0;0;0".to_string(),
            "d;d;d;d".to_string(),
            "0%".repeat(17),
            format!("[>5]0{}", " ".repeat(80)),
            "[>5][<9]0".to_string(),
            "[>5]0[<9]".to_string(),
        ];
        for code in too_complex {
            assert!(exceeds_complexity_limit(&code), "{code}");
        }
    }
}
