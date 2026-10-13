// CHAR, CODE, UNICHAR, CLEAN, ASC

use super::text_arg::broadcast_text;
use crate::{
    calc_result::CalcResult,
    cast::array_node_to_string,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

// Windows-1252 codepoints for the range 0x80..=0x9F (positions 128-159).
// Unicode has C1 control characters there; Windows-1252 has printable symbols.
// https://en.wikipedia.org/wiki/Windows-1252
const WIN1252_128_159: [char; 32] = [
    '\u{20AC}', // 128  €
    '',        // 129  '\u{FFFD}' undefined → replacement character
    '\u{201A}', // 130  ‚
    '\u{0192}', // 131  ƒ
    '\u{201E}', // 132  „
    '\u{2026}', // 133  …
    '\u{2020}', // 134  †
    '\u{2021}', // 135  ‡
    '\u{02C6}', // 136  ˆ
    '\u{2030}', // 137  ‰
    '\u{0160}', // 138  Š
    '\u{2039}', // 139  ‹
    '\u{0152}', // 140  Œ
    '',        // 141  undefined
    '\u{017D}', // 142  Ž
    '',        // 143  undefined
    '',        // 144  undefined
    '\u{2018}', // 145  '
    '\u{2019}', // 146  '
    '\u{201C}', // 147  "
    '\u{201D}', // 148  "
    '\u{2022}', // 149  •
    '\u{2013}', // 150  –
    '\u{2014}', // 151  —
    '\u{02DC}', // 152  ˜
    '\u{2122}', // 153  ™
    '\u{0161}', // 154  š
    '\u{203A}', // 155  ›
    '\u{0153}', // 156  œ
    '',        // 157  undefined
    '\u{017E}', // 158  ž
    '\u{0178}', // 159  Ÿ
];

fn win1252_to_char(n: u32) -> Option<char> {
    match n {
        0 => None,
        1..=127 => char::from_u32(n),
        128..=159 => {
            let c = WIN1252_128_159[(n - 128) as usize];
            if c == '\u{FFFD}' {
                None
            } else {
                Some(c)
            }
        }
        160..=255 => char::from_u32(n),
        _ => None,
    }
}

fn char_to_win1252(c: char) -> Option<u32> {
    let cp = c as u32;
    match cp {
        1..=127 => Some(cp),
        160..=255 => Some(cp),
        _ => {
            // Search the 128-159 table
            for (i, &wc) in WIN1252_128_159.iter().enumerate() {
                if wc == c {
                    return Some(128 + i as u32);
                }
            }
            None
        }
    }
}

impl Model {
    /// CHAR(number) — Returns the character specified by the Windows-1252 code number.
    pub(crate) fn fn_char(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let n = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        match win1252_to_char(n) {
            Some(c) => CalcResult::String(c.to_string()),
            None => CalcResult::new_error(Error::VALUE, cell, "Invalid character code".to_string()),
        }
    }

    /// CODE(text) — Returns the Windows-1252 numeric code for the first character of text.
    /// Lifts over a range or array argument (probes/text-argument-modes F5:F6).
    pub(crate) fn fn_code(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(t) => t,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text], |i, j| {
            let s = array_node_to_string(text.elem(i, j))?;
            let c = s.chars().next().ok_or(Error::VALUE)?;
            // Excel: a character outside Windows-1252 gives 63, the code of "?"
            // (docs/excel-semantics/text.md#clean-and-code-scalar-edges).
            let n = char_to_win1252(c).unwrap_or(63);
            Ok(ArrayNode::Number(f64::from(n)))
        })
    }

    /// UNICHAR(number) — the character for a Unicode code point, truncated toward zero. Lifts over
    /// a range or array argument. Below 1 or above 0x10FFFF is `#VALUE!`; a surrogate or a Unicode
    /// noncharacter (U+FDD0..U+FDEF, U+xxFFFE, U+xxFFFF) is `#N/A`
    /// (docs/excel-semantics/text.md#unichar-code-point-bounds).
    pub(crate) fn fn_unichar(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let number = match self.text_arg(&args[0], cell) {
            Ok(t) => t,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&number], |i, j| {
            let n = self.array_node_to_number(number.elem(i, j))?.trunc();
            if !(1.0..=f64::from(char::MAX as u32)).contains(&n) {
                return Err(Error::VALUE);
            }
            let cp = n as u32;
            let noncharacter = (0xFDD0..=0xFDEF).contains(&cp) || cp & 0xFFFE == 0xFFFE;
            match char::from_u32(cp) {
                Some(c) if !noncharacter => Ok(ArrayNode::String(c.to_string())),
                _ => Err(Error::NA),
            }
        })
    }

    /// CLEAN(text) — removes code points 0-31 and the C1 controls 128-159, and nothing else
    /// (docs/excel-semantics/text.md#clean-and-code-scalar-edges).
    pub(crate) fn fn_clean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, |s| {
            s.chars()
                .filter(|&c| !matches!(c as u32, 0..=31 | 128..=159))
                .collect()
        })
    }

    /// ASC(text) — For non-DBCS locales, returns text unchanged.
    /// Full-width to half-width conversion is only meaningful in DBCS locales.
    pub(crate) fn fn_asc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let s = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        CalcResult::String(asc_convert(&s))
    }

    /// DBCS(text) — text unchanged: en-US Excel for the web does not widen it
    /// (probes/text-modes-dbcs C3, `DBCS("abc")` = "abc"; T-1213). Only ASCII is probed, and
    /// so is only a scalar argument: like ASC it does not lift over a range until a probe shows how.
    pub(crate) fn fn_dbcs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.get_string(&args[0], cell) {
            Ok(s) => CalcResult::String(s),
            Err(e) => e,
        }
    }

    /// PHONETIC(reference) — `#N/A` for a text cell, a number cell and a range alike, as
    /// excel-graph evaluates it (probes/text-modes-phonetic, -range, -number; T-1213). A
    /// literal argument is refused by the oracle, so it is unobservable.
    pub(crate) fn fn_phonetic(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::new_error(
            Error::NA,
            cell,
            "PHONETIC has no phonetic text to return".to_string(),
        )
    }
}

/// Converts full-width Unicode characters to their ASCII half-width equivalents.
/// Full-width Latin/ASCII: U+FF01–U+FF5E → U+0021–U+007E
/// Full-width space: U+3000 → U+0020
fn asc_convert(text: &str) -> String {
    text.chars()
        .map(|c| {
            let cp = c as u32;
            if cp == 0x3000 {
                ' '
            } else if (0xFF01..=0xFF5E).contains(&cp) {
                char::from_u32(cp - 0xFF01 + 0x0021).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}
