use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use regex::{Regex, RegexBuilder};
#[cfg(target_arch = "wasm32")]
use regex_lite::{Regex, RegexBuilder};

use super::{lifted::COMPARED_BYTES_PER_VISIT, util::within_text_budget};
use crate::{
    calc_result::CalcResult,
    cast::MAX_TEXT_UNITS,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
    Text,
};

/// The size limits a pattern is compiled under, smallest first. A pattern is compiled under the
/// next one only when it is too big for the one before; the last is the regex crate's default,
/// so every pattern that compiled before T-1272 still compiles. Compile time, memory and the
/// cost of a search grow with the limit: in a release build the largest patterns the last
/// admits take 70-90 ms to compile (`\w{200}`), a failed attempt about as long as its limit
/// admits, and searching with one costs up to about 3 µs a byte of text when the lazy DFA gives
/// up (T-1272 performance review).
const SIZE_LIMITS: [usize; 5] = [1 << 16, 1 << 18, 1 << 20, 4 << 20, 10 << 20];

/// Bytes of size limit one call may compile under, attempts that were too big included: 2,048
/// small patterns, 409 of 256 KiB or 8 of the largest, about 1 s of compiling. A call that needs
/// more is `#ERROR!` (DEVIATION, decision 0014: Excel has no such limit).
const COMPILE_BUDGET: usize = 128 << 20;

/// Bytes of size limit one call may keep compiled, bounding its memory (CLAUDE.md directive 5):
/// 1,024 small patterns or 6 of the largest. A call compiles each distinct pattern once and
/// evicts none, so patterns that cycle cost no recompiles; one with more distinct patterns is
/// `#ERROR!` (DEVIATION, decision 0014).
const HELD_LIMIT: usize = 64 << 20;

/// Bytes of text searched times the size limit of the pattern searched with (at least
/// [`MIN_SEARCH_WEIGHT`]), per call: about 440,000 bytes with the largest patterns (about 1.3 s
/// at 3 µs a byte), 4.4 million with a 1 MiB one, so even an unlifted search of a full 32,767-character cell fits twice over. A call
/// past it is `#ERROR!` (DEVIATION, decision 0014). Search time grows with the compiled size
/// once the lazy DFA gives up, so the limit stands in for it.
const SEARCH_BUDGET: u64 = 1 << 42;

/// The least a byte of searched text is charged: a pattern that matches at every position (an
/// empty one) costs about 40 ns a match whatever its size, so the smallest limits are charged as
/// 256 KiB, about 17 million bytes a call.
const MIN_SEARCH_WEIGHT: usize = 1 << 18;

/// Whether a compile error says only that the pattern is too big for its size limit.
#[cfg(not(target_arch = "wasm32"))]
fn too_big(error: &regex::Error) -> bool {
    matches!(error, regex::Error::CompiledTooBig(_))
}

/// `regex_lite` does not tell a size error from a syntax error: try the next limit either way.
#[cfg(target_arch = "wasm32")]
fn too_big(_: &regex_lite::Error) -> bool {
    true
}

/// A compiled pattern (`None` when invalid) and the size limit it holds.
struct Compiled {
    regex: Option<Regex>,
    limit: usize,
}

/// The patterns one REGEX* call has compiled and the work it may still do: each distinct
/// pattern is compiled once, within [`COMPILE_BUDGET`] and [`HELD_LIMIT`], and every search is
/// charged against [`SEARCH_BUDGET`].
struct PatternCache {
    entries: HashMap<String, Compiled>,
    held: usize,
    budget: usize,
    search_budget: u64,
    exhausted: bool,
}

impl PatternCache {
    fn new() -> Self {
        PatternCache {
            entries: HashMap::new(),
            held: 0,
            budget: COMPILE_BUDGET,
            search_budget: SEARCH_BUDGET,
            exhausted: false,
        }
    }

    /// The regex for `pattern`, charged for `searches` passes over `haystack` bytes; or the
    /// `#VALUE!` of an invalid pattern; or `#ERROR!` once the call's work is spent.
    fn get(
        &mut self,
        pattern: &str,
        haystack: usize,
        searches: u64,
        cell: CellReferenceIndex,
    ) -> Result<&Regex, CalcResult> {
        if self.exhausted {
            return Err(too_much_regex_work(cell));
        }
        if !self.entries.contains_key(pattern) {
            let compiled = self.compile(pattern, cell)?;
            if self.held + compiled.limit > HELD_LIMIT {
                self.exhausted = true;
                return Err(too_much_regex_work(cell));
            }
            self.held += compiled.limit;
            self.entries.insert(pattern.to_string(), compiled);
        }
        let Some(entry) = self.entries.get(pattern) else {
            return Err(too_much_regex_work(cell));
        };
        let Some(regex) = entry.regex.as_ref() else {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid regular expression".to_string(),
            ));
        };
        let cost = to_u64(haystack.max(1))
            .saturating_mul(searches)
            .saturating_mul(to_u64(entry.limit.max(MIN_SEARCH_WEIGHT)));
        match self.search_budget.checked_sub(cost) {
            Some(left) => {
                self.search_budget = left;
                Ok(regex)
            }
            None => {
                self.exhausted = true;
                Err(too_much_regex_work(cell))
            }
        }
    }

    /// Compiles `pattern` under the smallest size limit it fits, charging each attempt.
    fn compile(&mut self, pattern: &str, cell: CellReferenceIndex) -> Result<Compiled, CalcResult> {
        let mut regex = None;
        let mut held = 0;
        for limit in SIZE_LIMITS {
            if self.budget < limit {
                self.exhausted = true;
                return Err(too_much_regex_work(cell));
            }
            self.budget -= limit;
            held = limit;
            match RegexBuilder::new(pattern).size_limit(limit).build() {
                Ok(compiled) => {
                    regex = Some(compiled);
                    break;
                }
                Err(error) if too_big(&error) => {}
                Err(_) => break,
            }
        }
        Ok(Compiled { regex, limit: held })
    }

    /// The call's result: `#ERROR!` when its work ran out in any element.
    fn finish(&self, result: CalcResult, cell: CellReferenceIndex) -> CalcResult {
        if self.exhausted {
            too_much_regex_work(cell)
        } else {
            result
        }
    }
}

fn to_u64(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

fn too_much_regex_work(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::ERROR,
        cell,
        "Too much regular expression work".to_string(),
    )
}

/// A REGEXREPLACE replacement split once into its literal text and the capture groups it
/// refers to, with the regex crates' own `$name` rules (`regex_automata`'s interpolation), so
/// the length of a result is known before it is built.
struct Replacement {
    literal_len: usize,
    /// Each group referred to and how often.
    groups: Vec<(usize, usize)>,
}

impl Replacement {
    fn new(re: &Regex, replacement: &str) -> Self {
        let mut literal = String::new();
        let mut refs = Vec::new();
        regex_automata::util::interpolate::string(
            replacement,
            |index, _| refs.push(index),
            |name| re.capture_names().position(|n| n == Some(name)),
            &mut literal,
        );
        refs.sort_unstable();
        let mut groups: Vec<(usize, usize)> = Vec::new();
        for index in refs {
            match groups.last_mut() {
                Some((last, count)) if *last == index => *count += 1,
                _ => groups.push((index, 1)),
            }
        }
        Replacement {
            literal_len: literal.len(),
            groups,
        }
    }

    /// The byte length of `REGEXREPLACE(text, re, replacement)`, saturating, without building it.
    fn result_len(&self, re: &Regex, text: &str) -> usize {
        let mut len = text.len();
        if self.groups.is_empty() {
            for m in re.find_iter(text) {
                len = len.saturating_add(self.literal_len).saturating_sub(m.len());
            }
            return len;
        }
        for caps in re.captures_iter(text) {
            let matched = caps.get(0).map_or(0, |m| m.len());
            let expanded = self
                .groups
                .iter()
                .map(|&(index, count)| caps.get(index).map_or(0, |m| m.len()).saturating_mul(count))
                .fold(self.literal_len, usize::saturating_add);
            len = len.saturating_add(expanded).saturating_sub(matched);
        }
        len
    }
}

/// The most bytes a REGEXREPLACE result may have: a text of 32,767 UTF-16 units has at most 3
/// bytes a unit, so a longer result is over Excel's text limit whatever its characters.
const MAX_REPLACED_BYTES: usize = 3 * MAX_TEXT_UNITS;

/// A REGEXREPLACE result over 32,767 UTF-16 units is `#VALUE!`, as SUBSTITUTE's is (UNVERIFIED
/// for REGEXREPLACE, T-1817), and so is a lifted call whose results pass the text broadcast cap.
fn too_long(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "REGEXREPLACE result is too long".to_string(),
    )
}

impl Model {
    /// The text and pattern of one REGEX* call element, coerced to text left to right.
    fn regex_operands(
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(String, String), CalcResult> {
        let text = Self::cast_string_ref(&values[0], cell)?;
        let pattern = Self::cast_string_ref(&values[1], cell)?;
        Ok((text, pattern))
    }

    /// REGEXEXTRACT(text, `regular_expression`, [`return_all`])
    ///
    /// Returns the first substring of `text` that matches `regular_expression`.
    /// If the pattern contains capture groups, returns the content of the first group.
    /// When the optional `return_all` argument is 1, returns a horizontal array of
    /// every match (one match per column) instead of only the first one.
    /// Returns #N/A if there is no match, #VALUE! if the regex is invalid.
    /// The text, pattern and `return_all` lift over a range or an array
    /// (probes/m7-argument-modes F20, F22).
    pub(crate) fn fn_regexextract(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let (operands, _) = self.lift_text_operands(args, &[], cell);
        let has_return_all = args.len() == 3;
        let mut cache = PatternCache::new();
        let result = self.lift_text_call(
            operands,
            cell,
            &[],
            COMPARED_BYTES_PER_VISIT,
            |model, values, first_only| {
                let (text, pattern) = match Model::regex_operands(values, cell) {
                    Ok(operands) => operands,
                    Err(error) => return error,
                };
                let return_all = if has_return_all {
                    match model.cast_number_ref(&values[2], cell) {
                        Ok(n) => n != 0.0,
                        Err(e) => return e,
                    }
                } else {
                    false
                };
                let re = match cache.get(&pattern, text.len(), 1, cell) {
                    Ok(re) => re,
                    Err(error) => return error,
                };
                if return_all && first_only {
                    // A lifted element keeps the first match only (T-1272 performance review).
                    return match re.find(&text) {
                        Some(m) => CalcResult::String(m.as_str().to_string().into()),
                        None => {
                            CalcResult::new_error(Error::NA, cell, "No match found".to_string())
                        }
                    };
                }
                if return_all {
                    let row: Vec<ArrayNode> = re
                        .find_iter(&text)
                        .map(|m| ArrayNode::String(m.as_str().to_string().into()))
                        .collect();
                    if row.is_empty() {
                        return CalcResult::new_error(
                            Error::NA,
                            cell,
                            "No match found".to_string(),
                        );
                    }
                    CalcResult::Array(vec![row])
                } else {
                    match re.captures(&text) {
                        None => {
                            CalcResult::new_error(Error::NA, cell, "No match found".to_string())
                        }
                        Some(caps) => {
                            // If there is at least one explicit capture group, return group 1.
                            // Otherwise return the full match (group 0).
                            let matched = if caps.len() > 1 {
                                caps.get(1).map_or("", |m| m.as_str())
                            } else {
                                caps.get(0).map_or("", |m| m.as_str())
                            };
                            CalcResult::String(matched.to_string().into())
                        }
                    }
                }
            },
        );
        cache.finish(result, cell)
    }

    /// REGEXREPLACE(text, `regular_expression`, replacement)
    ///
    /// Replaces every substring of `text` that matches `regular_expression` with
    /// `replacement`. Returns #VALUE! if the regex is invalid. All three arguments lift over a
    /// range or an array (probes/m7-argument-modes F21).
    pub(crate) fn fn_regexreplace(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let (operands, _) = self.lift_text_operands(args, &[], cell);
        let mut cache = PatternCache::new();
        let mut total = 0;
        let mut over_budget = false;
        let result = self.lift_text_call(
            operands,
            cell,
            &[],
            COMPARED_BYTES_PER_VISIT,
            |_, values, _| {
                let (text, pattern) = match Model::regex_operands(values, cell) {
                    Ok(operands) => operands,
                    Err(error) => return error,
                };
                let replacement = match Model::cast_string_ref(&values[2], cell) {
                    Ok(s) => s,
                    Err(e) => return e,
                };
                // Two passes over the text: one measures the result, one builds it.
                let re = match cache.get(&pattern, text.len(), 2, cell) {
                    Ok(re) => re,
                    Err(error) => return error,
                };
                let len = Replacement::new(re, &replacement).result_len(re, &text);
                if !within_text_budget(&mut total, len) {
                    over_budget = true;
                    return too_long(cell);
                }
                if len > MAX_REPLACED_BYTES {
                    return too_long(cell);
                }
                let replaced = Text::from(re.replace_all(&text, replacement.as_str()).as_ref());
                if replaced.len_utf16() > MAX_TEXT_UNITS {
                    return too_long(cell);
                }
                CalcResult::String(replaced)
            },
        );
        if over_budget {
            // As a text broadcast past its cap: one `#VALUE!` (text.md, first entry).
            return cache.finish(too_long(cell), cell);
        }
        cache.finish(result, cell)
    }

    /// REGEXTEST(text, `regular_expression`)
    ///
    /// Returns `TRUE` if `regular_expression` matches anywhere in `text`, `FALSE`
    /// otherwise. Returns #VALUE! if the regex is invalid. Both arguments lift over a range or
    /// an array (probes/m7-argument-modes F18, F19).
    pub(crate) fn fn_regextest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let (operands, _) = self.lift_text_operands(args, &[], cell);
        let mut cache = PatternCache::new();
        let result = self.lift_text_call(
            operands,
            cell,
            &[],
            COMPARED_BYTES_PER_VISIT,
            |_, values, _| {
                let (text, pattern) = match Model::regex_operands(values, cell) {
                    Ok(operands) => operands,
                    Err(error) => return error,
                };
                match cache.get(&pattern, text.len(), 1, cell) {
                    Ok(re) => CalcResult::Boolean(re.is_match(&text)),
                    Err(error) => error,
                }
            },
        );
        cache.finish(result, cell)
    }
}

#[cfg(test)]
mod tests {
    use super::{CalcResult, PatternCache, COMPILE_BUDGET, SEARCH_BUDGET};
    use crate::expressions::{token::Error, types::CellReferenceIndex};

    const CELL: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    fn is_error(result: Result<&super::Regex, CalcResult>) -> bool {
        matches!(
            result,
            Err(CalcResult::Error {
                error: Error::ERROR,
                ..
            })
        )
    }

    // T-1272 performance review: a search is charged its text's bytes times the pattern's size
    // limit before it runs. `\w{200}b` over 4,000 texts of 3,000 bytes took 38.5 s; at the
    // largest limit the budget is about 440,000 bytes.
    #[test]
    fn searches_are_charged_by_size_limit() {
        let mut cache = PatternCache::new();
        assert!(cache.get("a", 1 << 20, 2, CELL).is_ok());
        assert!(is_error(cache.get("a", 1 << 30, 1, CELL)));
        // Spent for the rest of the call.
        assert!(is_error(cache.get("a", 1, 1, CELL)));

        let mut cache = PatternCache::new();
        // 400,000 x 10 MiB is just under the budget; 100,000 more bytes are not.
        const _: () = assert!(400_000 * (10 << 20) < SEARCH_BUDGET);
        assert!(cache.get("\\w{200}b", 400_000, 1, CELL).is_ok());
        assert!(is_error(cache.get("\\w{200}b", 100_000, 1, CELL)));
    }

    // The compile budget charges every attempt, so past it the call is spent; an invalid
    // pattern is the `#VALUE!` of its own element.
    #[test]
    fn compiles_are_budgeted() {
        let mut cache = PatternCache::new();
        assert!(matches!(
            cache.get("(", 1, 1, CELL),
            Err(CalcResult::Error {
                error: Error::VALUE,
                ..
            })
        ));
        let small = COMPILE_BUDGET / (1 << 16);
        let mut spent = false;
        for i in 0..=small {
            spent |= is_error(cache.get(&format!("a{i}"), 1, 1, CELL));
        }
        assert!(spent);
    }
}
