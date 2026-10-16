//! Excel's complexity limit on a format code, which TEXT reports as `#VALUE!` long before the
//! code reaches 255 characters (docs/excel-semantics/number-formats.md, "TEXT format code
//! complexity limit"; probes/text-format-length-limit-c to -k, excel-graph, T-1393, T-1434,
//! T-1455).
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
//!   character, 2 per other token (`AM/PM` 3), 2 per color, 7 per condition, 0 per `;`;
//! - quoted text pays 4 more when a token or placeholder follows it directly, or when the literal
//!   run that closes its section goes on with unquoted text after it (`"d"` + 120 quoted + 2
//!   spaces works, + 3 fails; 122 quoted + `d` works, 123 fails);
//! - in a section whose closing literal run holds quoted text, its date, time and `@` tokens are
//!   free (`"d"` + 125 quoted works, like 125 quoted alone; `"h:mm"` + 123 quoted works), and `@`
//!   even takes 1 off the rest (`"@ "` + 124 quoted works, `"@  "` + 122 works and 123 fails,
//!   `"@ - "` + 121 works and 122 fails; `"d "` + 124 quoted fails);
//! - a section holding both a condition and quoted text pays 3 more (`"[>5]0"` + 116 quoted works,
//!   118 fails, while `"[>5]0"` + 79 spaces works).
//!
//! TEXT also rejects two placements of `@`, for any value (docs/excel-semantics/number-formats.md,
//! "TEXT of a text value: `@` with other tokens in one section"; probes/text-at-with-tokens,
//! probes/text-at-token-sections, excel-graph, T-1656), and they are checked here too:
//! - a section holding `@` that is not the last section (`"@;0"`, `"x@;0"`; `"0;x@"` is fine);
//! - `@` in the same section as a number or date token ([`conflicts_with_text`]): `"x@y"`, `"0@"`,
//!   `"x@%"`, `"x@[>5]"`, `"x@y;0;0;@"`. `AM/PM` and `A/P` do not conflict (`"x@AM/PM"` is "xabcAM").
//!

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
/// What `@` is refunded on top of its `TOKEN_COST` in a section whose closing literal run holds
/// quoted text (probes/text-format-length-limit-l, -m, T-1481).
const TEXT_TOKEN_EXTRA_REFUND: usize = 1;
const AMPM_COST: usize = 3;
const COLOR_COST: usize = 2;
const CONDITION_COST: usize = 7;
/// What quoted text costs on top of its characters when more of the section follows it.
const QUOTED_SWITCH_COST: usize = 4;
/// What a condition costs on top of `CONDITION_COST` in a section that also holds quoted text.
const CONDITION_WITH_QUOTED_COST: usize = 3;

#[derive(PartialEq)]
enum Run {
    None,
    Digits,
    Literal,
    /// A literal colon. It belongs to the literal run for element counting, and the next colon
    /// joins it at no cost: colon runs print as one (probes/text-format-length-limit-f, -g).
    Colon,
}

/// Whether `token` is a date or time letter token (`d`, `mmm`, `yy`, `h`, `ss`, ...). A positive
/// list (not `Token::is_date`, which leaves out time tokens), so a new variant is not caught by default.
fn is_date_time_letter(token: &Token) -> bool {
    matches!(
        token,
        Token::Day
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
            | Token::BuddhistYearShort
            | Token::BuddhistYear
            | Token::Hour
            | Token::HourPadded
            | Token::Second
            | Token::SecondPadded
    )
}

/// Whether `token` is a number or date/time token, which a fourth section may not hold.
fn is_number_or_date(token: &Token) -> bool {
    is_date_time_letter(token)
        || matches!(
            token,
            Token::Zero
                | Token::Sharp
                | Token::QuestionMark
                | Token::Scientific
                | Token::ScientificMinus
                | Token::ElapsedHour
                | Token::ElapsedMinute
                | Token::ElapsedSecond
                | Token::ElapsedHourPadded
                | Token::ElapsedMinutePadded
                | Token::ElapsedSecondPadded
                | Token::AMPM
                | Token::Era
                | Token::Calendar(_)
        )
}

/// Whether `token` makes a section holding `@` invalid (probes/text-at-with-tokens A1-A43,
/// B1-B43): the date and time letters, the elapsed tokens, the era and calendar tokens, the digit
/// placeholders, `.`, `/`, `E+`, `General`, `%` and a condition. Unlike [`is_number_or_date`] it
/// leaves out `AM/PM`, and `A/P` is not in it at all (a fourth section may hold `A/P`, probes/text-at-token-sections A20); the calendar tokens (`B1`, `B2`) are inferred from the other date tokens.
fn conflicts_with_text(token: &Token) -> bool {
    (is_number_or_date(token) && !matches!(token, Token::AMPM))
        || matches!(
            token,
            Token::Period
                | Token::Literal('/')
                | Token::General
                | Token::Percent
                | Token::Condition(..)
        )
}

/// Whether `token` costs nothing in a section whose closing literal run holds quoted text: the
/// date and time letters and `@` (probed: `d`, `dd`, `m`, `mm`, `h`, `yyyy`, `@`; seconds and day or
/// month names are inferred). Elapsed tokens, `AM/PM` and `%` keep their cost (`"[h]"` + 125 quoted
/// and `"%"` + 125 quoted fail).
fn is_free_before_closing_quoted(token: &Token) -> bool {
    is_date_time_letter(token) || matches!(token, Token::Raw)
}

/// The context-dependent costs of one section, settled when the section ends.
#[derive(Default)]
struct Section {
    elements: usize,
    conditions: usize,
    has_quoted: bool,
    /// The date, time and `@` tokens anywhere in the section (not only next to the closing run:
    /// both `h` and `mm` are free in `"h:mm"` + 123 quoted), free if that run holds quoted text.
    free_tokens: usize,
    /// How many of `free_tokens` are `@`.
    free_text_tokens: usize,
    /// Whether the latest element is quoted text.
    after_quoted: bool,
    /// Whether the current literal run holds quoted text and whether it now ends in unquoted text;
    /// both reset when a non-literal element starts.
    run_has_quoted: bool,
    run_ends_unquoted: bool,
}

impl Section {
    /// The cost to add (positive) and to take back (negative) once the section is complete.
    fn settle(&self) -> (usize, usize) {
        let mut extra = 0;
        let mut refund = 0;
        if self.run_has_quoted {
            if self.run_ends_unquoted {
                extra += QUOTED_SWITCH_COST;
            }
            refund +=
                TOKEN_COST * self.free_tokens + TEXT_TOKEN_EXTRA_REFUND * self.free_text_tokens;
        }
        if self.conditions > 0 && self.has_quoted {
            extra += CONDITION_WITH_QUOTED_COST;
        }
        (extra, refund)
    }
}

/// Whether Excel's TEXT rejects `format` as too complex (see the module docs).
pub(crate) fn exceeds_complexity_limit(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    let mut sections = 1;
    let mut section = Section::default();
    // Costs are summed before the end-of-section refunds, so the limit is checked per section.
    let mut cost = 0;
    let mut run = Run::None;
    // Whether the current section holds `@`, and whether it holds a token that conflicts with it.
    let mut has_text = false;
    let mut has_conflict = false;
    loop {
        let token = lexer.next_token();
        if sections == MAX_SECTIONS && is_number_or_date(&token) {
            return true;
        }
        has_text |= matches!(token, Token::Raw);
        has_conflict |= conflicts_with_text(&token);
        if has_text && (has_conflict || token == Token::Separator) {
            return true;
        }
        let (next_run, token_cost) = match &token {
            // An unparseable code is the formatter's to report.
            Token::EOF | Token::ILLEGAL => break,
            Token::Separator => {
                let (extra, refund) = section.settle();
                cost = (cost + extra).saturating_sub(refund);
                if cost > MAX_COST {
                    return true;
                }
                sections += 1;
                section = Section::default();
                run = Run::None;
                has_conflict = false;
                if sections > MAX_SECTIONS {
                    return true;
                }
                continue;
            }
            Token::Color(_) => (Run::None, COLOR_COST),
            Token::Condition(..) => {
                section.conditions += 1;
                if section.conditions > 1 {
                    return true;
                }
                (Run::None, CONDITION_COST)
            }
            Token::Zero | Token::Sharp | Token::QuestionMark => (Run::Digits, 0),
            Token::Comma if run == Run::Digits => continue,
            Token::Period if run == Run::Digits => (Run::Digits, DECIMAL_POINT_COST),
            Token::Literal(':') if run == Run::Colon => continue,
            Token::Literal(':') => (Run::Colon, LITERAL_COST),
            Token::Literal('$') | Token::Escaped('$') => (Run::Literal, DOLLAR_COST),
            // An escaped colon does not join a colon run: it prints (`0\:\:` is "1::").
            Token::Literal(_) | Token::Escaped(_) | Token::Currency(_) | Token::Locale => {
                (Run::Literal, LITERAL_COST)
            }
            Token::Text(text) => (Run::Literal, QUOTED_CHAR_COST * text.chars().count()),
            Token::AMPM | Token::AP(..) => (Run::None, AMPM_COST),
            _ => (Run::None, TOKEN_COST),
        };
        let counts_as_element = !matches!(token, Token::Color(_) | Token::Condition(..));
        let joins_run = match next_run {
            Run::None => false,
            Run::Digits => run == Run::Digits,
            Run::Literal | Run::Colon => matches!(run, Run::Literal | Run::Colon),
        };
        if counts_as_element && !joins_run {
            section.elements += 1;
            if next_run == Run::Digits {
                cost += PLACEHOLDER_RUN_COST;
            }
        }
        cost += token_cost;
        if counts_as_element {
            let quoted = matches!(token, Token::Text(_));
            match next_run {
                Run::Literal | Run::Colon => {
                    if !joins_run {
                        section.run_has_quoted = false;
                    }
                    section.run_has_quoted |= quoted;
                    section.run_ends_unquoted = !quoted;
                    section.has_quoted |= quoted;
                }
                Run::None | Run::Digits => {
                    if section.after_quoted {
                        cost += QUOTED_SWITCH_COST;
                    }
                    section.run_has_quoted = false;
                    section.run_ends_unquoted = false;
                }
            }
            section.after_quoted = quoted;
            if is_free_before_closing_quoted(&token) {
                section.free_tokens += 1;
                section.free_text_tokens += usize::from(matches!(token, Token::Raw));
            }
            run = next_run;
        }
        if section.elements > MAX_ELEMENTS {
            return true;
        }
    }
    let (extra, refund) = section.settle();
    (cost + extra).saturating_sub(refund) > MAX_COST
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

    /// Quoted text next to more of its section (probes/text-format-length-limit-f to -k, T-1455).
    #[test]
    fn quoted_text_in_context() {
        let ok = [
            format!("d{}", quoted(125)),
            format!("dd{}", quoted(125)),
            format!("yyyy{}", quoted(125)),
            format!("@{}", quoted(125)),
            format!("@ {}", quoted(124)),
            format!("@-{}", quoted(124)),
            format!("@  {}", quoted(122)),
            format!("@ - {}", quoted(121)),
            format!(" @ {}", quoted(122)),
            format!("d {}", quoted(123)),
            format!("h:mm{}", quoted(123)),
            format!("[h]{}", quoted(124)),
            format!("d{}{}", quoted(120), " ".repeat(2)),
            format!("d{}{}", quoted(110), " ".repeat(8)),
            format!("d{}{}", quoted(100), " ".repeat(15)),
            format!("{}{}", quoted(120), "--"),
            format!("{}d", quoted(122)),
            format!("d{}d", quoted(121)),
            format!("d{} d", quoted(120)),
            format!("d{} {}", quoted(61), quoted(61)),
            format!("0;d{}", quoted(122)),
            format!("[Red]0{}", quoted(120)),
            format!("[>5]{}", quoted(120)),
            format!("[>5]0{}", quoted(116)),
        ];
        for code in ok {
            assert!(!exceeds_complexity_limit(&code), "{code}");
        }
        let too_complex = [
            format!("d{}", quoted(126)),
            format!("d {}", quoted(124)),
            format!("@{}", quoted(126)),
            format!("@ {}", quoted(125)),
            format!("@  {}", quoted(123)),
            format!("@ - {}", quoted(122)),
            format!(" @ {}", quoted(123)),
            format!("[h]{}", quoted(125)),
            format!("AM/PM{}", quoted(124)),
            format!("%{}", quoted(125)),
            format!("d{}{}", quoted(120), " ".repeat(3)),
            format!("d{} ", quoted(122)),
            format!("d{}{}", quoted(110), " ".repeat(9)),
            format!("d{}{}", quoted(100), " ".repeat(16)),
            format!("{}{}", quoted(120), "---"),
            format!("{}d", quoted(123)),
            format!("d{}d", quoted(122)),
            format!("{}@", quoted(124)),
            format!("0;d{}", quoted(123)),
            format!("[Red]0{}", quoted(122)),
            format!("[>5]{}", quoted(122)),
            format!("[>5]0{}", quoted(118)),
        ];
        for code in too_complex {
            assert!(exceeds_complexity_limit(&code), "{code}");
        }
    }

    /// `@` outside the last section or beside a number or date token (probes/text-at-with-tokens,
    /// probes/text-at-token-sections, T-1656).
    #[test]
    fn text_token_placement() {
        let ok = [
            "x@",
            "@@",
            "0;x@",
            "0;0;x@",
            "0;0;0;x@",
            "0;@",
            "x@a",
            "x@A/P",
            "x@AM/PM",
            "A/P@",
            "x@[Red]",
            "x@[$-409]",
            "x@*-",
            "x@_x",
            "x@\\y",
            "x@\"y\"",
            "x@,",
            "x@ ",
            "0;0;0;x@\\y",
        ];
        for code in ok {
            assert!(!exceeds_complexity_limit(code), "{code}");
        }
        let invalid = [
            "x@y",
            "y@",
            "x@mmm",
            "x@h",
            "x@s",
            "x@b",
            "x@e",
            "x@g",
            "x@[h]",
            "x@[s]",
            "x@0",
            "#@",
            "x@?",
            "x@.",
            "x@/",
            "x@E+0",
            "x@General",
            "x@%",
            "x@[>5]",
            "[>5]@",
            "x@y;z",
            "x@;0",
            "@;0",
            "[Red]x@;0",
            "x@\\y;0;0;@",
            "x@y;0;0;@",
            "0;x@y;0;@",
            "0;0;x@y;@",
            "0;x@0",
        ];
        for code in invalid {
            assert!(exceeds_complexity_limit(code), "{code}");
        }
    }
}
