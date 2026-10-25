//! Reading and validating the VLOOKUP/HLOOKUP index argument.

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{
        lift::{LiftCost, LiftOptions, LiftedArg},
        util::array_node_to_calc_result,
        util::near_integer::near_integer_count,
        Function,
    },
    model::Model,
    reference_operators::{several_areas_ref, Ranges, RefArg},
};

/// The row/column index argument of VLOOKUP/HLOOKUP, read before the search but validated only
/// after a match (docs/excel-semantics/lookup.md "VLOOKUP/HLOOKUP search before validating the
/// index").
#[derive(Clone)]
pub(super) enum LookupIndexArg {
    /// A value argument, already coerced to a number (a failed coercion fails at once).
    Number(f64),
    /// A reference argument; its cell is read only once the lookup value is found.
    Reference(CalcResult),
    /// An array argument (`{1,2}`): with a scalar `lookup_value` and `range_lookup` it lifts, once
    /// the value is found; otherwise only its first element is used. Elements validate like a
    /// referenced cell (probes/vlookup-array-index-range-lookup, T-1269).
    Array(Vec<Vec<ArrayNode>>),
    /// A multi-cell range argument, kept unread like a reference: it lifts like an array of its
    /// values once the value is found, so a value that is not found is `#N/A` however large the
    /// range is (probes/lookup-index-multicell-if, T-1426); a lifted lookup reads only its
    /// first cell.
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
}

impl LookupIndexArg {
    /// The index as one value: an array argument gives its first element, read like a referenced
    /// cell (probes/vlookup-array-index-range-lookup A1-A3, A137-A149).
    pub(super) fn first_element(&self, cell: CellReferenceIndex) -> LookupIndexArg {
        match self {
            LookupIndexArg::Array(grid) => grid
                .first()
                .and_then(|row| row.first())
                .map_or(LookupIndexArg::Number(f64::NAN), |node| {
                    LookupIndexArg::Reference(array_node_to_calc_result(node, cell))
                }),
            LookupIndexArg::Range { left, .. } => LookupIndexArg::Reference(CalcResult::Range {
                left: *left,
                right: *left,
            }),
            other => other.clone(),
        }
    }
}

impl Model {
    /// Reads `lookup_value`, the index and `range_lookup` of VLOOKUP/HLOOKUP in argument order,
    /// with `table_error` the `table_array`'s error, if it has one. An error in `lookup_value` or
    /// the index returns at once, unless `range_lookup` is a multi-cell range or an array: that
    /// still lifts, so each element is the error and the result spills (gen-m2/f7e6d351c8538ef5
    /// Calc!A296 and 942bdade381b7537 Calc!A381, T-1468). Arguments fail in the order
    /// `lookup_value`, `table_array`, index, `range_lookup`, so a table error beats the last two,
    /// and a lifted `lookup_value` gives each element the table error
    /// (probes/lookup-argument-edges H66-H123, T-2234). The `lookup_value`'s error beats
    /// `range_lookup`'s, lifted or not (H30-H51). A scalar `range_lookup` that is not a boolean
    /// beside a lifted `lookup_value` is kept as its error, so each element's scalar call gives it,
    /// before any search (probes/vlookup-scalar-range-lookup-lifted G1-G12, T-2411).
    pub(super) fn lookup_head(
        &mut self,
        args: &[Node],
        table_error: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<(CalcResult, LookupIndexArg, CalcResult), CalcResult> {
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        let index = self.lookup_index_arg(&args[2], cell);
        let is_sorted = self.lookup_sorted_arg(args.get(3), cell);
        let lifts = matches!(
            is_sorted,
            Ok(CalcResult::Range { .. } | CalcResult::Array(_))
        );
        if lookup_value.is_error() && !lifts {
            return Err(lookup_value);
        }
        if let Some(error) = table_error {
            let sorted_grid = is_sorted.ok().filter(LiftedArg::is_grid);
            return Err(self.argument_error_each(lookup_value, sorted_grid, error, cell));
        }
        let index = match index {
            Ok(index) => index,
            // A failed value index in a lifted call is that error in every element whose
            // lookup_value is not an error itself, ahead of range_lookup and of the search, a
            // not-found element included (probes/vlookup-scalar-range-lookup-lifted G13-G39, T-2411;
            // gen-m2/942bdade381b7537 Calc!A381, T-1468).
            Err(error) if lifts || LiftedArg::is_grid(&lookup_value) => {
                let sorted_grid = is_sorted.ok().filter(LiftedArg::is_grid);
                return Err(self.argument_error_each(lookup_value, sorted_grid, error, cell));
            }
            Err(error) => return Err(error),
        };
        let is_sorted = match is_sorted {
            Ok(is_sorted) => is_sorted,
            Err(error) if LiftedArg::is_grid(&lookup_value) => error,
            Err(error) => return Err(error),
        };
        Ok((lookup_value, index, is_sorted))
    }

    /// A failed `table_array` or value index under `lookup_value`: the error, or, lifted over an
    /// array or multi-cell range `lookup_value` or `range_lookup` (`sorted_grid`), that error in
    /// each element whose `lookup_value` is not an error itself: `VLOOKUP({2;9},1/0,2)` and
    /// `VLOOKUP(2,1/0,2,{TRUE;FALSE})` spill `#DIV/0!` twice, `VLOOKUP(NA(),1/0,2,{TRUE;FALSE})`
    /// `#N/A` twice (probes/lookup-argument-edges H120-H124, H336-H352), and
    /// `VLOOKUP({1;99},A1:B2,"a")` is `#VALUE!` twice (probes/vlookup-scalar-range-lookup-lifted).
    fn argument_error_each(
        &mut self,
        lookup_value: CalcResult,
        sorted_grid: Option<CalcResult>,
        error: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // The engine's own `#ERROR!` (a table past the array-size limit, arrays.md "Array results
        // above 1,048,576 cells are #ERROR!") stops the call: spread over a lifted lookup_value
        // it would build a grid as large as the one the limit refused.
        let is_limit = matches!(
            error,
            CalcResult::Error {
                error: Error::ERROR,
                ..
            }
        );
        let mut criteria = vec![lookup_value];
        criteria.extend(sorted_grid);
        if is_limit || !criteria.iter().any(LiftedArg::is_grid) {
            return error;
        }
        self.lift_values(
            criteria,
            LiftCost::Unit,
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |_, values| {
                if values[0].is_error() {
                    values[0].clone()
                } else {
                    error.clone()
                }
            },
        )
    }

    /// Reads the index argument of VLOOKUP/HLOOKUP. A value argument is coerced now, so literal
    /// text or an error fails before the search (probes/lookup-index-order H9, H13, H28); a
    /// reference is kept for [`Model::resolve_lookup_index`]. `INDEX(...)` is a reference too: its
    /// selected cell is read only after a match (probes/lookup-index-shapes H2, H3, H11, H12).
    pub(super) fn lookup_index_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LookupIndexArg, CalcResult> {
        // An `IF` index is the cell it picks, so the lookup pick stays a reference
        // (probes/lookup-pick-index-shapes A3, A9, B2; T-1377).
        let index = match node.ungrouped() {
            Node::FunctionKind {
                kind: Function::If, ..
            } => match self.evaluate_reference_arg(node, cell) {
                // A union picked by `IF` is `#REF!` like a non-numeric referenced cell, so only
                // after a match: a value not found is `#N/A` (probes/lookup-index-multicell-if
                // M1, lookup-index-multicell-lifted M1; T-1426).
                RefArg::Ranges(Ranges::Areas(_)) => {
                    return Ok(LookupIndexArg::Reference(several_areas_ref(cell)))
                }
                picked => picked.into_single_area(cell),
            },
            _ => self.eval_ref_or_index(node, cell),
        };
        match index {
            // A multi-cell range index lifts like an array of its values, never intersecting
            // (probes/lookup-index-multicell-if A1-D2, J1-L2, O1; T-1426).
            CalcResult::Range { left, right }
                if left.row != right.row || left.column != right.column =>
            {
                Ok(LookupIndexArg::Range { left, right })
            }
            range @ CalcResult::Range { .. } => Ok(LookupIndexArg::Reference(range)),
            CalcResult::Array(grid) => Ok(LookupIndexArg::Array(grid)),
            value => self.cast_to_number(value, cell).map(LookupIndexArg::Number),
        }
    }

    /// Reads `range_lookup` (default TRUE): a scalar is coerced to a boolean now, so a bad one
    /// fails before the search; an array stays as it is and lifts with the lookup value.
    pub(super) fn lookup_sorted_arg(
        &mut self,
        node: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let Some(node) = node else {
            return Ok(CalcResult::Boolean(true));
        };
        match self.evaluate_node_in_context(node, cell) {
            array @ CalcResult::Array(_) => Ok(array),
            // A multi-cell range lifts like an array (gen-m2 T-1468); a one-cell range is a scalar.
            range @ CalcResult::Range { left, right }
                if left.row != right.row || left.column != right.column =>
            {
                Ok(range)
            }
            value => self.cast_to_bool(value, cell).map(CalcResult::Boolean),
        }
    }

    /// Validates the index once the lookup value is found and returns it 0-based. A referenced
    /// cell that is not a number (text, an error) is `#REF!`; an index below 1 is `#VALUE!`, past
    /// `size` or at or below the 32-bit integer minimum is `#REF!` (probes/lookup-index-order, excel-graph).
    /// A fractional index is floored, snapping up within [`near_integer_count`]'s tolerance.
    pub(super) fn resolve_lookup_index(
        &mut self,
        index: LookupIndexArg,
        size: i32,
        cell: CellReferenceIndex,
    ) -> Result<i32, CalcResult> {
        let invalid_reference = || CalcResult::Error {
            error: Error::REF,
            origin: cell,
            message: "Invalid reference".to_string(),
        };
        let number = match index {
            LookupIndexArg::Number(number) => number,
            LookupIndexArg::Reference(range) => self
                .cast_to_number(range, cell)
                .map_err(|_| invalid_reference())?,
            // Callers narrow an array or range with `first_element`; one that does not gets `#REF!`.
            LookupIndexArg::Array(_) | LookupIndexArg::Range { .. } => {
                return Err(invalid_reference())
            }
        };
        // The index is floored, but a fraction within NEAR_INTEGER_TOLERANCE below an integer
        // counts as that integer, as in INDEX: `VLOOKUP(1,B1:CW1,2-2^-22)` reads column 2 and
        // 3.9999999 is past a 3-column table (probes/lookup-index-near-integer, T-2234).
        let number = near_integer_count(number);
        // -2147483648 is already #REF! while -2147483647 is #VALUE! (probes/lookup-index-shapes
        // H19-H20, H31-H32), tested after flooring: -2147483647.5 is #REF! and -2147483646.5
        // #VALUE! (probes/lookup-argument-edges H192, H228).
        if number <= f64::from(i32::MIN) {
            return Err(invalid_reference());
        }
        // In range now, so the cast cannot saturate on the low side; a huge positive index
        // saturates to i32::MAX, which is past any table.
        let index = number as i32;
        if index < 1 {
            return Err(CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Index must be at least 1".to_string(),
            });
        }
        if index > size {
            return Err(invalid_reference());
        }
        Ok(index - 1)
    }
}
