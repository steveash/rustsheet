use std::collections::HashMap;

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    fn collect_mode_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        for arg in args {
            // A union evaluates to `#VALUE!` here, returned below (probes/multiarea-foreach
            // H17, H18); evaluating through `evaluate_reference_arg` would turn a single-cell
            // reference into a range and lose the blank-cell `#VALUE!` (T-0947).
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Range { left, right } => {
                    let v = self.values_from_range(left, right)?;
                    values.extend(v.into_iter().flatten());
                }
                CalcResult::Array(arr) => match self.values_from_array(arr) {
                    Ok(v) => values.extend(v.into_iter().flatten()),
                    Err(e) => {
                        return Err(CalcResult::new_error(Error::VALUE, cell, format!("{e:?}")))
                    }
                },
                CalcResult::Number(n) => values.push(n),
                // A union's `#VALUE!` (H17, H18) and the error of a computed argument
                // (`MODE(CHOOSE(1,B1:B6*1))` off the range's rows is `#VALUE!`, probes/legacy-choose-arm-aggregate-and-array-constant-at
                // I10, T-1643) propagate; the error of a single-cell reference keeps being skipped
                // (unprobed).
                error @ CalcResult::Error { .. }
                    if !matches!(arg.ungrouped(), Node::ReferenceKind { .. }) =>
                {
                    return Err(error)
                }
                // Excel: a text value or a blank single-cell reference is #VALUE!, while
                // text/blanks inside ranges are skipped (coercion.md, MODE; T-0947). A
                // single-cell reference to text is unprobed, so it keeps being skipped.
                CalcResult::String(_) if !matches!(arg.ungrouped(), Node::ReferenceKind { .. }) => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "MODE: text argument".to_string(),
                    ));
                }
                CalcResult::EmptyCell if matches!(arg.ungrouped(), Node::ReferenceKind { .. }) => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "MODE: blank cell reference".to_string(),
                    ));
                }
                _ => {}
            }
        }
        Ok(values)
    }

    // MODE.SNGL(number1, [number2], ...)
    // Returns the most frequently occurring value. Returns #N/A if no repeated value.
    pub(crate) fn fn_mode_sngl(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let values = match self.collect_mode_values(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        if values.is_empty() {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "MODE.SNGL: no numeric values".to_string(),
            );
        }

        let (mode, max_count) = find_modes(&values);

        if max_count < 2 {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "MODE.SNGL: no value appears more than once".to_string(),
            );
        }

        CalcResult::Number(mode[0])
    }

    // MODE.MULT(number1, [number2], ...)
    // Returns all modes as a vertical spill array. Returns #N/A if no repeated value.
    pub(crate) fn fn_mode_mult(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let values = match self.collect_mode_values(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        if values.is_empty() {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "MODE.MULT: no numeric values".to_string(),
            );
        }

        let (modes, max_count) = find_modes(&values);

        if max_count < 2 {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "MODE.MULT: no value appears more than once".to_string(),
            );
        }

        let result: Vec<Vec<ArrayNode>> =
            modes.iter().map(|&v| vec![ArrayNode::Number(v)]).collect();
        CalcResult::Array(result)
    }
}

/// Returns (`modes_in_first_encounter_order`, `max_frequency`).
/// When multiple values tie for the highest frequency, they are returned in the
/// order they were first seen in the input — matching Excel's behaviour.
fn find_modes(values: &[f64]) -> (Vec<f64>, usize) {
    // counts: bits → (value, count, first_seen_index)
    let mut counts: HashMap<u64, (f64, usize, usize)> = HashMap::new();
    for (pos, &v) in values.iter().enumerate() {
        let key = v.to_bits();
        let entry = counts.entry(key).or_insert((v, 0, pos));
        entry.1 += 1;
    }

    let max_count = counts.values().map(|(_, c, _)| *c).max().unwrap_or(0);

    let mut modes: Vec<(f64, usize)> = counts
        .values()
        .filter(|(_, c, _)| *c == max_count)
        .map(|(v, _, first)| (*v, *first))
        .collect();

    modes.sort_by_key(|&(_, first)| first);
    (modes.into_iter().map(|(v, _)| v).collect(), max_count)
}
