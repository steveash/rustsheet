use std::collections::HashMap;

use super::{
    lifted::COMPARED_BYTES_PER_VISIT,
    regex_engine::{BuildFailure, Caches, Engine, Search, Spent},
    util::within_text_budget,
};
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
/// so every pattern that compiled before T-1272 still compiles. Compile time grows with the
/// limit: in a release build the largest patterns the last admits take 70-90 ms to compile
/// (`\w{200}`), a failed attempt about as long as its limit admits.
const SIZE_LIMITS: [usize; 5] = [1 << 16, 1 << 18, 1 << 20, 4 << 20, 10 << 20];

/// Bytes of size limit one call may compile under, attempts that were too big included: 2,048
/// small patterns, 409 of 256 KiB or 8 of the largest, about 1 s of compiling. A call that needs
/// more is `#ERROR!` (DEVIATION, decision 0014: Excel has no such limit).
const COMPILE_BUDGET: usize = 128 << 20;

/// Bytes of heap one call's compiled patterns and their search caches may hold, as measured
/// after each search (CLAUDE.md directive 5). A call compiles each distinct pattern once and
/// evicts none, so patterns that cycle cost no recompiles; one that holds more is `#ERROR!`
/// (DEVIATION, decision 0014).
const HELD_LIMIT: usize = 64 << 20;

/// The search work one call may do, in the units of `regex_engine` (about a nanosecond each):
/// about 1 s of the slowest searches, or 250 MB of text a simple pattern scans. A call past it
/// is `#ERROR!` (DEVIATION, decision 0014).
const SEARCH_BUDGET: u64 = 1 << 30;

/// A compiled pattern (`None` when invalid), its search caches and the heap they last held.
struct Compiled {
    engine: Option<(Engine, Caches)>,
    held: usize,
}

/// The patterns one REGEX* call has compiled and the work it may still do: each distinct
/// pattern is compiled once, within [`COMPILE_BUDGET`] and [`HELD_LIMIT`], and every search is
/// charged against [`SEARCH_BUDGET`].
struct PatternCache {
    entries: HashMap<String, Compiled>,
    held: usize,
    budget: usize,
    work: u64,
    exhausted: bool,
}

impl PatternCache {
    fn new() -> Self {
        PatternCache {
            entries: HashMap::new(),
            held: 0,
            budget: COMPILE_BUDGET,
            work: SEARCH_BUDGET,
            exhausted: false,
        }
    }

    /// Runs `searches` with `pattern`: its result; or the `#VALUE!` of an invalid pattern; or
    /// `#ERROR!` once the call's work is spent.
    fn with<R>(
        &mut self,
        pattern: &str,
        cell: CellReferenceIndex,
        searches: impl FnOnce(&mut Search<'_>) -> Result<R, Spent>,
    ) -> Result<R, CalcResult> {
        if self.exhausted {
            return Err(too_much_regex_work(cell));
        }
        if !self.entries.contains_key(pattern) {
            let engine = self.compile(pattern, cell)?;
            self.entries
                .insert(pattern.to_string(), Compiled { engine, held: 0 });
        }
        let Some(entry) = self.entries.get_mut(pattern) else {
            return Err(too_much_regex_work(cell));
        };
        let Some((engine, caches)) = entry.engine.as_mut() else {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid regular expression".to_string(),
            ));
        };
        let result = searches(&mut Search {
            engine,
            caches,
            work: &mut self.work,
        });
        let held = engine.memory_usage(caches);
        self.held = self.held.saturating_sub(entry.held).saturating_add(held);
        entry.held = held;
        match result {
            Ok(result) if self.held <= HELD_LIMIT => Ok(result),
            _ => {
                self.exhausted = true;
                Err(too_much_regex_work(cell))
            }
        }
    }

    /// Compiles `pattern` under the smallest size limit it fits, charging each attempt; `None`
    /// when it is invalid.
    fn compile(
        &mut self,
        pattern: &str,
        cell: CellReferenceIndex,
    ) -> Result<Option<(Engine, Caches)>, CalcResult> {
        for limit in SIZE_LIMITS {
            if self.budget < limit {
                self.exhausted = true;
                return Err(too_much_regex_work(cell));
            }
            self.budget -= limit;
            match Engine::new(pattern, limit) {
                Ok(engine) => {
                    let caches = engine.create_caches();
                    return Ok(Some((engine, caches)));
                }
                Err(BuildFailure::TooBig) => {}
                Err(BuildFailure::Invalid) => break,
            }
        }
        Ok(None)
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
struct Replacement<'a> {
    text: &'a str,
    /// The replacement with no groups expanded: all of it when it refers to none.
    literal: String,
    /// Each group referred to and how often.
    groups: Vec<(usize, usize)>,
}

impl<'a> Replacement<'a> {
    fn new(engine: &Engine, text: &'a str) -> Self {
        let mut literal = String::new();
        let mut refs = Vec::new();
        regex_automata::util::interpolate::string(
            text,
            |index, _| refs.push(index),
            |name| engine.group_index(name),
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
            text,
            literal,
            groups,
        }
    }

    /// The byte length of `REGEXREPLACE(text, re, replacement)`, saturating, without building it.
    fn result_len(&self, search: &mut Search<'_>, text: &str) -> Result<usize, Spent> {
        let mut len = text.len();
        let literal_len = self.literal.len();
        if self.groups.is_empty() {
            search.for_each_match(text, |m| {
                len = len.saturating_add(literal_len).saturating_sub(m.len());
            })?;
            return Ok(len);
        }
        let mut caps = search.engine.create_captures();
        search.for_each_captures(text, &mut caps, |caps| {
            let span_len = |index| caps.get_group(index).map_or(0, |m| m.len());
            let expanded = self
                .groups
                .iter()
                .map(|&(index, count)| span_len(index).saturating_mul(count))
                .fold(literal_len, usize::saturating_add);
            len = len.saturating_add(expanded).saturating_sub(span_len(0));
        })?;
        Ok(len)
    }

    /// `REGEXREPLACE(text, re, replacement)`, `len` bytes long.
    fn replace(&self, search: &mut Search<'_>, text: &str, len: usize) -> Result<String, Spent> {
        let mut out = String::with_capacity(len);
        let mut last = 0;
        if self.groups.is_empty() {
            search.for_each_match(text, |m| {
                out.push_str(text.get(last..m.start()).unwrap_or_default());
                out.push_str(&self.literal);
                last = m.end();
            })?;
        } else {
            let mut caps = search.engine.create_captures();
            search.for_each_captures(text, &mut caps, |caps| {
                if let Some(m) = caps.get_match() {
                    out.push_str(text.get(last..m.start()).unwrap_or_default());
                    caps.interpolate_string_into(text, self.text, &mut out);
                    last = m.end();
                }
            })?;
        }
        out.push_str(text.get(last..).unwrap_or_default());
        Ok(out)
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
                let found = cache.with(&pattern, cell, |search| {
                    if return_all && first_only {
                        // A lifted element keeps the first match only (T-1272 performance
                        // review).
                        return Ok(search.find(&text)?.map(|m| vec![m.range()]));
                    }
                    if return_all {
                        let mut all = Vec::new();
                        search.for_each_match(&text, |m| all.push(m.range()))?;
                        return Ok(Some(all).filter(|all| !all.is_empty()));
                    }
                    let mut caps = search.engine.create_captures();
                    if !search.captures(&text, &mut caps)? {
                        return Ok(None);
                    }
                    // If there is at least one explicit capture group, return group 1.
                    // Otherwise return the full match (group 0).
                    let group = usize::from(caps.group_len() > 1);
                    Ok(Some(vec![caps
                        .get_group(group)
                        .map_or(0..0, |m| m.range())]))
                });
                let piece =
                    |range: std::ops::Range<usize>| Text::from(text.get(range).unwrap_or_default());
                match found {
                    Err(error) => error,
                    Ok(None) => {
                        CalcResult::new_error(Error::NA, cell, "No match found".to_string())
                    }
                    Ok(Some(ranges)) if return_all && !first_only => {
                        CalcResult::Array(vec![ranges
                            .into_iter()
                            .map(|range| ArrayNode::String(piece(range)))
                            .collect()])
                    }
                    Ok(Some(ranges)) => ranges
                        .into_iter()
                        .next()
                        .map_or(CalcResult::String(Text::from("")), |range| {
                            CalcResult::String(piece(range))
                        }),
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
                let replaced = cache.with(&pattern, cell, |search| {
                    let replacement = Replacement::new(search.engine, &replacement);
                    let len = replacement.result_len(search, &text)?;
                    if !within_text_budget(&mut total, len) {
                        over_budget = true;
                        return Ok(None);
                    }
                    if len > MAX_REPLACED_BYTES {
                        return Ok(None);
                    }
                    Ok(Some(replacement.replace(search, &text, len)?))
                });
                let replaced = match replaced {
                    Ok(Some(replaced)) => Text::from(replaced.as_str()),
                    Ok(None) => return too_long(cell),
                    Err(error) => return error,
                };
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
                match cache.with(&pattern, cell, |search| search.is_match(&text)) {
                    Ok(matched) => CalcResult::Boolean(matched),
                    Err(error) => error,
                }
            },
        );
        cache.finish(result, cell)
    }
}

#[cfg(test)]
mod tests {
    use super::{CalcResult, PatternCache, COMPILE_BUDGET};
    use crate::expressions::{token::Error, types::CellReferenceIndex};

    const CELL: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    fn is_error<R>(result: &Result<R, CalcResult>) -> bool {
        matches!(
            result,
            Err(CalcResult::Error {
                error: Error::ERROR,
                ..
            })
        )
    }

    // T-1272 performance review: a simple pattern is charged what its lazy DFA scans, so a
    // long column fits; the call is spent once the work runs out, and stays spent.
    #[test]
    fn searches_are_charged_by_work_done() {
        let text = "y".repeat(1 << 20);
        let mut cache = PatternCache::new();
        for _ in 0..100 {
            assert!(matches!(
                cache.with("x", CELL, |search| search.is_match(&text)),
                Ok(false)
            ));
        }
        let mut cache = PatternCache::new();
        cache.work = 1_000;
        assert!(is_error(
            &cache.with("x", CELL, |search| search.is_match(&text))
        ));
        assert!(is_error(
            &cache.with("x", CELL, |search| search.is_match("x"))
        ));
    }

    // T-1272 performance review: a compact pattern whose lazy DFA thrashes gives up and runs on
    // the PikeVM, charged its NFA's states for every byte, so a call over many such texts stops
    // early instead of running for seconds.
    #[test]
    fn thrashing_patterns_are_charged_by_state_count() {
        let text: String = (1..=30_000u64)
            .map(|i| {
                if (i * i * 7919 + i * 104_729) % 30_011 > 15_005 {
                    'a'
                } else {
                    'b'
                }
            })
            .collect();
        let pattern = "a[ab]{100}c|b[ab]{100}c";
        let mut cache = PatternCache::new();
        let mut searches = 0;
        while cache
            .with(pattern, CELL, |search| search.is_match(&text))
            .is_ok()
        {
            searches += 1;
        }
        // About 30,000 bytes x 210 states x 5 units a search once the DFA gives up.
        assert!(searches < 100, "{searches} searches");
        assert!(searches > 2, "{searches} searches");
    }

    // The compile budget charges every attempt, so past it the call is spent; an invalid
    // pattern is the `#VALUE!` of its own element.
    #[test]
    fn compiles_are_budgeted() {
        let mut cache = PatternCache::new();
        assert!(matches!(
            cache.with("(", CELL, |search| search.is_match("")),
            Err(CalcResult::Error {
                error: Error::VALUE,
                ..
            })
        ));
        let small = COMPILE_BUDGET / (1 << 16);
        let mut spent = false;
        for i in 0..=small {
            spent |= is_error(&cache.with(&format!("a{i}"), CELL, |search| search.is_match("")));
        }
        assert!(spent);
    }
}
