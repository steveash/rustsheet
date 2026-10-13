//! Excel's complexity limit on a format code, which TEXT reports as `#VALUE!` long before the
//! code reaches 255 characters (docs/excel-semantics/number-formats.md, "TEXT format code
//! complexity limit"; probes/text-format-length-limit-c to -p, excel-graph, T-1393, T-1434,
//! T-1455, T-1481, T-1486).
//!
//! The rule is fitted to the probed boundaries, not documented by Microsoft. The weights look like
//! the size in bytes of a compiled code (a quoted character 2, a UTF-16 unit; an unquoted one 3):
//! - at most four sections, and a fourth section may not hold a digit placeholder or a date or
//!   time token (`"0;0;0;0"` and `"d;d;d;d"` are `#VALUE!` whatever the value);
//! - at most one condition in a section (`"[>5][<9]0"` is `#VALUE!` for any value);
//! - at most 32 elements in a section, where an element is a run of digit placeholders (with
//!   their `,` and `.`), a run of literals (unquoted, escaped and quoted text merge), or any other
//!   token (`d`, `mm`, `%`, `@`, `_x`, `*x`, `[h]`, `AM/PM`, ...);
//! - a cost of at most 250 over the whole code: 3 per section holding a digit placeholder, 2 per
//!   placeholder run (`"0"` costs 5, `"0-0"` 10) plus 2 per `.` in it, 3 per
//!   unquoted or escaped literal character (`$` 1; a run of colons counts once), 2 per quoted
//!   character, 2 per other token (`AM/PM` 3), 2 per color, 7 per condition, 0 per `;`;
//! - quoted text pays 4 more when a token follows it directly, 2 when unquoted text comes between
//!   them (`"a" @` + 120 quoted works, 121 fails) and 1 when a color or condition does; 1 when a
//!   placeholder run follows it directly or after unquoted text and 0 after a color or condition
//!   (`"0"` + 120 quoted + `"[Red]0"` works, 121 fails); and 4 when the literal run that closes its section goes
//!   on with unquoted text after it (`"d"` + 120 quoted + 2 spaces works, + 3 fails; 122 quoted +
//!   `d` works, 123 fails);
//! - in a section whose closing literal run holds quoted text, its date, time and `@` tokens are
//!   free (`"d"` + 125 quoted works, like 125 quoted alone; `"h:mm"` + 123 quoted works), and `@`
//!   even takes 1 off the rest (`"@ "` + 124 quoted works, `"@  "` + 122 works and 123 fails,
//!   `"@ - "` + 121 works and 122 fails; `"d "` + 124 quoted fails);
//! - a section holding a condition, quoted text and a placeholder pays 3 more (`"[>5]0"` + 116
//!   quoted works, 118 fails, while `"[>5]0"` + 79 spaces works), and a section after the first
//!   holding quoted text and a placeholder 1 more (`"0;0"` + 120 quoted fails).
//!
//! A code with a section that does not parse is rejected too, whatever the value
//! ([`is_unparseable`]).
//!

use super::lexer::{Lexer, Token};
use super::parser::{ParsePart, Parser};

const MAX_SECTIONS: usize = 4;
const MAX_ELEMENTS: usize = 32;
const MAX_COST: usize = 250;
/// What a section's first digit placeholder costs once, on top of `PLACEHOLDER_RUN_COST` per run
/// (`"0-0"` + 120 quoted works and 121 fails, `"0-0-0"` + 117 works and 118 fails;
/// probes/text-format-length-limit-o, -p, T-1486).
const NUMBER_SECTION_COST: usize = 3;
const PLACEHOLDER_RUN_COST: usize = 2;
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
/// What a token pays instead of `QUOTED_SWITCH_COST` when unquoted text comes between the quoted
/// text and the token (`"a" @` + 120 quoted works and 121 fails, `"a"--d` + 120 works and 121
/// fails; probes/text-format-length-limit-n, -o).
const QUOTED_SWITCH_INDIRECT_COST: usize = 2;
/// What a token pays instead when a color or condition comes between (`"d"` + 121 quoted +
/// `"[Red]d"` works and 122 fails, + 119 quoted + `"[>5]d"` works and 120 fails;
/// probes/text-format-length-limit-n, -o, -q).
const QUOTED_SWITCH_BRACKET_COST: usize = 1;
/// What a placeholder run pays after quoted text, directly or after unquoted text; nothing after a
/// color or condition (`"a" 0` + 119 quoted works and 120 fails, `"0"` + 120 quoted + `"[Red]0"`
/// works and 121 fails; probes/text-format-length-limit-l, -n, -o, -p).
const QUOTED_SWITCH_PLACEHOLDER_COST: usize = 1;
/// What a condition costs on top of `CONDITION_COST` in a section that also holds quoted text and a
/// digit placeholder (`"[>5]0"` + 116 quoted works and 118 fails; without a placeholder,
/// `"d"` + 119 quoted + `"[>5]d"` works; probes/text-format-length-limit-q A16, T-1486).
const CONDITION_WITH_QUOTED_COST: usize = 3;
/// What a section after the first pays when it holds both a digit placeholder and quoted text
/// (`"0-0;0-0"` + 114 quoted works and 115 fails, `"0;0"` + 120 fails, while `"0-0"` + 120 works
/// and `"0-0;"` + 120 works; probes/text-format-length-limit-r, -q A14:A15, T-1486).
const LATER_NUMBER_SECTION_WITH_QUOTED_COST: usize = 1;

/// What came after the latest quoted text of a section, which sets what the next token or
/// placeholder run pays for following it.
#[derive(Clone, Copy, Default, PartialEq)]
enum AfterQuoted {
    /// No quoted text, or a token or placeholder already followed it.
    #[default]
    None,
    /// The quoted text is the latest element.
    Directly,
    /// Unquoted text in the same literal run followed it.
    Unquoted,
    /// A color or condition followed it.
    Bracket,
}

impl AfterQuoted {
    /// The state after a color or condition.
    fn through_bracket(self) -> Self {
        if self == AfterQuoted::None {
            self
        } else {
            AfterQuoted::Bracket
        }
    }

    /// What a token (`placeholder` false) or a placeholder run pays for following quoted text.
    fn switch_cost(self, placeholder: bool) -> usize {
        match (self, placeholder) {
            (AfterQuoted::None, _) | (AfterQuoted::Bracket, true) => 0,
            (AfterQuoted::Directly, false) => QUOTED_SWITCH_COST,
            (AfterQuoted::Unquoted, false) => QUOTED_SWITCH_INDIRECT_COST,
            (AfterQuoted::Bracket, false) => QUOTED_SWITCH_BRACKET_COST,
            (AfterQuoted::Directly | AfterQuoted::Unquoted, true) => QUOTED_SWITCH_PLACEHOLDER_COST,
        }
    }
}

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
    /// Whether a section comes before this one.
    later: bool,
    elements: usize,
    conditions: usize,
    has_quoted: bool,
    /// The date, time and `@` tokens anywhere in the section (not only next to the closing run:
    /// both `h` and `mm` are free in `"h:mm"` + 123 quoted), free if that run holds quoted text.
    free_tokens: usize,
    /// How many of `free_tokens` are `@`.
    free_text_tokens: usize,
    /// Whether the section holds a digit placeholder (it pays `NUMBER_SECTION_COST` once).
    has_placeholder: bool,
    /// What followed the latest quoted text.
    after_quoted: AfterQuoted,
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
        if self.conditions > 0 && self.has_quoted && self.has_placeholder {
            extra += CONDITION_WITH_QUOTED_COST;
        }
        if self.later && self.has_quoted && self.has_placeholder {
            extra += LATER_NUMBER_SECTION_WITH_QUOTED_COST;
        }
        (extra, refund)
    }
}

/// Whether any section of `format` fails to parse. TEXT is then `#VALUE!` whatever the value,
/// even when another section would format it: `TEXT(5,"0;""")` and `TEXT("x","0;0;0;""")` are
/// `#VALUE!` (probes/text-format-length-limit-q; docs/excel-semantics/number-formats.md).
pub(crate) fn is_unparseable(format: &str) -> bool {
    let mut parser = Parser::new(format);
    parser.parse();
    parser.parts.iter().any(ParsePart::is_error)
}

/// Whether Excel's TEXT rejects `format` as too complex (see the module docs).
pub(crate) fn exceeds_complexity_limit(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    let mut sections = 1;
    let mut section = Section::default();
    // Costs are summed before the end-of-section refunds, so the limit is checked per section.
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
                let (extra, refund) = section.settle();
                cost = (cost + extra).saturating_sub(refund);
                if cost > MAX_COST {
                    return true;
                }
                sections += 1;
                section = Section {
                    later: true,
                    ..Section::default()
                };
                run = Run::None;
                if sections > MAX_SECTIONS {
                    return true;
                }
                continue;
            }
            Token::Color(_) => {
                section.after_quoted = section.after_quoted.through_bracket();
                (Run::None, COLOR_COST)
            }
            Token::Condition(..) => {
                section.after_quoted = section.after_quoted.through_bracket();
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
            Token::Literal('$') => (Run::Literal, DOLLAR_COST),
            Token::Literal(_) | Token::Currency(_) => (Run::Literal, LITERAL_COST),
            Token::Text(text) => (Run::Literal, QUOTED_CHAR_COST * text.chars().count()),
            Token::AMPM => (Run::None, AMPM_COST),
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
                if !section.has_placeholder {
                    section.has_placeholder = true;
                    cost += NUMBER_SECTION_COST;
                }
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
                    section.after_quoted = if quoted {
                        AfterQuoted::Directly
                    } else if section.after_quoted == AfterQuoted::None {
                        AfterQuoted::None
                    } else {
                        AfterQuoted::Unquoted
                    };
                }
                Run::None | Run::Digits => {
                    // A digit joining its run (after a color) pays nothing more.
                    if !joins_run {
                        cost += section.after_quoted.switch_cost(next_run == Run::Digits);
                    }
                    section.after_quoted = AfterQuoted::None;
                    section.run_has_quoted = false;
                    section.run_ends_unquoted = false;
                }
            }
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

    /// What follows quoted text, and placeholder runs after the first: each code works with `n`
    /// quoted characters between its prefix and suffix and fails with `n + 1`, or only works where
    /// one side was probed (probes/text-format-length-limit-l to -q, T-1486).
    #[test]
    fn after_quoted_text_and_placeholder_runs() {
        let code = |prefix: &str, n: usize, suffix: &str| format!("{prefix}{}{suffix}", quoted(n));
        let boundaries = [
            ("0", 120, "[Red]0"),
            ("", 121, "[Red]0"),
            ("0", 116, "[>5]0"),
            ("d", 121, "[Red]d"),
            ("d", 119, "[>5]d"),
            ("\"a\" @ ", 120, ""),
            ("\"a\"-@", 122, ""),
            ("\"a\" d", 121, ""),
            ("d\"a\" d", 121, ""),
            ("\"a\"\\-d", 121, ""),
            ("\"a\"--d", 120, ""),
            ("\"a\" 0", 119, ""),
            (" ", 120, "d"),
            ("0-0", 120, ""),
            ("0-0-0", 117, ""),
            ("0-0;0-0", 114, ""),
            ("0-0;0", 117, ""),
            ("0;0-0", 117, ""),
            ("0-0;", 120, ""),
            ("0-0;-", 118, ""),
            ("0;", 122, ""),
        ];
        for (prefix, n, suffix) in boundaries {
            assert!(
                !exceeds_complexity_limit(&code(prefix, n, suffix)),
                "{prefix} {n} {suffix}"
            );
            assert!(
                exceeds_complexity_limit(&code(prefix, n + 1, suffix)),
                "{prefix} {n} {suffix}"
            );
        }
        let works = [
            (" ", 120, "[Red]0"),
            (" ", 116, "[>5]0"),
            ("\"a\"@", 122, ""),
            ("d\"a\"d", 122, ""),
            ("", 121, "0"),
            ("0", 121, "0"),
            ("0[Red]0", 121, ""),
        ];
        for (prefix, n, suffix) in works {
            assert!(
                !exceeds_complexity_limit(&code(prefix, n, suffix)),
                "{prefix} {n} {suffix}"
            );
        }
    }
}
