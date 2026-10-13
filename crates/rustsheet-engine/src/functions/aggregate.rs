//! `AGGREGATE(function_num`, options, ref1, [ref2], ...) and
//! `AGGREGATE(function_num`, options, array, k).
//!
//! AGGREGATE gathers the numbers its options keep and hands them, as an array, to the function
//! it names (SUM, LARGE, PERCENTILE.INC, ...) as number arguments or an array, so every
//! statistic keeps its one definition.
//! Excel's rules are in docs/excel-semantics/numbers.md "AGGREGATE" (probes/aggregate-grid,
//! probes/aggregate-args, probes/aggregate-overflow).

use std::collections::HashSet;

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    reference_operators::RefArg,
};

/// What an option (0-7) leaves out.
#[derive(Clone, Copy)]
struct Ignore {
    /// Cells whose formula is a SUBTOTAL or AGGREGATE (options 0-3).
    nested: bool,
    /// Hidden rows (options 1, 3, 5, 7).
    hidden: bool,
    /// Error values (options 2, 3, 6, 7).
    errors: bool,
}

/// The values AGGREGATE reads from its references or array.
#[derive(Default)]
struct Collected {
    numbers: Vec<f64>,
    /// Non-empty values, for COUNTA (function 3).
    non_empty: usize,
    /// The first error not ignored. COUNT and COUNTA never return it.
    error: Option<CalcResult>,
    /// Sheets whose hidden rows are already in `hidden_rows` (odd options only).
    hidden_sheets: HashSet<u32>,
    hidden_rows: HashSet<(u32, i32)>,
}

impl Model {
    pub(crate) fn fn_aggregate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        // An array function_num, options or (in a four-argument call) fourth argument lifts:
        // each element is its own AGGREGATE, broadcast like the *IF(S) criteria, so
        // `AGGREGATE({9,4},0,E1:E6)` spills SUM and MAX and an array k spills one LARGE per k,
        // even in the reference form, where every element is `#VALUE!` (probes/aggregate-args
        // Misc!A36:B38, probes/aggregate-array-args). A range fourth argument is a reference
        // and is not lifted. A multi-cell range function_num or options
        // lifts like an array, a single cell is a scalar (probes/aggregate-range-args).
        let function = self.evaluate_node_in_context(&args[0], cell);
        let option = self.evaluate_node_in_context(&args[1], cell);
        let fourth = (args.len() == 4).then(|| self.evaluate_node_with_reference(&args[3], cell));
        let lifts = |value: &CalcResult| matches!(value, CalcResult::Array(_));
        let lifts_head = |value: &CalcResult| match value {
            CalcResult::Range { left, right } => is_multi_cell(left, right),
            other => lifts(other),
        };
        if !lifts_head(&function) && !lifts_head(&option) && !fourth.as_ref().is_some_and(lifts) {
            // A scalar fourth argument is passed on evaluated, so it is not computed twice;
            // a reference stays a node.
            return match fourth.as_ref().and_then(value_node) {
                Some(node) => {
                    let rest = [args[2].clone(), node];
                    self.aggregate_scalar(function, option, &rest, cell)
                }
                None => self.aggregate_scalar(function, option, &args[2..], cell),
            };
        }
        let lifts_fourth = fourth.as_ref().is_some_and(lifts);
        // The value arguments are evaluated once, not once per element: a computed array
        // becomes a literal. Every element reads all of them, which the visit cap counts
        // (directive 5; criteria.md "*IF(S) lift array criteria", DEVIATION).
        let value_count = if lifts_fourth { 1 } else { args.len() - 2 };
        let mut rest: Vec<Node> = Vec::with_capacity(args.len() - 2);
        let mut cost: u64 = 0;
        for node in &args[2..2 + value_count] {
            match self.evaluate_node_with_reference(node, cell) {
                CalcResult::Array(rows) => {
                    let size = rows.len().saturating_mul(rows.first().map_or(0, Vec::len));
                    cost = cost.saturating_add(u64::try_from(size).unwrap_or(u64::MAX));
                    rest.push(Node::ArrayKind(rows));
                }
                _ => {
                    cost = cost.saturating_add(self.walk_cost(node, cell));
                    rest.push(node.clone());
                }
            }
        }
        let mut lifted = vec![function, option];
        lifted.extend(fourth.filter(lifts));
        // A single-cell range stays a scalar reference; `lift_with_cost` reads multi-cell ones.
        let ranges: Vec<Option<CalcResult>> = lifted
            .iter_mut()
            .map(|value| match value {
                CalcResult::Range { left, right } if !is_multi_cell(left, right) => {
                    Some(std::mem::replace(value, CalcResult::EmptyCell))
                }
                _ => None,
            })
            .collect();
        self.lift_with_cost(
            lifted,
            |_| cost.max(1),
            cell,
            |model, values| {
                let scalar = |index: usize| {
                    ranges[index]
                        .clone()
                        .unwrap_or_else(|| values[index].clone())
                };
                if lifts_fourth {
                    let k = value_node(&values[2]).unwrap_or(Node::EmptyArgKind);
                    let element_rest = [rest[0].clone(), k];
                    model.aggregate_scalar(scalar(0), scalar(1), &element_rest, cell)
                } else {
                    model.aggregate_scalar(scalar(0), scalar(1), &rest, cell)
                }
            },
        )
    }

    /// One AGGREGATE call with `function_num` and options already evaluated.
    fn aggregate_scalar(
        &mut self,
        function: CalcResult,
        option: CalcResult,
        rest: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // Both are truncated: AGGREGATE(9.9,7.9,..) is AGGREGATE(9,7,..) (probes/aggregate-args).
        let function = match self.cast_to_number(function, cell) {
            Ok(f) => f.trunc(),
            Err(error) => return error,
        };
        let option = match self.cast_to_number(option, cell) {
            Ok(f) => f.trunc(),
            Err(error) => return error,
        };
        if !(1.0..=19.0).contains(&function) || !(0.0..=7.0).contains(&option) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid function_num or options".to_string(),
            );
        }
        let (function, option) = (function as u8, option as u8);
        let ignore = Ignore {
            nested: option <= 3,
            hidden: option % 2 == 1,
            errors: matches!(option, 2 | 3 | 6 | 7),
        };
        if function >= 14 {
            self.aggregate_array_form(function, ignore, rest, cell)
        } else {
            self.aggregate_reference_form(function, ignore, rest, cell)
        }
    }

    /// A multi-cell range k reads its top-left cell (probes/aggregate-range-args A31, A61-A71).
    /// Any other k, or a top-left cell that is not a plain value, is passed on unchanged.
    fn top_left_k(&mut self, k: &Node, cell: CellReferenceIndex) -> Node {
        if !matches!(k.ungrouped(), Node::RangeKind { .. }) {
            return k.clone();
        }
        match self.evaluate_node_with_reference(k, cell) {
            CalcResult::Range { left, right }
                if left.sheet == right.sheet && is_multi_cell(&left, &right) =>
            {
                value_node(&self.evaluate_cell(left)).unwrap_or_else(|| k.clone())
            }
            _ => k.clone(),
        }
    }

    /// Functions 1-13: every argument must be a reference. A value, an array or a nested
    /// SUBTOTAL call is `#VALUE!`, but an error argument is returned whatever the options.
    fn aggregate_reference_form(
        &mut self,
        function: u8,
        ignore: Ignore,
        refs: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut collected = Collected::default();
        for node in refs {
            let areas = match self.reference_areas(node, cell) {
                Ok(areas) => areas,
                Err(error) => return error,
            };
            // Several areas are walked as a union's, on the formula's walk budget, as SUM's are.
            let of_union = areas.len() > 1;
            for area in areas {
                if let Err(error) =
                    self.aggregate_collect_area(&area, of_union, ignore, &mut collected, cell)
                {
                    return error;
                }
            }
        }
        match function {
            2 => return CalcResult::Number(collected.numbers.len() as f64),
            3 => return CalcResult::Number(collected.non_empty as f64),
            _ => {}
        }
        if let Some(error) = collected.error {
            return error;
        }
        if collected.numbers.is_empty() {
            // probes/aggregate-grid F21:F33 (an empty range).
            return match function {
                4 | 5 | 6 | 9 => CalcResult::Number(0.0),
                12 => CalcResult::new_error(Error::NUM, cell, "No numbers".to_string()),
                13 => CalcResult::new_error(Error::NA, cell, "No numbers".to_string()),
                _ => CalcResult::new_error(Error::DIV, cell, "No numbers".to_string()),
            };
        }
        // Plain number arguments: PRODUCT does not read array literals.
        let values: Vec<Node> = collected
            .numbers
            .into_iter()
            .map(Node::NumberKind)
            .collect();
        match function {
            1 => self.fn_average(&values, cell),
            4 => self.fn_max(&values, cell),
            5 => self.fn_min(&values, cell),
            6 => self.fn_product(&values, cell),
            7 => self.fn_stdev_s(&values, cell),
            8 => self.fn_stdev_p(&values, cell),
            9 => self.fn_sum(&values, cell),
            10 => self.fn_var_s(&values, cell),
            11 => self.fn_var_p(&values, cell),
            12 => self.fn_median(&values, cell),
            _ => self.fn_mode_sngl(&values, cell),
        }
    }

    /// Functions 14-19 take exactly an array (a reference, a union or a computed array) and k.
    fn aggregate_array_form(
        &mut self,
        function: u8,
        ignore: Ignore,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let [array, k] = args else {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected an array and k".to_string(),
            );
        };
        let mut collected = Collected::default();
        // A union reaches it behind a name, a LET variable, IF or CHOOSE too (SPEC-0004).
        let result = match self.evaluate_reference_arg(array, cell) {
            RefArg::Ranges(ranges) => Ok(ranges.to_vec()),
            RefArg::Value(value) => match value {
                CalcResult::Array(rows) => {
                    for value in rows.iter().flatten() {
                        collect_value(value_of(value, cell), ignore, &mut collected);
                    }
                    Ok(Vec::new())
                }
                CalcResult::Number(number) => {
                    collected.numbers.push(number);
                    Ok(Vec::new())
                }
                error @ CalcResult::Error { .. } => Err(error),
                _ => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expected an array".to_string(),
                )),
            },
        };
        let areas = match result {
            Ok(areas) => areas,
            Err(error) => return error,
        };
        let of_union = areas.len() > 1;
        for area in areas {
            if let Err(error) =
                self.aggregate_collect_area(&area, of_union, ignore, &mut collected, cell)
            {
                return error;
            }
        }
        if let Some(error) = collected.error {
            return error;
        }
        if collected.numbers.is_empty() {
            // probes/aggregate-grid I41:I46 (an empty range).
            return CalcResult::new_error(Error::NUM, cell, "No numbers".to_string());
        }
        let k = self.top_left_k(k, cell);
        let values = [numbers_array(collected.numbers), k];
        match function {
            14 => self.fn_large(&values, cell),
            15 => self.fn_small(&values, cell),
            16 => self.fn_percentile_inc(&values, cell),
            17 => self.fn_quartile_inc(&values, cell),
            18 => self.fn_percentile_exc(&values, cell),
            _ => self.fn_quartile_exc(&values, cell),
        }
    }

    /// Adds the cells of `area` that `ignore` keeps to `collected`. Hidden rows (and rows a
    /// table filter hides) and nested SUBTOTAL/AGGREGATE cells are skipped unevaluated.
    fn aggregate_collect_area(
        &mut self,
        area: &Range,
        of_union: bool,
        ignore: Ignore,
        collected: &mut Collected,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        let nested = [Function::Subtotal, Function::Aggregate];
        // The hidden rows of each sheet, read once per AGGREGATE call: a per-cell (or per-area)
        // lookup would rescan the sheet's row styles (cells or areas x styled rows).
        if ignore.hidden {
            for sheet in area.left.sheet..=area.right.sheet {
                if !collected.hidden_sheets.insert(sheet) {
                    continue;
                }
                let worksheet = self
                    .workbook
                    .worksheet(sheet)
                    .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
                collected.hidden_rows.extend(
                    worksheet
                        .rows
                        .iter()
                        .filter(|row| row.hidden)
                        .map(|row| (sheet, row.r)),
                );
            }
        }
        let mut f = |model: &mut Self, position: CellReferenceIndex| {
            if collected
                .hidden_rows
                .contains(&(position.sheet, position.row))
            {
                return Ok(());
            }
            if ignore.nested
                && model.cell_formula_is_one_of(
                    position.sheet,
                    position.row,
                    position.column,
                    &nested,
                )
            {
                return Ok(());
            }
            let value = model.evaluate_cell(position);
            collect_value(value, ignore, collected);
            Ok(())
        };
        self.for_each_position_in_area(area, of_union, cell, &mut f)
    }
}

/// Text and booleans count only for COUNTA; blanks never count.
fn collect_value(value: CalcResult, ignore: Ignore, collected: &mut Collected) {
    match value {
        CalcResult::Number(number) => {
            collected.numbers.push(number);
            collected.non_empty += 1;
        }
        CalcResult::Error { .. } => {
            if !ignore.errors {
                collected.non_empty += 1;
                if collected.error.is_none() {
                    collected.error = Some(value);
                }
            }
        }
        CalcResult::EmptyCell | CalcResult::EmptyArg => {}
        _ => collected.non_empty += 1,
    }
}

/// An element of a computed array as a value.
fn value_of(value: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match value {
        ArrayNode::Number(number) => CalcResult::Number(*number),
        ArrayNode::Boolean(boolean) => CalcResult::Boolean(*boolean),
        ArrayNode::String(text) => CalcResult::String(text.clone()),
        ArrayNode::Error(error) => CalcResult::new_error(error.clone(), cell, String::new()),
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

/// `numbers` as a one-row array literal, the form the statistics functions read.
fn numbers_array(numbers: Vec<f64>) -> Node {
    Node::ArrayKind(vec![numbers.into_iter().map(ArrayNode::Number).collect()])
}

/// Whether a range spans more than one cell.
fn is_multi_cell(left: &CellReferenceIndex, right: &CellReferenceIndex) -> bool {
    left.row != right.row || left.column != right.column
}

/// An evaluated scalar as a literal argument node, so the scalar path reads it like a typed
/// value; `None` for a reference or an array.
fn value_node(value: &CalcResult) -> Option<Node> {
    Some(match value {
        CalcResult::Number(number) => Node::NumberKind(*number),
        CalcResult::Boolean(boolean) => Node::BooleanKind(*boolean),
        // `StringKind` holds the formula spelling, with `"` doubled.
        CalcResult::String(text) => Node::StringKind(text.replace('"', "\"\"")),
        CalcResult::Error { error, .. } => Node::ErrorKind(error.clone()),
        CalcResult::EmptyCell | CalcResult::EmptyArg => Node::EmptyArgKind,
        _ => return None,
    })
}
