//! REGEXREPLACE's replacement text: PCRE2's extended substitution syntax, parsed once per
//! pattern, and the case forcing it applies (text.md "REGEXREPLACE replacement syntax").

use regex_automata::{util::captures::Captures, Match};

use super::{
    regex_engine::{Engine, Search, Spent},
    string_format::{lower_case, upper_case},
};

/// One part of a parsed REGEXREPLACE replacement.
enum Piece {
    /// Text copied as it is.
    Literal(String),
    /// A capture group's text.
    Group(usize),
    /// A case-forcing escape that changes how the text after it is emitted.
    Case(CaseOp),
}

/// A PCRE2 case-forcing escape in a replacement (text.md "REGEXREPLACE replacement syntax").
#[derive(Clone, Copy, PartialEq)]
enum CaseOp {
    /// `\U`: upper-case what follows, up to `\E` or the end of the match's replacement.
    Upper,
    /// `\L`: lower-case what follows.
    Lower,
    /// `\u`: upper-case the next character only; a mode set by `\U` or `\L` goes on after it.
    UpperNext,
    /// `\l`: lower-case the next character and, unlike PCRE2's, everything after it, like `\L`
    /// (probes/regexreplace-extended-escapes B1, B9, B11, I1).
    LowerNext,
    /// `\E`: stop forcing case, a pending `\u` or `\l` included.
    End,
}

/// Which way a case-forcing escape changes the case of the text.
#[derive(Clone, Copy, PartialEq)]
enum Force {
    Upper,
    Lower,
}

/// The case forcing in effect while one match's replacement is built.
#[derive(Default)]
struct CaseState {
    /// The `\U` or `\L` mode.
    mode: Option<Force>,
    /// The `\u` or `\l` that applies to the next character.
    next: Option<Force>,
    /// Text under `\L` not yet lowered: the run is lowered whole, so a capital sigma sees the
    /// text that follows it in the next piece (probes/regexreplace-final-sigma A2: `\L$1x` with
    /// `$1` = ΑΣ gives ασx, not αςx).
    pending: String,
}

impl CaseState {
    fn apply(&mut self, op: CaseOp, out: &mut String) {
        self.finish(out);
        match op {
            CaseOp::Upper => self.mode = Some(Force::Upper),
            CaseOp::Lower => self.mode = Some(Force::Lower),
            CaseOp::UpperNext => self.next = Some(Force::Upper),
            CaseOp::LowerNext => {
                self.mode = Some(Force::Lower);
                self.next = Some(Force::Lower);
            }
            CaseOp::End => *self = CaseState::default(),
        }
    }

    /// Lowers the text held back under `\L` into `out`.
    fn finish(&mut self, out: &mut String) {
        if !self.pending.is_empty() {
            out.push_str(&lower_case(&self.pending));
            self.pending.clear();
        }
    }

    /// Appends `text` to `out` in the case in effect. Empty text leaves a pending `\u` for the
    /// text after it (`\u$2b` with `$2` empty gives `B`; probes/regexreplace-extended-escapes J3).
    fn push(&mut self, out: &mut String, text: &str) {
        let force = |force: Force, text: &str| match force {
            Force::Upper => upper_case(text),
            Force::Lower => lower_case(text),
        };
        let mut rest = text;
        if let (Some(next), Some(first)) = (self.next, text.chars().next()) {
            self.next = None;
            let (head, tail) = text.split_at(first.len_utf8());
            out.push_str(&force(next, head));
            rest = tail;
        }
        match self.mode {
            Some(Force::Lower) => self.pending.push_str(rest),
            Some(Force::Upper) => out.push_str(&force(Force::Upper, rest)),
            None => out.push_str(rest),
        }
    }
}

/// The escape a backslash starts.
enum Escape<'a> {
    /// One character.
    Char(char),
    /// A case-forcing escape.
    Case(CaseOp),
    /// The text of a `\Q...\E` quotation.
    Quoted(&'a str),
}

/// The value of the digits `digits` in `radix`; `None` when empty or past `char::MAX` (an overflow is `None` too).
fn char_from_digits(digits: &str, radix: u32) -> Option<char> {
    let trimmed = digits.trim_start_matches('0');
    let value = if trimmed.is_empty() {
        0
    } else {
        u32::from_str_radix(trimmed, radix).ok()?
    };
    // A surrogate or a value past U+10FFFF is no character: `#VALUE!` in Excel.
    char::from_u32(value)
}

/// The bracketed number of `\x{...}` or `\o{...}` at the start of `after` (the text after the
/// letter) and the bytes it takes; spaces and tabs around the digits are allowed
/// (probes/regexreplace-extended-escapes G11, I15-I18).
fn braced_char(after: &str, radix: u32) -> Option<(char, usize)> {
    let inner = after.strip_prefix('{')?;
    let close = inner.find('}')?;
    let digits = inner.get(..close)?.trim_matches([' ', '\t']);
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    Some((char_from_digits(digits, radix)?, close + 2))
}

/// A REGEXREPLACE replacement split once into its literal text and the capture groups it
/// refers to, so the length of a result is known before it is built. Excel reads `$n`, `${n}`,
/// `$name`, `${name}` and `$$` as PCRE2 does (a number is all the digits after the `$`, so `$1a`
/// is group 1 then "a"), and also `\n` for group n, `\\` for `\` and `\$` for `$`
/// (text.md "REGEXREPLACE replacement syntax"; probes/regex-pcre2-features B1-B14).
pub(super) struct Replacement {
    pieces: Vec<Piece>,
    /// The bytes of literal text in all the pieces.
    literal_len: usize,
    /// Each group referred to and how often.
    groups: Vec<(usize, usize)>,
    /// Whether a case-forcing escape is in it.
    cased: bool,
}

/// The decimal number `digits` spells, `None` past `usize`: a group no pattern has.
fn group_number(digits: &str) -> Option<usize> {
    digits.parse().ok()
}

/// The length of the run at the start of `s` of the characters `keep` accepts.
fn run_len(s: &str, keep: impl Fn(char) -> bool) -> usize {
    s.find(|c: char| !keep(c)).unwrap_or(s.len())
}

/// Whether `c` may appear in a group name.
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl Replacement {
    /// Parses `text` for a pattern compiled to `engine`; `None` when Excel gives `#VALUE!` for
    /// it: a group the pattern does not have (`$2`, `\2`, `${y}` with one unnamed group), a `$`
    /// that starts no reference, a trailing `\`, `\0` or a letter escape other than `\n` and
    /// `\t` (probes/regex-pcre2-features B15-B24, B30). A backslash before any other character
    /// is that character (`\-`, `\é`; E6, E7). The `?`s on slices of the text never fail: they
    /// cut at the ASCII `$`, `\` and digits the scan found.
    pub(super) fn new(engine: &Engine, text: &str) -> Option<Self> {
        let mut pieces = Vec::new();
        let mut literal = String::new();
        let mut rest = text;
        while let Some(at) = rest.find(['$', '\\']) {
            literal.push_str(rest.get(..at)?);
            let backslash = rest.get(at..)?.starts_with('\\');
            let after = rest.get(at + 1..)?;
            let next = after.chars().next()?;
            let reference = if next == '$' || (backslash && !next.is_ascii_alphanumeric()) {
                // `$$`, `\$`, `\\` and a backslash before any other punctuation are the
                // character itself (PCRE2's rule; only `$` and `\` are probed).
                literal.push(next);
                None
            } else if backslash && next.is_ascii_digit() {
                match Replacement::digit_escape(engine, after) {
                    Some(Ok(group)) => Some(group),
                    Some(Err((c, used))) => {
                        literal.push(c);
                        rest = after.get(used..)?;
                        continue;
                    }
                    None => return None,
                }
            } else if backslash {
                let (escape, used) = Replacement::letter_escape(after)?;
                match escape {
                    Escape::Char(c) => literal.push(c),
                    Escape::Quoted(text) => literal.push_str(text),
                    Escape::Case(op) => {
                        if !literal.is_empty() {
                            pieces.push(Piece::Literal(std::mem::take(&mut literal)));
                        }
                        // A repeated escape changes nothing, so `\U\U\U...` stays one piece
                        // and the per-match work stays bounded by the replacement's text.
                        if !matches!(pieces.last(), Some(Piece::Case(last)) if *last == op) {
                            pieces.push(Piece::Case(op));
                        }
                    }
                }
                rest = after.get(used..)?;
                continue;
            } else {
                Some(Replacement::dollar_group(engine, after)?)
            };
            let used = match reference {
                Some((index, used)) => {
                    if index >= engine.group_len() {
                        return None;
                    }
                    if !literal.is_empty() {
                        pieces.push(Piece::Literal(std::mem::take(&mut literal)));
                    }
                    pieces.push(Piece::Group(index));
                    used
                }
                None => next.len_utf8(),
            };
            rest = after.get(used..)?;
        }
        literal.push_str(rest);
        if !literal.is_empty() {
            pieces.push(Piece::Literal(literal));
        }
        let mut refs: Vec<usize> = pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Group(index) => Some(*index),
                Piece::Literal(_) | Piece::Case(_) => None,
            })
            .collect();
        refs.sort_unstable();
        let mut groups: Vec<(usize, usize)> = Vec::new();
        for index in refs {
            match groups.last_mut() {
                Some((last, count)) if *last == index => *count += 1,
                _ => groups.push((index, 1)),
            }
        }
        let literal_len = pieces
            .iter()
            .map(|piece| match piece {
                Piece::Literal(text) => text.len(),
                Piece::Group(_) | Piece::Case(_) => 0,
            })
            .sum();
        let cased = pieces.iter().any(|piece| matches!(piece, Piece::Case(_)));
        Some(Replacement {
            pieces,
            literal_len,
            groups,
            cased,
        })
    }

    /// What a backslash followed by the digits at the start of `after` is: `Ok((group, bytes))`
    /// for a group reference, `Err((char, bytes))` for an octal character, `None` for `#VALUE!`.
    /// PCRE2 reads the digits as a decimal number: below 10 or not past the pattern's groups it is
    /// a group (a missing one is `#VALUE!`), else three octal digits up to `\377` are a
    /// character. `\0` takes one or two more octal digits and needs at least one
    /// (probes/regexreplace-extended-escapes A37, C1-C14, D1-D11, E1-E7, I11-I14).
    fn digit_escape(engine: &Engine, after: &str) -> Option<Result<(usize, usize), (char, usize)>> {
        if let Some(zeroed) = after.strip_prefix('0') {
            let more = run_len(zeroed, |c| ('0'..='7').contains(&c)).min(2);
            if more == 0 {
                return None;
            }
            return Some(Err((char_from_digits(after.get(..=more)?, 8)?, more + 1)));
        }
        let digits = run_len(after, |c| c.is_ascii_digit());
        let index = group_number(after.get(..digits)?);
        if index.is_some_and(|i| i < 10 || i < engine.group_len()) {
            return Some(Ok((index?, digits)));
        }
        let code = after
            .get(..3)
            .filter(|d| d.bytes().all(|b| (b'0'..=b'7').contains(&b)))?;
        // Past `\377` is no character in Excel's replacement.
        let c = char_from_digits(code, 8).filter(|c| u32::from(*c) <= 0xff)?;
        Some(Err((c, 3)))
    }

    /// The escape for the letter at the start of `after` (the text after a backslash) and the
    /// bytes it takes; `None` for the letters Excel gives `#VALUE!` (`\q`, `\d`, `\x` with no
    /// hex digits...; probes/regexreplace-extended-escapes K1-K52).
    fn letter_escape(after: &str) -> Option<(Escape<'_>, usize)> {
        let letter = after.chars().next()?;
        let tail = after.get(1..)?;
        let simple = |c: char| Some((Escape::Char(c), 1));
        let case = |op: CaseOp| Some((Escape::Case(op), 1));
        match letter {
            'n' => simple('\n'),
            't' => simple('\t'),
            'r' => simple('\r'),
            'a' => simple('\u{7}'),
            'b' => simple('\u{8}'),
            'e' => simple('\u{1b}'),
            'f' => simple('\u{c}'),
            'v' => simple('\u{b}'),
            'U' => case(CaseOp::Upper),
            'L' => case(CaseOp::Lower),
            'u' => case(CaseOp::UpperNext),
            'l' => case(CaseOp::LowerNext),
            'E' => case(CaseOp::End),
            'Q' => {
                // Up to the first `\E` (a lone `\` before it is literal), or the end.
                let end = tail.find("\\E").unwrap_or(tail.len());
                let used = (end + 3).min(tail.len() + 1);
                Some((Escape::Quoted(tail.get(..end)?), used))
            }
            'x' => {
                if let Some((c, used)) = braced_char(tail, 16) {
                    return Some((Escape::Char(c), used + 1));
                }
                let digits = run_len(tail, |c| c.is_ascii_hexdigit()).min(2);
                let c = char_from_digits(tail.get(..digits)?, 16).filter(|_| digits > 0)?;
                Some((Escape::Char(c), digits + 1))
            }
            'o' => braced_char(tail, 8).map(|(c, used)| (Escape::Char(c), used + 1)),
            'c' => {
                // PCRE2's control character: the upper-cased ASCII character XOR 0x40.
                let control = tail.chars().next().filter(char::is_ascii)?;
                let code = u32::from(control.to_ascii_uppercase()) ^ 0x40;
                Some((Escape::Char(char::from_u32(code)?), 2))
            }
            _ => None,
        }
    }

    /// The group a `$` followed by `after` refers to and the bytes of `after` the reference
    /// takes; `None` when it is no reference or names no group of the pattern.
    fn dollar_group(engine: &Engine, after: &str) -> Option<(usize, usize)> {
        let resolve = |name: &str| {
            if name.bytes().all(|b| b.is_ascii_digit()) {
                group_number(name)
            } else {
                engine.group_index(name)
            }
        };
        let digits = run_len(after, |c| c.is_ascii_digit());
        if digits > 0 {
            return Some((group_number(after.get(..digits)?)?, digits));
        }
        if let Some(braced) = after.strip_prefix('{') {
            let name = run_len(braced, is_name_char);
            if name > 0 && braced.get(name..)?.starts_with('}') {
                return Some((resolve(braced.get(..name)?)?, name + 2));
            }
            return None;
        }
        if after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            let name = run_len(after, is_name_char);
            return Some((resolve(after.get(..name)?)?, name));
        }
        None
    }

    /// The whole replacement when it refers to no group (an empty one has no pieces).
    fn literal_only(&self) -> Option<&str> {
        match self.pieces.as_slice() {
            [] => Some(""),
            [Piece::Literal(literal)] => Some(literal),
            _ => None,
        }
    }

    /// `bytes` of text as long as case forcing may make them: case mapping may lengthen a
    /// character's UTF-8 form by half at most (2 bytes to 3: ȿ).
    fn grown(&self, bytes: usize) -> usize {
        if self.cased {
            bytes.saturating_add(bytes.div_ceil(2))
        } else {
            bytes
        }
    }

    /// An upper bound on the bytes of this replacement for the match whose groups are `caps`,
    /// saturating.
    pub(super) fn expanded_len(&self, caps: &Captures) -> usize {
        let span_len = |index| caps.get_group(index).map_or(0, |m| m.len());
        self.groups
            .iter()
            .map(|&(index, count)| self.grown(span_len(index).saturating_mul(count)))
            .fold(self.grown(self.literal_len), usize::saturating_add)
    }

    /// The byte length of `REGEXREPLACE(text, re, replacement)`, saturating, without building it.
    pub(super) fn result_len(&self, search: &mut Search<'_>, text: &str) -> Result<usize, Spent> {
        let mut len = text.len();
        if self.groups.is_empty() {
            let literal_len = self.grown(self.literal_len);
            search.for_each_match(text, |m| {
                len = len.saturating_add(literal_len).saturating_sub(m.len());
            })?;
            return Ok(len);
        }
        let mut caps = search.engine.create_captures();
        search.for_each_captures(text, &mut caps, |caps| {
            let whole = caps.get_match().map_or(0, |m| m.len());
            len = len
                .saturating_add(self.expanded_len(caps))
                .saturating_sub(whole);
        })?;
        Ok(len)
    }

    /// Appends this replacement for the match whose groups in `text` are `caps` to `out`. Case
    /// forcing starts afresh with every match (probes/regexreplace-extended-escapes J5, J7, L1).
    fn expand(&self, out: &mut String, text: &str, caps: &Captures) {
        let mut case = CaseState::default();
        for piece in &self.pieces {
            match piece {
                Piece::Literal(literal) => case.push(out, literal),
                Piece::Group(index) => {
                    if let Some(span) = caps.get_group(*index) {
                        case.push(out, text.get(span.range()).unwrap_or_default());
                    }
                }
                Piece::Case(op) => case.apply(*op, out),
            }
        }
        case.finish(out);
    }

    /// `text` with the match `m`, whose groups are `caps`, replaced; `len` bytes long at most.
    pub(super) fn replace_one(&self, text: &str, m: Match, caps: &Captures, len: usize) -> String {
        let mut out = String::with_capacity(len);
        out.push_str(text.get(..m.start()).unwrap_or_default());
        self.expand(&mut out, text, caps);
        out.push_str(text.get(m.end()..).unwrap_or_default());
        out
    }

    /// `REGEXREPLACE(text, re, replacement)`, `len` bytes long.
    pub(super) fn replace(
        &self,
        search: &mut Search<'_>,
        text: &str,
        len: usize,
    ) -> Result<String, Spent> {
        let mut out = String::with_capacity(len);
        let mut last = 0;
        if let Some(literal) = self.literal_only() {
            search.for_each_match(text, |m| {
                out.push_str(text.get(last..m.start()).unwrap_or_default());
                out.push_str(literal);
                last = m.end();
            })?;
            out.push_str(text.get(last..).unwrap_or_default());
            return Ok(out);
        }
        let mut caps = search.engine.create_captures();
        search.for_each_captures(text, &mut caps, |caps| {
            if let Some(m) = caps.get_match() {
                out.push_str(text.get(last..m.start()).unwrap_or_default());
                self.expand(&mut out, text, caps);
                last = m.end();
            }
        })?;
        out.push_str(text.get(last..).unwrap_or_default());
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::{CaseOp, CaseState};

    fn forced(ops: &[(Option<CaseOp>, &str)]) -> String {
        let mut state = CaseState::default();
        let mut out = String::new();
        for (op, text) in ops {
            if let Some(op) = op {
                state.apply(*op, &mut out);
            }
            state.push(&mut out, text);
        }
        state.finish(&mut out);
        out
    }

    // T-1897: oracle cells probes/regexreplace-extended-escapes B1-B11, I1-I5, J3.
    #[test]
    fn case_forcing_follows_excel() {
        use CaseOp::{End, Lower, LowerNext, Upper, UpperNext};
        assert_eq!(
            forced(&[(Some(Upper), "ab"), (Some(LowerNext), "cd")]),
            "ABcd"
        );
        assert_eq!(
            forced(&[(Some(LowerNext), ""), (Some(Upper), "abc")]),
            "aBC"
        );
        assert_eq!(
            forced(&[(Some(Lower), "AB"), (Some(UpperNext), "cd")]),
            "abCd"
        );
        assert_eq!(forced(&[(Some(UpperNext), ""), (None, "b")]), "B");
        assert_eq!(forced(&[(Some(UpperNext), "x"), (Some(End), "y")]), "Xy");
    }

    // T-1934: oracle cells probes/regexreplace-final-sigma A2, A8, A10: a `\L` run is lowered whole.
    #[test]
    fn lowered_run_sees_the_text_after_a_sigma() {
        use CaseOp::{End, Lower};
        assert_eq!(forced(&[(Some(Lower), "ΑΣ"), (None, "x")]), "ασx");
        assert_eq!(forced(&[(Some(Lower), "Α"), (None, "Σ")]), "ας");
        assert_eq!(forced(&[(Some(Lower), "Α"), (Some(End), "Σ")]), "αΣ");
    }

    #[test]
    fn case_mapping_can_lengthen_text_by_half() {
        // ȿ (2 bytes) upper-cases to Ɀ (3 bytes).
        let mut state = CaseState::default();
        let mut out = String::new();
        state.apply(CaseOp::Upper, &mut out);
        state.push(&mut out, "ȿ");
        assert_eq!(out.len(), 3);
    }
}
