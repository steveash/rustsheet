#[cfg(not(target_arch = "wasm32"))]
use regex::Regex;
#[cfg(target_arch = "wasm32")]
use regex_lite::Regex;

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// The last pattern compiled and its regex (`None` when invalid): a lifted call over a range of
/// text with one pattern compiles it once.
#[derive(Default)]
struct PatternCache(Option<(String, Option<Regex>)>);

impl PatternCache {
    /// The regex for `pattern`, or the `#VALUE!` of an invalid one.
    fn get(&mut self, pattern: String, cell: CellReferenceIndex) -> Result<&Regex, CalcResult> {
        if self.0.as_ref().is_none_or(|(last, _)| *last != pattern) {
            let regex = Regex::new(&pattern).ok();
            self.0 = Some((pattern, regex));
        }
        self.0
            .as_ref()
            .and_then(|(_, regex)| regex.as_ref())
            .ok_or_else(|| {
                CalcResult::new_error(Error::VALUE, cell, "Invalid regular expression".to_string())
            })
    }
}

impl Model {
    /// The text and pattern of one REGEX* call element, coerced to text left to right.
    fn regex_operands(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(String, String), CalcResult> {
        let text = self.cast_to_string(values[0].clone(), cell)?;
        let pattern = self.cast_to_string(values[1].clone(), cell)?;
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
        let mut cache = PatternCache::default();
        self.lift_text_call(operands, cell, 1, true, |model, values| {
            let (text, pattern) = match model.regex_operands(values, cell) {
                Ok(operands) => operands,
                Err(error) => return error,
            };
            let return_all = if has_return_all {
                match model.cast_to_number(values[2].clone(), cell) {
                    Ok(n) => n != 0.0,
                    Err(e) => return e,
                }
            } else {
                false
            };
            let re = match cache.get(pattern, cell) {
                Ok(re) => re,
                Err(error) => return error,
            };
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
        })
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
        let mut cache = PatternCache::default();
        self.lift_text_call(operands, cell, 1, true, |model, values| {
            let (text, pattern) = match model.regex_operands(values, cell) {
                Ok(operands) => operands,
                Err(error) => return error,
            };
            let replacement = match model.cast_to_string(values[2].clone(), cell) {
                Ok(s) => s,
                Err(e) => return e,
            };
            match cache.get(pattern, cell) {
                Ok(re) => CalcResult::String(
                    re.replace_all(&text, replacement.as_str())
                        .into_owned()
                        .into(),
                ),
                Err(error) => error,
            }
        })
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
        let mut cache = PatternCache::default();
        self.lift_text_call(operands, cell, 1, true, |model, values| {
            let (text, pattern) = match model.regex_operands(values, cell) {
                Ok(operands) => operands,
                Err(error) => return error,
            };
            match cache.get(pattern, cell) {
                Ok(re) => CalcResult::Boolean(re.is_match(&text)),
                Err(error) => error,
            }
        })
    }
}
