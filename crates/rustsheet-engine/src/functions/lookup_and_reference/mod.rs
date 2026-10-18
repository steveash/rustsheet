use crate::arithmetic::bcast_get;
use crate::cast::NumberOrArray;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::defined_names::ParsedDefinedName;
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::expressions::utils::is_out_of_range_r1c1_name;
use crate::external_reference::{one_external_area, PickedReference};
use crate::functions::logical::{if_arg_at, if_arg_dims};
use crate::functions::text::util::within_text_budget;
use crate::functions::util::array_node_to_calc_result;
use crate::functions::Function;
use crate::limits::{check_array_size, check_range_size};
use crate::reference_operators::{several_areas_ref, Picked, Ranges, RefArg};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
    utils::ParsedReference,
};

use super::criteria::wildcard_text_matcher;
use crate::functions::lift::{LiftCost, LiftOptions, LiftedArg};
use index_arg::LookupIndexArg;
use lookup_each::LookupSearch;
use lookup_table::{LookupTable, SearchVector};

mod address_areas;
mod choosecols_chooserows;
mod drop_take;
mod eta_min_args;
mod expand;
mod getpivotdata;
mod group_functions;
mod groupby;
mod grouping;
mod hstack_vstack;
mod index;
mod index_arg;
mod lookup;
mod lookup_each;
pub(crate) mod lookup_table;
mod pivotby;
mod relative_to;
mod tocol_torow;
mod transpose;
mod trimrange;
mod wrapcols_wraprows;
pub(crate) mod xmatch;

/// Finds the 0-based index of `lookup_value` within `search_vector`.
/// When `is_sorted` is true Excel's approximate-match search is used (the
/// largest value of the lookup value's type smaller than or equal to it);
/// otherwise a linear search for an exact match (supporting wildcards for
/// text), shared by VLOOKUP/HLOOKUP FALSE and MATCH 0. Returns `None` when
/// the value is not found.
fn lookup_index(
    lookup_value: &CalcResult,
    search_vector: &SearchVector,
    is_sorted: bool,
) -> Option<usize> {
    if is_sorted {
        search_vector.approximate_search(lookup_value, false)
    } else if let CalcResult::String(s) = lookup_value {
        search_vector.position(wildcard_text_matcher(s))
    } else {
        search_vector.exact_position(lookup_value)
    }
}

impl Model {
    /// A call to a function whose result is a reference when its source argument is one,
    /// evaluated as that reference: INDEX, XLOOKUP, TAKE, DROP and TRIMRANGE, which are otherwise
    /// evaluated to their values (probes/intersection-function-operands H8, H11, H12, H14;
    /// T-0165). `None` for any other function.
    pub(crate) fn evaluate_reference_form(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        Some(match kind {
            // `INDEX(A1:A2,2) A1:A2` intersects the cell INDEX picks (T-0235).
            Function::Index => self.index_reference(args, cell),
            Function::Xlookup => self.xlookup_impl(args, cell, true),
            Function::Take => self.take_drop_impl(args, cell, drop_take::Edge::Take, true),
            Function::Drop => self.take_drop_impl(args, cell, drop_take::Edge::Drop, true),
            Function::Trimrange => self.trimrange_impl(args, cell, true),
            _ => return None,
        })
    }

    /// The `lookup_value` of MATCH, XMATCH or LOOKUP. A function that returns a one-cell reference
    /// (`CHOOSE(1,A1)`, `OFFSET(A1,0,0)`, `INDIRECT("A1")`, `A1:A1`) gives that cell's value: Excel
    /// finds 30 for `MATCH(CHOOSE(1,J1),A1:A5,0)` with J1 = 30 (docs/excel-semantics/lookup.md,
    /// probes/lookup-reference-lookup-value). VLOOKUP, HLOOKUP and XLOOKUP get there by lifting.
    /// A multi-cell `lookup_value` is left as it is: XMATCH, MATCH and LOOKUP lift it.
    pub(super) fn lookup_value_arg(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        match self.evaluate_node_in_context(node, cell) {
            range @ CalcResult::Range { left, right } if left == right => {
                self.dereference(range, cell)
            }
            other => other,
        }
    }

    //     MATCH(lookup_value, lookup_array, [match_type])
    // The MATCH function syntax has the following arguments:
    //   * lookup_value    Required. The value that you want to match in lookup_array.
    //                     The lookup_value argument can be a value (number, text, or logical value)
    //                     or a cell reference to a number, text, or logical value.
    //   * lookup_array    Required. The range of cells being searched.
    //   * match_type      Optional. The number -1, 0, or 1.
    //                     The match_type argument specifies how Excel matches lookup_value
    //                     with values in lookup_array. The default value for this argument is 1.
    // Excel: see docs/excel-semantics/lookup.md "Approximate-match lookups".
    pub(crate) fn fn_match(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 3 || args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let target = self.lookup_value_arg(&args[0], cell);
        // An array match_type lifts with lookup_value (probes/match-lookup-value-lift E14:E16,
        // T-1917); a multi-cell range still reads its top-left cell (match_type_arg). A scalar
        // error lookup_value lifts over it too: `MATCH(NA(),A1:A3,{0;1})` is 2x1 of #N/A
        // (probes/match-error-lookup-value-array-match-type E1, E2, E7, E8; T-1926).
        let match_type = if args.len() == 3 {
            self.match_type_arg(&args[2], cell)
        } else {
            CalcResult::Number(1.0)
        };
        if target.is_error() && !LiftedArg::is_grid(&target) && !LiftedArg::is_grid(&match_type) {
            return target;
        }
        if match_type.is_error() && !LiftedArg::is_grid(&match_type) && !LiftedArg::is_grid(&target)
        {
            return match_type;
        }
        let match_range = self.evaluate_single_sheet_arg(&args[1], cell);

        // MATCH operates on a vector (a single row or single column). We
        // materialize the second argument, be it a range reference or an array
        // literal, into a flat list of values and then run the search on it.
        let values = self.as_vector(match_range, cell);

        // lookup_value lifts over an array or a multi-cell range, one search per element. An
        // element that is an error is the result before an invalid match_type is checked:
        // `MATCH({#N/A;1},A1:A3,"x")` is #N/A in slot 1 (probes/match-lookup-value-lift E17; slot 2,
        // #VALUE!, follows from the per-slot match_type check and is not probed;
        // docs/excel-semantics/lookup.md "MATCH lifts over lookup_value", T-1866).
        self.lift_values(
            vec![target, match_type],
            LiftCost::Cells(values.as_ref().map_or(1, SearchVector::scan_cells)),
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |model, args| {
                let [target, match_type] = args else {
                    return CalcResult::new_args_number_error(cell);
                };
                if target.is_error() {
                    return target.clone();
                }
                // Only the sign counts: -0.5 searches descending (gen-m2/f7e6d351c8538ef5 A160,
                // gen-m2/96a0952412cefa6c A241) and 0.5 ascending (probes/match-fractional-match-type
                // C1-C3), so neither is the exact search that 0 is (docs/excel-semantics/lookup.md).
                let match_type = match model.cast_number_ref(match_type, cell) {
                    Ok(v) if v < 0.0 => -1,
                    Ok(v) if v > 0.0 => 1,
                    Ok(_) => 0,
                    Err(error) => return error,
                };
                let values = match &values {
                    Ok(values) => values,
                    Err(error) => return error.clone(),
                };
                let found = if match_type == 0 {
                    lookup_index(target, values, false)
                } else {
                    // Any negative match_type searches descending, any positive one
                    // ascending (probes/lookup-approximate-search N20, N21).
                    values.approximate_search(target, match_type < 0)
                };
                match found {
                    Some(l) => CalcResult::Number(l as f64 + 1.0),
                    None => CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Not found".to_string(),
                    },
                }
            },
        )
    }

    /// MATCH's `match_type` as a number. Text that comes from a cell (a reference) and is not a
    /// number is `#REF!`, while literal text is `#VALUE!` (docs/excel-semantics/lookup.md "MATCH
    /// descending search skips backward", probes/match-descending-argcheck J9-J20; T-1313).
    fn match_type_arg(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        // A range of several cells is its top-left cell, plain or picked by IF, IFS, SWITCH or
        // LET (probes/match-type-nested-passthrough K2, K3, K7:K9, K14, K15, T-1452). An array
        // stays whole: it lifts.
        let value = match self.evaluate_chosen_reference(node, cell) {
            CalcResult::Range { left, .. } => CalcResult::Range { left, right: left },
            other => other,
        };
        let from_cell = matches!(value, CalcResult::Range { .. });
        let value = self.dereference(value, cell);
        // Only non-numeric text is remapped: an error in the cell, or a failed implicit
        // intersection, propagates unchanged (no oracle evidence for remapping those).
        if from_cell {
            if let CalcResult::String(text) = &value {
                if self.cast_number(text).is_none() {
                    return CalcResult::Error {
                        error: Error::REF,
                        origin: cell,
                        message: "Invalid reference".to_string(),
                    };
                }
            }
        }
        value
    }

    /// `HLOOKUP(lookup_value`, `table_array`, `row_index`, [`is_sorted`])
    /// We look for `lookup_value` in the first row of table array
    /// We return the value in row `row_index` of the same column in `table_array`
    /// `is_sorted` is true by default and assumes that values in first row are ordered
    pub(crate) fn fn_hlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.hlookup_impl(args, cell, false)
    }

    /// HLOOKUP; `as_reference` gives the picked cell of a range table as a reference to an
    /// aggregate argument, but only when the row index argument is itself a cell reference
    /// (docs/excel-semantics/references.md, probes/lookup-pick-aggregates P1-P6, Q1-Q6; T-0295).
    pub(crate) fn hlookup_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if args.len() > 4 || args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let (lookup_value, row_index, is_sorted) = match self.lookup_head(args, cell) {
            Ok(head) => head,
            Err(error) => return error,
        };
        let as_reference = as_reference && matches!(row_index, LookupIndexArg::Reference(_));
        let range = self.evaluate_single_sheet_arg(&args[1], cell);
        let table = match range {
            CalcResult::Range { left, right } => LookupTable::Range { left, right },
            CalcResult::Array(array) => LookupTable::Array(array),
            error @ CalcResult::Error { .. } => return error,
            CalcResult::String(_) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
            _ => {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        // We look for `lookup_value` in the first row of the table
        let search_vector = table.first_row(self, cell);
        let search = LookupSearch {
            search_vector: &search_vector,
            index: row_index,
            size: table.rows(),
            is_sorted,
        };
        self.lookup_each(
            lookup_value,
            &search,
            as_reference,
            cell,
            |model, column, row, as_reference| {
                table.pick(model, row, column as i32, cell, as_reference)
            },
        )
    }

    /// `VLOOKUP(lookup_value`, `table_array`, `row_index`, [`is_sorted`])
    /// We look for `lookup_value` in the first column of table array
    /// We return the value in column `column_index` of the same row in `table_array`
    /// `is_sorted` is true by default and assumes that values in first column are ordered
    pub(crate) fn fn_vlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.vlookup_impl(args, cell, false)
    }

    /// VLOOKUP; `as_reference` gives the picked cell of a range table as a reference to an
    /// aggregate argument, but only when the column index argument is itself a cell reference
    /// (docs/excel-semantics/references.md, probes/lookup-pick-aggregates P1-P6, Q1-Q6; T-0295).
    pub(crate) fn vlookup_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if args.len() > 4 || args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let (lookup_value, column_index, is_sorted) = match self.lookup_head(args, cell) {
            Ok(head) => head,
            Err(error) => return error,
        };
        let as_reference = as_reference && matches!(column_index, LookupIndexArg::Reference(_));
        let range = self.evaluate_single_sheet_arg(&args[1], cell);
        let table = match range {
            CalcResult::Range { left, right } => LookupTable::Range { left, right },
            CalcResult::Array(array) => LookupTable::Array(array),
            error @ CalcResult::Error { .. } => return error,
            CalcResult::String(_) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
            _ => {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        // We look for `lookup_value` in the first column of the table
        let search_vector = table.first_column(self, cell);
        let search = LookupSearch {
            search_vector: &search_vector,
            index: column_index,
            size: table.columns(),
            is_sorted,
        };
        self.lookup_each(
            lookup_value,
            &search,
            as_reference,
            cell,
            |model, row, column, as_reference| {
                table.pick(model, row as i32, column, cell, as_reference)
            },
        )
    }

    // HYPERLINK(link_location, [friendly_name])
    // Attaches a link to the cell and returns friendly_name (or link_location
    // if friendly_name is missing). A link_location starting with "#" is an
    // internal link ("#Sheet1!A1"), anything else is external.
    // The link itself is dynamic: it lives in `Model::links` and is rebuilt on
    // every evaluation, it is not part of the workbook.
    // Both arguments lift over an array and a multi-cell range (`ROWS(HYPERLINK(D1:D3))`,
    // `ROWS(HYPERLINK(D1,A1:A3))` are 3; probes/lookup-argument-modes F11:G13,
    // gen-m7/2bb922eef72e785e A22; docs/excel-semantics/lookup.md "Lookup and reference function
    // argument modes"). A lifted call attaches the link of its last element.
    pub(crate) fn fn_hyperlink(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hyperlink, args, cell, &mut |model, values| {
            model.hyperlink_of(values, cell)
        })
    }

    /// HYPERLINK of one element.
    fn hyperlink_of(&mut self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let (location, friendly_name) = match values {
            [location] => (location, None),
            [location, friendly_name] => (location, Some(friendly_name)),
            _ => return CalcResult::new_args_number_error(cell),
        };
        let location = match Self::cast_string_ref(location, cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        // The friendly name is the value displayed in the cell. It can be
        // a string, a number or a boolean.
        let display = match friendly_name {
            Some(value) => value.clone(),
            None => CalcResult::String(location.clone().into()),
        };
        // A cell displaying an error is not clickable: no link is attached
        if matches!(display, CalcResult::Error { .. }) {
            return display;
        }

        let link = match location.strip_prefix('#') {
            Some(internal) => crate::types::Link::Internal {
                location: internal.to_string(),
                tooltip: None,
            },
            None => crate::types::Link::External {
                target: location,
                tooltip: None,
            },
        };
        self.links.insert((cell.sheet, cell.row, cell.column), link);
        display
    }

    // ROW([reference])
    // If reference is not present returns the row of the present cell.
    // Otherwise returns the row number of reference. A reference that spans
    // several rows gives a column with the number of each of them.
    pub(crate) fn fn_row(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.row_or_column(args, cell, true, false)
    }

    /// Shared body of ROW and COLUMN. With `first_only` a multi-cell reference yields just its
    /// first number, which is what implicit intersection (`@ROW(A:A)`) keeps, so the vector over
    /// up to a million rows is never built.
    fn row_or_column(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        by_row: bool,
        first_only: bool,
    ) -> CalcResult {
        if args.len() > 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if args.is_empty() {
            return CalcResult::Number(if by_row { cell.row } else { cell.column } as f64);
        }
        // `ROW(IF(TRUE,…!B3))` is 3 (probes/external-ref-no-link-ifchoose A25).
        let reference = match self.pick_reference(&args[0], cell) {
            PickedReference::Node(node) => node,
            PickedReference::External(areas) => {
                let area = match one_external_area(areas, cell) {
                    Ok(area) => area,
                    Err(error) => return error,
                };
                let (first, last) = if by_row {
                    (area.first_row, area.last_row)
                } else {
                    (area.first_column, area.last_column)
                };
                return row_or_column_numbers(first, last, by_row, first_only);
            }
            PickedReference::Value(error @ CalcResult::Error { .. }) => return error,
            // A value an IF or CHOOSE gives here is no reference: `ROW(IF(FALSE,…!A1))` is
            // `#VALUE!` (probes/external-ref-no-link-ifchoose-b A4).
            PickedReference::Value(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Expected a reference".into())
            }
        };
        let (first, last) = match self.reference_ranges(reference, cell) {
            // ROW and COLUMN of a union are #REF!, not the #VALUE! of a value context
            // (probes/multiarea-cellfn H3, multiarea-cellfn-types H9; SPEC-0004 row 18).
            Ok(Ranges::Areas(_)) => return several_areas_ref(cell),
            Ok(Ranges::One(range)) => {
                let (a, b) = if by_row {
                    (range.left.row, range.right.row)
                } else {
                    (range.left.column, range.right.column)
                };
                (a.min(b), a.max(b))
            }
            Err(s) => return s,
        };
        row_or_column_numbers(first, last, by_row, first_only)
    }

    /// The reference an explicit `@` reads. `@` of a span over several sheets is `#REF!`, not
    /// the `#VALUE!` of a 3-D reference used as a single-sheet one: `=@Data1:Data3!A1` and
    /// `=SUM(@Data1:Data3!A1:A2)` (probes/3d-ref-span-edges B1, B3; T-1170).
    pub(crate) fn evaluate_at_operand_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let node = node.ungrouped();
        match node {
            Node::SheetSpanKind {
                first_index,
                last_index,
                ..
            } if first_index != last_index => CalcResult::new_error(
                Error::REF,
                cell,
                "Implicit intersection of a 3-D reference".to_string(),
            ),
            // `@` intersects the row or column XLOOKUP picks, not its first value:
            // `@XLOOKUP(2,A1:A3,B1:C3)` outside rows 2 and columns B:C is `#VALUE!`
            // (probes/xlookup-row-column-picks E4, G15).
            Node::FunctionKind {
                kind: Function::Xlookup,
                args,
            } => self.xlookup_impl(args, cell, true),
            _ => self
                .cached_external_at(node, cell)
                .unwrap_or_else(|| self.evaluate_node_with_reference(node, cell)),
        }
    }

    /// Evaluates the operand of `@`. `@ROW(A:A)` and `@COLUMN(A1:Z1)` take a fast path that skips
    /// materialising the whole vector; everything else is evaluated normally.
    pub(crate) fn evaluate_implicit_intersection_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let node = node.ungrouped();
        match node {
            Node::FunctionKind {
                kind: Function::Row,
                args,
            } => self.row_or_column(args, cell, true, true),
            Node::FunctionKind {
                kind: Function::Column,
                args,
            } => self.row_or_column(args, cell, false, true),
            _ => self.evaluate_at_operand_reference(node, cell),
        }
    }

    // ROWS(range)
    // Returns the number of rows in range
    pub(crate) fn fn_rows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.rows_columns_extent(&args[0], cell) {
            Ok((rows, _)) => CalcResult::Number(rows as f64),
            Err(s) => s,
        }
    }

    // COLUMN([reference])
    // If reference is not present returns the column of the present cell.
    // Otherwise returns the column number of reference. A reference that spans
    // several columns gives a row with the number of each of them.
    pub(crate) fn fn_column(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.row_or_column(args, cell, false, false)
    }

    /// `CHOOSE(index_num`, value1, [value2], ...)
    /// Uses `index_num` to return a value from the list of value arguments.
    pub(crate) fn fn_choose(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.choose_impl(args, cell, false).into_single_area(cell)
    }

    /// CHOOSE; with `as_reference` a scalar index hands back the chosen argument in
    /// reference context, so a union there stays its areas: `SUM(CHOOSE(2,A1,(A1,B2)))` is 6
    /// (probes/multiarea-ifchoose H3).
    pub(crate) fn choose_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> RefArg {
        let node = match self.choose_pick(args, cell) {
            Picked::Node(node) => node,
            Picked::Value(value) => return RefArg::Value(value),
        };
        if as_reference {
            return self.evaluate_reference_arg(node, cell);
        }
        // A reference into another workbook picked reads as the cell INDEX picks: `CHOOSE(1,…!A1)`
        // is 0 (probes/external-ref-no-link-ifchoose A2, A12).
        self.external_reference_picked_value(node, cell)
            .unwrap_or_else(|| self.evaluate_node_with_reference(node, cell))
            .into()
    }

    /// What CHOOSE's index picks: with a scalar index the argument it chooses, unevaluated;
    /// otherwise the value CHOOSE gives (the index's error, `#VALUE!` out of range, or CHOOSE
    /// over an index array).
    pub(crate) fn choose_pick<'a>(
        &mut self,
        args: &'a [Node],
        cell: CellReferenceIndex,
    ) -> Picked<'a> {
        if args.len() < 2 {
            return Picked::Value(CalcResult::new_args_number_error(cell));
        }

        // The index argument may be a single number or an array/range; over an array it lifts
        // (see `choose_array`).
        match self.get_number_or_array(&args[0], cell) {
            Ok(NumberOrArray::Number(index_num)) => {
                let index_num = index_num as usize;
                if index_num < 1 || index_num > (args.len() - 1) {
                    let error = CalcResult::new_error(Error::VALUE, cell, "Invalid index".into());
                    return Picked::Value(error);
                }
                Picked::Node(&args[index_num])
            }
            Ok(NumberOrArray::Array(index_array)) => {
                Picked::Value(self.choose_array(args, &index_array, cell))
            }
            Err(calc_err) => Picked::Value(calc_err),
        }
    }

    /// CHOOSE over an index array, broadcast like IF over an array condition: every value
    /// argument is evaluated as a scalar or an array, the result spans the largest extent of the
    /// index and all of them (an argument no element picks included), and each position takes
    /// the picked argument's element there under size-1 broadcasting. A ragged position is
    /// `#N/A`, an index element out of range `#VALUE!`: `CHOOSE({1,2},D1:D2,D3:D4)` is the 2x2
    /// `{D1,D3;D2,D4}` (probes/choose-array-index-range-arms, docs/excel-semantics/lookup.md).
    fn choose_array(
        &mut self,
        args: &[Node],
        index_array: &[Vec<ArrayNode>],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut values = Vec::with_capacity(args.len() - 1);
        for arg in &args[1..] {
            match self.evaluate_if_arg(arg, cell) {
                Ok(value) => values.push(value),
                Err(error) => return error,
            }
        }
        let index_rows = index_array.len();
        let index_cols = index_array.first().map_or(0, Vec::len);
        let (rows, cols) = values
            .iter()
            .map(if_arg_dims)
            .fold((index_rows, index_cols), |(r, c), (vr, vc)| {
                (r.max(vr), c.max(vc))
            });
        if let Some((error, message)) = check_array_size(rows, cols) {
            return CalcResult::new_error(error, cell, message);
        }
        let mut output = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut line = Vec::with_capacity(cols);
            for col in 0..cols {
                let node = match bcast_get(index_array, row, col) {
                    None => ArrayNode::Error(Error::NA),
                    Some(index) => match self.choose_index_element(index, values.len()) {
                        Ok(picked) => if_arg_at(&values[picked], row, col),
                        Err(error) => error,
                    },
                };
                line.push(node);
            }
            output.push(line);
        }
        CalcResult::Array(output)
    }

    /// The 0-based value argument one CHOOSE index element picks among `count`, or the error
    /// element CHOOSE gives there (the element's own error; `#VALUE!` for text that is no number
    /// or an index out of range). TRUE is 1, FALSE and an empty element 0.
    fn choose_index_element(&self, index: &ArrayNode, count: usize) -> Result<usize, ArrayNode> {
        // The cast truncates a fraction and saturates: a negative or NaN index is 0, out of range.
        let index_num = self.array_node_to_number(index).map_err(ArrayNode::Error)? as usize;
        if index_num < 1 || index_num > count {
            return Err(ArrayNode::Error(Error::VALUE));
        }
        Ok(index_num - 1)
    }

    // COLUMNS(range)
    // Returns the number of columns in range
    pub(crate) fn fn_columns(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.rows_columns_extent(&args[0], cell) {
            Ok((_, columns)) => CalcResult::Number(columns as f64),
            Err(s) => s,
        }
    }

    // INDIRECT(ref_text, [a1])
    // Returns the reference specified by 'ref_text'. It is read in A1 notation
    // unless 'a1' is FALSE, in which case it is read in R1C1 notation.
    pub(crate) fn fn_indirect(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let ref_text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(v) => return v,
        };
        let a1 = if args.len() == 2 {
            match self.get_boolean(&args[1], cell) {
                Ok(b) => b,
                Err(v) => return v,
            }
        } else {
            true
        };

        let parsed_reference = if a1 {
            ParsedReference::parse_reference_formula(
                Some(cell.sheet),
                &ref_text,
                self.locale,
                |name| self.get_sheet_index_by_name(name),
            )
        } else {
            ParsedReference::parse_r1c1_reference(cell, &ref_text, |name| {
                self.get_sheet_index_by_name(name)
            })
        };

        match parsed_reference {
            Ok(ParsedReference::CellReference(reference)) => CalcResult::Range {
                left: reference,
                right: reference,
            },
            Ok(ParsedReference::Range(left, right)) => CalcResult::Range { left, right },
            Err(message) => match self.indirect_defined_name(&ref_text, cell.sheet) {
                Some(found) => found,
                None => CalcResult::Error {
                    error: Error::REF,
                    origin: cell,
                    message,
                },
            },
        }
    }

    /// INDIRECT of the text of a defined name that stands for a reference (`INDIRECT("RCR")`,
    /// `ISREF` TRUE; probes/r1c1-name-use-rcr, excel-graph). A name spelled like an out-of-range
    /// R1C1 reference is never resolved (T-1657).
    ///
    /// Excel: see docs/excel-semantics/lookup.md "INDIRECT resolves a defined name that holds a
    /// reference": `a1` FALSE still resolves a name, trailing (not leading) spaces are ignored, and
    /// a `Sheet!name` prefix picks the name scoped to that sheet, else the workbook name
    /// (probes/indirect-defined-name-text A5, A8, A22; T-1682).
    fn indirect_defined_name(&self, text: &str, sheet: u32) -> Option<CalcResult> {
        let text = text.trim_end_matches(' ');
        if is_out_of_range_r1c1_name(text) {
            return None;
        }
        let scope = match text.rsplit_once('!') {
            Some((prefix, _)) => {
                let prefix = prefix
                    .strip_prefix('\'')
                    .and_then(|p| p.strip_suffix('\''))
                    .unwrap_or(prefix);
                self.get_sheet_index_by_name(prefix)?
            }
            None => sheet,
        };
        let found = self
            .parsed_defined_name(text, Some(scope))
            .or_else(|| self.parsed_defined_name(text, None));
        match found? {
            ParsedDefinedName::CellReference(reference) => Some(CalcResult::Range {
                left: *reference,
                right: *reference,
            }),
            ParsedDefinedName::RangeReference(range) => Some(CalcResult::Range {
                left: range.left,
                right: range.right,
            }),
            _ => None,
        }
    }

    /// Height or width argument of OFFSET as the signed distance from the anchor cell to the far
    /// edge of the area; `default` when the argument is missing or empty, `None` for exactly 0.
    ///
    /// Excel: see docs/excel-semantics/references.md "OFFSET negative and fractional height/width".
    /// A positive size `h` reaches `floor(h - 1)` cells on, a negative one `ceil(h + 1)` cells (so
    /// -3 reaches 2 back), and the fractions in (-1, 1) reach one cell back and on respectively.
    fn offset_size(
        &mut self,
        arg: Option<&Node>,
        default: i32,
        cell: CellReferenceIndex,
    ) -> Result<Option<i32>, CalcResult> {
        match arg {
            None | Some(Node::EmptyArgKind) => Ok(Some(default)),
            Some(node) => {
                let c = self.get_number(node, cell)?;
                Ok(if c > 0.0 {
                    Some((c - 1.0).floor() as i32)
                } else if c < 0.0 {
                    Some((c + 1.0).ceil() as i32)
                } else {
                    None
                })
            }
        }
    }

    // OFFSET(reference, rows, cols, [height], [width])
    // Returns a reference to a range that is a specified number of rows and columns from a cell or range of cells.
    // The reference that is returned can be a single cell or a range of cells.
    // You can specify the number of rows and the number of columns to be returned.
    pub(crate) fn fn_offset(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let l = args.len();
        if !(3..=5).contains(&l) {
            return CalcResult::new_args_number_error(cell);
        }
        // OFFSET of a reference into another workbook is `#VALUE!`, not a reference
        // (`ISREF(OFFSET(…!A1,1,1))` is FALSE; probes/external-ref-no-link-shapes A5,
        // external-ref-no-link-index A16, A17, A32).
        // An error in `rows`, `cols` or `width` comes first: `OFFSET(…!A1,1/0,1)`,
        // `OFFSET(…!A1,1,1/0)` and `OFFSET(…!A1,1,1,1,1/0)` are `#DIV/0!`, but a height of 0 is
        // still `#VALUE!` (probes/external-ref-no-link-ifchoose A7, -ifchoose-b A2, A3, A6).
        if let Some(areas) = self.external_reference(&args[0], cell) {
            if let Err(error) = areas {
                return error;
            }
            for arg in &args[1..] {
                if let Err(error) = self.get_number(arg, cell) {
                    return error;
                }
            }
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "OFFSET of an external reference".to_string(),
            );
        }
        let reference = match self.get_reference(&args[0], cell) {
            Ok(c) => c,
            Err(s) => return s,
        };
        let rows = match self.get_number(&args[1], cell) {
            Ok(c) => {
                if c < 0.0 {
                    c.ceil() as i32
                } else {
                    c.floor() as i32
                }
            }
            Err(s) => return s,
        };
        let cols = match self.get_number(&args[2], cell) {
            Ok(c) => {
                if c < 0.0 {
                    c.ceil() as i32
                } else {
                    c.floor() as i32
                }
            }
            Err(s) => return s,
        };
        // `as i32` saturates, so sums saturate too (untrusted input must not overflow): any such
        // result is outside the sheet and falls through to #REF!.
        let row_start = reference.left.row.saturating_add(rows);
        let column_start = reference.left.column.saturating_add(cols);
        // A missing or empty height/width keeps the reference's size
        // (poi/FormulaEvalTestData_Copy.xlsx EverythingTests!I1008, J1008, M1008, N1008).
        let height =
            match self.offset_size(args.get(3), reference.right.row - reference.left.row, cell) {
                Ok(h) => h,
                Err(e) => return e,
            };
        let width = match self.offset_size(
            args.get(4),
            reference.right.column - reference.left.column,
            cell,
        ) {
            Ok(w) => w,
            Err(e) => return e,
        };
        let (Some(height), Some(width)) = (height, width) else {
            // A size of exactly 0 is #REF!, not an empty area.
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        };
        // A negative size extends the area up/left from the anchor cell.
        let column_end = column_start.saturating_add(width);
        let row_end = row_start.saturating_add(height);
        let (row_start, row_end) = (row_start.min(row_end), row_start.max(row_end));
        let (column_start, column_end) =
            (column_start.min(column_end), column_start.max(column_end));
        if row_start < 1 || row_end > LAST_ROW || column_start < 1 || column_end > LAST_COLUMN {
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        }
        let left = CellReferenceIndex {
            sheet: reference.left.sheet,
            row: row_start,
            column: column_start,
        };
        let right = CellReferenceIndex {
            sheet: reference.right.sheet,
            row: row_end,
            column: column_end,
        };
        CalcResult::Range { left, right }
    }

    // FORMULATEXT(reference)
    // Returns a formula as a string. Two differences with Excel:
    // - It returns the formula in English
    // - It formats the formula without spaces between elements
    pub(crate) fn fn_formulatext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if let CalcResult::Range { left, right } = self.evaluate_node_with_reference(&args[0], cell)
        {
            if left.sheet != right.sheet {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "3D ranges not supported".to_string(),
                };
            }
            if left == right {
                return self.formula_text_of(left, cell);
            }
            // A multi-cell reference gives one result per cell, `#N/A` for a cell with no
            // formula (`ROWS(FORMULATEXT(A1:A3))` is 3; probes/lookup-argument-modes F38:H38,
            // docs/excel-semantics/lookup.md "Lookup and reference function argument modes").
            // The cells and the total text are bounded (CLAUDE.md directive 5).
            if let Some((error, message)) = check_range_size(&left, &right) {
                return CalcResult::new_error(error, cell, message);
            }
            let mut text_bytes = 0;
            let width = usize::try_from(right.column - left.column + 1).unwrap_or(0);
            let mut rows =
                Vec::with_capacity(usize::try_from(right.row - left.row + 1).unwrap_or(0));
            for row in left.row..=right.row {
                let mut values = Vec::with_capacity(width);
                for column in left.column..=right.column {
                    let at = CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    };
                    values.push(match self.formula_text_of(at, cell) {
                        CalcResult::String(text) => {
                            if !within_text_budget(&mut text_bytes, text.len()) {
                                return CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Total text result is too large".to_string(),
                                );
                            }
                            ArrayNode::String(text)
                        }
                        _ => ArrayNode::Error(Error::NA),
                    });
                }
                rows.push(values);
            }
            CalcResult::Array(rows)
        } else {
            CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Argument must be a reference".to_string(),
            }
        }
    }

    /// FORMULATEXT of the one cell `at`: its formula, or `#N/A` when it holds none.
    fn formula_text_of(&self, at: CellReferenceIndex, cell: CellReferenceIndex) -> CalcResult {
        if let Ok(Some(f)) = self.get_english_cell_formula(at.sheet, at.row, at.column) {
            CalcResult::String(f.into())
        } else {
            CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Reference does not have a formula".to_string(),
            }
        }
    }
}

/// ROW's numbers `first..=last` down a column (`by_row`) or COLUMN's across a row; one number
/// when they are one or with `first_only` (implicit intersection keeps the first).
fn row_or_column_numbers(first: i32, last: i32, by_row: bool, first_only: bool) -> CalcResult {
    if first == last || first_only {
        return CalcResult::Number(f64::from(first));
    }
    let numbers = (first..=last).map(|n| ArrayNode::Number(f64::from(n)));
    CalcResult::Array(if by_row {
        numbers.map(|n| vec![n]).collect()
    } else {
        vec![numbers.collect()]
    })
}
