use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use regex::{Regex, RegexBuilder};
#[cfg(target_arch = "wasm32")]
use regex_lite::{Regex, RegexBuilder};

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// The size limits a pattern is compiled under, smallest first. A pattern is compiled under the
/// next one only when it is too big for the one before; the last is the regex crate's default,
/// so every pattern that compiled before T-1272 still compiles. Compile time and memory grow
/// with the limit: in a release build the largest patterns each admits take about 0.5 ms, 7 ms
/// and 70-90 ms (`\w{200}`), and a failed attempt about as long (T-1272 performance review).
const SIZE_LIMITS: [usize; 3] = [1 << 16, 1 << 20, 10 << 20];

/// Bytes of size limit one call may compile under, attempts that were too big included: about
/// 2 s of compiling, 4,096 small patterns, 256 medium or 25 of the largest. A call that needs
/// more is `#ERROR!` (DEVIATION, decision 0014: Excel has no such limit).
const COMPILE_BUDGET: usize = 256 << 20;

/// Bytes of size limit the cache keeps compiled at once, bounding its memory (CLAUDE.md
/// directive 5): 512 small patterns, 32 medium or 3 of the largest.
const CACHE_LIMIT: usize = 32 << 20;

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

/// A compiled pattern (`None` when invalid), the size limit it holds and when it was last used.
struct Compiled {
    regex: Option<Regex>,
    limit: usize,
    used: u64,
}

/// The patterns one REGEX* call has compiled, least recently used evicted first: a lifted call
/// compiles each of a few distinct patterns once, within [`COMPILE_BUDGET`] and [`CACHE_LIMIT`].
struct PatternCache {
    entries: HashMap<String, Compiled>,
    held: usize,
    budget: usize,
    clock: u64,
    exhausted: bool,
}

impl PatternCache {
    fn new() -> Self {
        PatternCache {
            entries: HashMap::new(),
            held: 0,
            budget: COMPILE_BUDGET,
            clock: 0,
            exhausted: false,
        }
    }

    /// The regex for `pattern`, or the `#VALUE!` of an invalid one, or `#ERROR!` once the
    /// call's compile budget is spent.
    fn get(&mut self, pattern: &str, cell: CellReferenceIndex) -> Result<&Regex, CalcResult> {
        self.clock += 1;
        if !self.entries.contains_key(pattern) {
            let compiled = self.compile(pattern, cell)?;
            self.make_room(compiled.limit);
            self.held += compiled.limit;
            self.entries.insert(pattern.to_string(), compiled);
        }
        let clock = self.clock;
        self.entries
            .get_mut(pattern)
            .and_then(|entry| {
                entry.used = clock;
                entry.regex.as_ref()
            })
            .ok_or_else(|| {
                CalcResult::new_error(Error::VALUE, cell, "Invalid regular expression".to_string())
            })
    }

    /// Compiles `pattern` under the smallest size limit it fits, charging each attempt.
    fn compile(&mut self, pattern: &str, cell: CellReferenceIndex) -> Result<Compiled, CalcResult> {
        let mut regex = None;
        let mut held = 0;
        for limit in SIZE_LIMITS {
            if self.exhausted || self.budget < limit {
                self.exhausted = true;
                return Err(too_many_compiles(cell));
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
        Ok(Compiled {
            regex,
            limit: held,
            used: 0,
        })
    }

    /// Evicts the least recently used patterns until one of size limit `limit` fits.
    fn make_room(&mut self, limit: usize) {
        while self.held + limit > CACHE_LIMIT {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.used)
                .map(|(pattern, _)| pattern.clone());
            match oldest.and_then(|pattern| self.entries.remove(&pattern)) {
                Some(entry) => self.held -= entry.limit,
                None => break,
            }
        }
    }

    /// The call's result: `#ERROR!` when the compile budget ran out in any element.
    fn finish(&self, result: CalcResult, cell: CellReferenceIndex) -> CalcResult {
        if self.exhausted {
            too_many_compiles(cell)
        } else {
            result
        }
    }
}

fn too_many_compiles(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::ERROR,
        cell,
        "Too many regular expressions to compile".to_string(),
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
        let result = self.lift_text_call(operands, cell, &[], |model, values, first_only| {
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
            let re = match cache.get(&pattern, cell) {
                Ok(re) => re,
                Err(error) => return error,
            };
            if return_all && first_only {
                // A lifted element keeps the first match only (T-1272 performance review).
                return match re.find(&text) {
                    Some(m) => CalcResult::String(m.as_str().to_string().into()),
                    None => CalcResult::new_error(Error::NA, cell, "No match found".to_string()),
                };
            }
            if return_all {
                let row: Vec<ArrayNode> = re
                    .find_iter(&text)
                    .map(|m| ArrayNode::String(m.as_str().to_string().into()))
                    .collect();
                if row.is_empty() {
                    return CalcResult::new_error(Error::NA, cell, "No match found".to_string());
                }
                CalcResult::Array(vec![row])
            } else {
                match re.captures(&text) {
                    None => CalcResult::new_error(Error::NA, cell, "No match found".to_string()),
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
        });
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
        let result = self.lift_text_call(operands, cell, &[], |_, values, _| {
            let (text, pattern) = match Model::regex_operands(values, cell) {
                Ok(operands) => operands,
                Err(error) => return error,
            };
            let replacement = match Model::cast_string_ref(&values[2], cell) {
                Ok(s) => s,
                Err(e) => return e,
            };
            match cache.get(&pattern, cell) {
                Ok(re) => CalcResult::String(
                    re.replace_all(&text, replacement.as_str())
                        .into_owned()
                        .into(),
                ),
                Err(error) => error,
            }
        });
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
        let result = self.lift_text_call(operands, cell, &[], |_, values, _| {
            let (text, pattern) = match Model::regex_operands(values, cell) {
                Ok(operands) => operands,
                Err(error) => return error,
            };
            match cache.get(&pattern, cell) {
                Ok(re) => CalcResult::Boolean(re.is_match(&text)),
                Err(error) => error,
            }
        });
        cache.finish(result, cell)
    }
}
