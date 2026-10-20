use std::collections::HashMap;

use regex_automata::Match;

use super::{
    lifted::COMPARED_BYTES_PER_VISIT,
    regex_engine::{BuildFailure, Caches, Engine, Search, Spent},
    regex_posix::unicode_posix_classes,
    regex_replacement::Replacement,
    util::within_text_budget,
};
use crate::{
    calc_result::CalcResult,
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
/// pattern is compiled once for each case sensitivity, within [`COMPILE_BUDGET`] and
/// [`HELD_LIMIT`], and every search is charged against [`SEARCH_BUDGET`].
struct PatternCache {
    /// The case-sensitive patterns, then the case-insensitive ones.
    entries: [HashMap<String, Compiled>; 2],
    held: usize,
    budget: usize,
    work: u64,
    exhausted: bool,
}

impl PatternCache {
    fn new() -> Self {
        PatternCache {
            entries: [HashMap::new(), HashMap::new()],
            held: 0,
            budget: COMPILE_BUDGET,
            work: SEARCH_BUDGET,
            exhausted: false,
        }
    }

    /// Runs `searches` with `pattern`, ignoring case when `case_insensitive`: its result; or the
    /// `#VALUE!` of an invalid pattern; or `#ERROR!` once the call's work is spent.
    fn with<R>(
        &mut self,
        pattern: &str,
        case_insensitive: bool,
        cell: CellReferenceIndex,
        searches: impl FnOnce(&mut Search<'_>) -> Result<R, Spent>,
    ) -> Result<R, CalcResult> {
        if self.exhausted {
            return Err(too_much_regex_work(cell));
        }
        let index = usize::from(case_insensitive);
        if !self.entries[index].contains_key(pattern) {
            let engine = self.compile(pattern, case_insensitive, cell)?;
            self.entries[index].insert(pattern.to_string(), Compiled { engine, held: 0 });
        }
        let Some(entry) = self.entries[index].get_mut(pattern) else {
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
        case_insensitive: bool,
        cell: CellReferenceIndex,
    ) -> Result<Option<(Engine, Caches)>, CalcResult> {
        let Ok(pattern) = unicode_posix_classes(pattern, case_insensitive) else {
            return Ok(None);
        };
        for limit in SIZE_LIMITS {
            if self.budget < limit {
                self.exhausted = true;
                return Err(too_much_regex_work(cell));
            }
            self.budget -= limit;
            match Engine::new(&pattern, limit, case_insensitive) {
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

/// The longest REGEXREPLACE result, in UTF-16 units. Unlike SUBSTITUTE's, it is not the cell
/// text limit: 65,535 units come back (and a cell holds them), 65,536 are `#CALC!`, emoji
/// counting two (text.md "REGEXREPLACE results are capped at 65,535 UTF-16 units";
/// probes/regexreplace-result-cap).
const MAX_REPLACED_UNITS: usize = 65_535;

/// The most bytes a REGEXREPLACE result may have: a UTF-16 unit takes at most 3 bytes, so a
/// longer result is over [`MAX_REPLACED_UNITS`] whatever its characters.
const MAX_REPLACED_BYTES: usize = 3 * MAX_REPLACED_UNITS;

/// A REGEXREPLACE result over [`MAX_REPLACED_UNITS`].
fn over_cap(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::CALC,
        cell,
        "REGEXREPLACE result is too long".to_string(),
    )
}

/// A lifted REGEXREPLACE whose results pass the text broadcast cap is one `#VALUE!` (text.md,
/// first entry).
fn too_long(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "REGEXREPLACE results are too long".to_string(),
    )
}

/// Which matches REGEXREPLACE replaces: its `occurrence` truncated toward zero (text.md
/// "REGEX* optional arguments").
#[derive(Clone, Copy)]
enum Occurrence {
    /// 0, omitted or blank: every match.
    All,
    /// n > 0: the n-th match, here counted from 0.
    Nth(usize),
    /// n < 0: the |n|-th match from the end, here counted from 0.
    NthLast(usize),
}

impl Occurrence {
    fn new(n: f64) -> Self {
        // The casts saturate: a count past `usize` is past every match, which leaves the text
        // unchanged as Excel does for 1E+300 and -1E+300 (probes/regex-optional-args A141, A142).
        let n = n.trunc();
        if n > 0.0 {
            Occurrence::Nth((n as usize).saturating_sub(1))
        } else if n < 0.0 {
            Occurrence::NthLast(((-n) as usize).saturating_sub(1))
        } else {
            Occurrence::All
        }
    }

    /// The match of `text` this occurrence names; with [`Occurrence::All`] the first. Matches
    /// are counted as REGEXREPLACE replaces them, empty ones included (`REGEXREPLACE("abc","",
    /// "-",4)` is `abc-`; probes/regex-optional-args A125-A128).
    fn find(self, search: &mut Search<'_>, text: &str) -> Result<Option<Match>, Spent> {
        let index = match self {
            Occurrence::All => return search.find(text),
            Occurrence::Nth(index) => index,
            Occurrence::NthLast(back) => {
                let mut count = 0usize;
                search.for_each_match(text, |_| count += 1)?;
                match count.checked_sub(back).and_then(|n| n.checked_sub(1)) {
                    Some(index) => index,
                    None => return Ok(None),
                }
            }
        };
        let mut seen = 0usize;
        let mut found = None;
        search.for_each_match(text, |m| {
            if seen == index {
                found = Some(m);
            }
            seen += 1;
        })?;
        Ok(found)
    }
}

/// What REGEXEXTRACT returns, its `return_mode`.
#[derive(Clone, Copy, PartialEq)]
enum ReturnMode {
    /// 0: the first match, whole even when the pattern has groups.
    First,
    /// 1: every match, whole, in one row.
    All,
    /// 2: the groups of the first match, in one row.
    Groups,
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

    /// A REGEX* option (`case_sensitivity`, `return_mode`): absent or blank is 0; a number,
    /// boolean or number text is truncated toward zero and must then be 0 to `max`; anything else,
    /// "" included, is `#VALUE!` (probes/regex-optional-args A1-A17, A61-A86, A146).
    fn regex_option(
        &self,
        value: Option<&CalcResult>,
        max: u8,
        cell: CellReferenceIndex,
    ) -> Result<u8, CalcResult> {
        let n = self.cast_number_or(value, 0.0, cell)?.trunc();
        (0..=max)
            .find(|&option| f64::from(option) == n)
            .ok_or_else(|| CalcResult::new_error(Error::VALUE, cell, "Invalid option".to_string()))
    }

    /// Whether a `case_sensitivity` argument asks to ignore case (1; 0 or absent is
    /// case-sensitive).
    fn regex_case_insensitive(
        &self,
        value: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        Ok(self.regex_option(value, 1, cell)? == 1)
    }

    /// REGEXEXTRACT(text, `regular_expression`, [`return_mode`], [`case_sensitivity`])
    ///
    /// `return_mode` 0 (the default) returns the first match, 1 every match in one row, 2 the
    /// capture groups of the first match in one row, a group that took no part in the match
    /// being `#N/A`; 2 is `#VALUE!` for a pattern without groups, matched or not
    /// (probes/regex-optional-args A29-A60, A138). No match is `#N/A`, an invalid pattern
    /// `#VALUE!`. Every argument lifts over a range or an array, a lifted element keeping the
    /// first value of its row (probes/m7-argument-modes F20, F22; textsplit-lift-guesses A53).
    pub(crate) fn fn_regexextract(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let (operands, _) = self.lift_text_operands(args, &[], cell);
        let mut cache = PatternCache::new();
        let result = self.lift_text_call(
            operands,
            cell,
            &[],
            COMPARED_BYTES_PER_VISIT,
            |model, values, first_only| {
                let options = Model::regex_operands(values, cell).and_then(|operands| {
                    let mode = match model.regex_option(values.get(2), 2, cell)? {
                        0 => ReturnMode::First,
                        1 => ReturnMode::All,
                        _ => ReturnMode::Groups,
                    };
                    let insensitive = model.regex_case_insensitive(values.get(3), cell)?;
                    Ok((operands, mode, insensitive))
                });
                let ((text, pattern), mode, insensitive) = match options {
                    // A lifted element keeps the first match only (T-1272 performance review).
                    Ok((operands, ReturnMode::All, insensitive)) if first_only => {
                        (operands, ReturnMode::First, insensitive)
                    }
                    Ok(options) => options,
                    Err(error) => return error,
                };
                let no_groups =
                    || CalcResult::new_error(Error::VALUE, cell, "No capture groups".to_string());
                let found = cache.with(&pattern, insensitive, cell, |search| match mode {
                    ReturnMode::First => Ok(Ok(search.find(&text)?.map(|m| vec![Some(m.range())]))),
                    ReturnMode::All => {
                        let mut all = Vec::new();
                        search.for_each_match(&text, |m| all.push(Some(m.range())))?;
                        Ok(Ok(Some(all).filter(|all| !all.is_empty())))
                    }
                    ReturnMode::Groups => {
                        if search.engine.group_len() < 2 {
                            return Ok(Err(no_groups()));
                        }
                        let mut caps = search.engine.create_captures();
                        if !search.captures(&text, &mut caps)? {
                            return Ok(Ok(None));
                        }
                        let groups = if first_only { 2 } else { caps.group_len() };
                        Ok(Ok(Some(
                            (1..groups)
                                .map(|group| caps.get_group(group).map(|m| m.range()))
                                .collect(),
                        )))
                    }
                });
                let ranges = match found {
                    Err(error) | Ok(Err(error)) => return error,
                    Ok(Ok(None)) => {
                        return CalcResult::new_error(Error::NA, cell, "No match found".to_string())
                    }
                    Ok(Ok(Some(ranges))) => ranges,
                };
                let piece = |range: Option<std::ops::Range<usize>>| match range {
                    Some(range) => {
                        ArrayNode::String(Text::from(text.get(range).unwrap_or_default()))
                    }
                    // A group that took no part in the match (probes/regex-optional-args A45,
                    // A139).
                    None => ArrayNode::Error(Error::NA),
                };
                let mut row: Vec<ArrayNode> = ranges.into_iter().map(piece).collect();
                if mode == ReturnMode::First || first_only {
                    return match row.swap_remove(0) {
                        ArrayNode::String(text) => CalcResult::String(text),
                        _ => {
                            CalcResult::new_error(Error::NA, cell, "Group not matched".to_string())
                        }
                    };
                }
                CalcResult::Array(vec![row])
            },
        );
        cache.finish(result, cell)
    }

    /// REGEXREPLACE(text, `regular_expression`, replacement, [occurrence], [`case_sensitivity`])
    ///
    /// Replaces the matches of `regular_expression` in `text` with `replacement`: every match
    /// when `occurrence` is 0 (the default), the n-th when it is n > 0, the n-th from the end when
    /// it is -n; a match beyond the count leaves `text` unchanged (probes/t1626-regexreplace
    /// C1-C13, regex-optional-args A104-A148). Returns #VALUE! if the regex is invalid. All the
    /// arguments lift over a range or an array (probes/m7-argument-modes F21).
    pub(crate) fn fn_regexreplace(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 3 || args.len() > 5 {
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
            |model, values, _| {
                let options = Model::regex_operands(values, cell).and_then(|operands| {
                    let replacement = Model::cast_string_ref(&values[2], cell)?;
                    let occurrence = model.cast_number_or(values.get(3), 0.0, cell)?;
                    let insensitive = model.regex_case_insensitive(values.get(4), cell)?;
                    Ok((
                        operands,
                        replacement,
                        Occurrence::new(occurrence),
                        insensitive,
                    ))
                });
                let ((text, pattern), replacement, occurrence, insensitive) = match options {
                    Ok(options) => options,
                    Err(error) => return error,
                };
                // Two passes over the text: one measures the result, one builds it.
                let replaced = cache.with(&pattern, insensitive, cell, |search| {
                    let Some(replacement) = Replacement::new(search.engine, &replacement) else {
                        // An invalid replacement is #VALUE! only once a match it replaces is
                        // found: else the text comes back unchanged (regex-pcre2-features E1-E3,
                        // regex-optional-args A133, A134).
                        if occurrence.find(search, &text)?.is_none() {
                            return Ok(Ok(text.to_string()));
                        }
                        return Ok(Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Invalid replacement".to_string(),
                        )));
                    };
                    let one = match occurrence {
                        Occurrence::All => None,
                        _ => {
                            let Some(m) = occurrence.find(search, &text)? else {
                                return Ok(Ok(text.to_string()));
                            };
                            let mut caps = search.engine.create_captures();
                            search.captures_at(&text, m, &mut caps)?;
                            Some((m, caps))
                        }
                    };
                    let len = match &one {
                        None => replacement.result_len(search, &text)?,
                        Some((m, caps)) => {
                            (text.len() - m.len()).saturating_add(replacement.expanded_len(caps))
                        }
                    };
                    if len > MAX_REPLACED_BYTES {
                        return Ok(Err(over_cap(cell)));
                    }
                    if !within_text_budget(&mut total, len) {
                        over_budget = true;
                        return Ok(Err(too_long(cell)));
                    }
                    Ok(Ok(match one {
                        None => replacement.replace(search, &text, len)?,
                        Some((m, caps)) => replacement.replace_one(&text, m, &caps, len),
                    }))
                });
                let replaced = match replaced {
                    Ok(Ok(replaced)) => Text::from(replaced.as_str()),
                    Ok(Err(error)) | Err(error) => return error,
                };
                if replaced.len_utf16() > MAX_REPLACED_UNITS {
                    return over_cap(cell);
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

    /// REGEXTEST(text, `regular_expression`, [`case_sensitivity`])
    ///
    /// Returns `TRUE` if `regular_expression` matches anywhere in `text`, `FALSE`
    /// otherwise; case is ignored when `case_sensitivity` is 1 (probes/regex-optional-args
    /// A1-A28). Returns #VALUE! if the regex is invalid. All the arguments lift over a range or
    /// an array (probes/m7-argument-modes F18, F19).
    pub(crate) fn fn_regextest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let (operands, _) = self.lift_text_operands(args, &[], cell);
        let mut cache = PatternCache::new();
        let result = self.lift_text_call(
            operands,
            cell,
            &[],
            COMPARED_BYTES_PER_VISIT,
            |model, values, _| {
                let options = Model::regex_operands(values, cell).and_then(|operands| {
                    Ok((operands, model.regex_case_insensitive(values.get(2), cell)?))
                });
                let ((text, pattern), insensitive) = match options {
                    Ok(options) => options,
                    Err(error) => return error,
                };
                match cache.with(&pattern, insensitive, cell, |search| search.is_match(&text)) {
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
                cache.with("x", false, CELL, |search| search.is_match(&text)),
                Ok(false)
            ));
        }
        let mut cache = PatternCache::new();
        cache.work = 1_000;
        assert!(is_error(
            &cache.with("x", false, CELL, |search| search.is_match(&text))
        ));
        assert!(is_error(
            &cache.with("x", false, CELL, |search| search.is_match("x"))
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
            .with(pattern, false, CELL, |search| search.is_match(&text))
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
            cache.with("(", false, CELL, |search| search.is_match("")),
            Err(CalcResult::Error {
                error: Error::VALUE,
                ..
            })
        ));
        let small = COMPILE_BUDGET / (1 << 16);
        let mut spent = false;
        for i in 0..=small {
            spent |=
                is_error(&cache.with(&format!("a{i}"), false, CELL, |search| search.is_match("")));
        }
        assert!(spent);
    }
}
