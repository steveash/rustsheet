use crate::arithmetic::{bcast_get, extended_quotient, flush_subnormal, MISSING};
use crate::cast::{calc_result_to_array_node, number_text_too_long};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::criteria::build_criteria;
use crate::functions::util::array_node_to_calc_result;
use crate::limits::check_array_size;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    model::Model,
};

/// A compiled criterion predicate, as returned by `build_criteria`.
type Criterion<'c> = Box<dyn Fn(&CalcResult) -> bool + 'c>;

/// Rows and columns spanned by a range.
fn range_shape(range: &Range) -> (i32, i32) {
    (
        range.right.row - range.left.row,
        range.right.column - range.left.column,
    )
}

/// True when every range has the same rows x columns as the first one.
fn same_shape(ranges: &[Range]) -> bool {
    ranges
        .first()
        .is_none_or(|first| ranges.iter().all(|r| range_shape(r) == range_shape(first)))
}

/// `anchor`'s top-left cell with `shape_of`'s rows x columns, clamped to the grid.
fn resize_to(anchor: &Range, shape_of: &Range) -> Range {
    let (rows, columns) = range_shape(shape_of);
    let mut resized = Range {
        left: anchor.left,
        right: anchor.left,
    };
    resized.right.row = (anchor.left.row + rows).min(LAST_ROW);
    resized.right.column = (anchor.left.column + columns).min(LAST_COLUMN);
    resized
}

/// One criteria argument of an *IF(S) function, as [`Model::lift_criteria`] sees it.
enum CriteriaArg {
    /// A single value: every element of a lifted result uses it.
    Scalar(CalcResult),
    /// An array, or a multi-cell range read element by element.
    Grid(Vec<Vec<ArrayNode>>),
}

const TOO_LONG: &str = "Criterion too long";

fn criterion_too_long(criterion: &CalcResult) -> bool {
    matches!(criterion, CalcResult::String(text) if number_text_too_long(text))
}

impl CriteriaArg {
    fn dims(&self) -> (usize, usize) {
        match self {
            CriteriaArg::Scalar(_) => (1, 1),
            CriteriaArg::Grid(grid) => (grid.len(), grid.first().map_or(0, Vec::len)),
        }
    }

    /// The criterion at output position (`row`, `column`): a dimension of length 1 stretches,
    /// a position past a shorter array is `#N/A` (probes/ifs-array-criteria H11, H33-H37).
    fn element(&self, row: usize, column: usize) -> Option<&ArrayNode> {
        match self {
            CriteriaArg::Scalar(_) => None,
            CriteriaArg::Grid(grid) => Some(bcast_get(grid, row, column).unwrap_or(&MISSING)),
        }
    }
}

/// Most cells all the per-element walks of one lifted *IF(S) call may visit, so a crafted formula
/// (`COUNTIFS(A:A,SEQUENCE(1048576))` over a filled column) cannot run for hours (CLAUDE.md
/// directive 5). Excel has no such limit: DEVIATION, docs/excel-semantics/criteria.md
/// ("*IF(S) lift array criteria").
const MAX_LIFTED_CELL_VISITS: u64 = 1 << 27;

/// Elements [`Model::lift_criteria`] computes: one per run of equal consecutive criteria, in
/// row-major order (the rest reuse the previous result).
fn count_walks(args: &[CriteriaArg], rows: usize, columns: usize) -> u64 {
    let mut walks = 0;
    let mut previous: Option<(usize, usize)> = None;
    for row in 0..rows {
        for column in 0..columns {
            let same = previous.is_some_and(|(r, c)| {
                args.iter()
                    .all(|arg| arg.element(row, column) == arg.element(r, c))
            });
            if !same {
                walks += 1;
            }
            previous = Some((row, column));
        }
    }
    walks
}

fn shape_mismatch(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "Ranges must have the same shape".to_string(),
    )
}

impl Model {
    /// Evaluates the criteria arguments `args[first]`, `args[first + 2]`, ...
    fn criteria_values(
        &mut self,
        args: &[Node],
        first: usize,
        cell: CellReferenceIndex,
    ) -> Vec<CalcResult> {
        args.iter()
            .skip(first)
            .step_by(2)
            .map(|node| self.evaluate_node_in_context(node, cell))
            .collect()
    }

    /// Runs an *IF(S) function once per criteria element: when any criterion is an array or a
    /// multi-cell range, the result is an array of the criteria broadcast against each other,
    /// as the element-wise operators do (a row and a column give a grid; a position past a
    /// shorter array uses an `#N/A` criterion). Otherwise `compute` runs once with the
    /// criteria as they are. Evidence: probes/ifs-array-criteria, poi/70005-countifs.xlsx
    /// (docs/excel-semantics/criteria.md "*IF(S) lift array criteria").
    ///
    /// `walked` is the range every element walks (the value range, or COUNTIFS's first criteria
    /// range): the call is `#ERROR!` when its used cells times the criteria count times the
    /// elements to compute exceed [`MAX_LIFTED_CELL_VISITS`].
    ///
    fn lift_criteria(
        &mut self,
        criteria: Vec<CalcResult>,
        walked: &Node,
        cell: CellReferenceIndex,
        compute: impl FnMut(&mut Self, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        self.lift_with_cost(
            criteria,
            |model| model.walk_cost(walked, cell),
            cell,
            compute,
        )
    }

    /// [`Model::lift_criteria`] with the cells one element visits given by `walk_cost` (called
    /// only when something lifts). AGGREGATE lifts its `function_num`, options and k this way.
    pub(crate) fn lift_with_cost(
        &mut self,
        criteria: Vec<CalcResult>,
        walk_cost: impl FnOnce(&mut Self) -> u64,
        cell: CellReferenceIndex,
        mut compute: impl FnMut(&mut Self, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        if !criteria
            .iter()
            .any(|c| matches!(c, CalcResult::Range { .. } | CalcResult::Array(_)))
        {
            return compute(self, &criteria);
        }
        let mut args = Vec::with_capacity(criteria.len());
        for criterion in criteria {
            args.push(match criterion {
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    match self.evaluate_range(left, right, cell) {
                        Ok(grid) => CriteriaArg::Grid(grid),
                        Err(error) => return error,
                    }
                }
                CalcResult::Array(grid) => CriteriaArg::Grid(grid),
                scalar => CriteriaArg::Scalar(scalar),
            });
        }
        let (rows, columns) = args
            .iter()
            .map(CriteriaArg::dims)
            .fold((1, 1), |acc, d| (acc.0.max(d.0), acc.1.max(d.1)));
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        // Refuse before walking anything when the walks would visit too many cells.
        let walks = count_walks(&args, rows, columns);
        let criteria_count = u64::try_from(args.len()).unwrap_or(u64::MAX);
        let visits = walk_cost(self)
            .saturating_mul(criteria_count)
            .saturating_mul(walks);
        if visits > MAX_LIFTED_CELL_VISITS {
            return CalcResult::new_error(
                Error::ERROR,
                cell,
                "Too many cells to visit for array criteria".to_string(),
            );
        }
        // Consecutive equal criteria (the blank tail of a whole-column criteria range) reuse
        // the previous result instead of walking the ranges again.
        let mut previous: Option<(Vec<Option<ArrayNode>>, ArrayNode)> = None;
        let mut output = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut out_row = Vec::with_capacity(columns);
            for column in 0..columns {
                let key: Vec<Option<ArrayNode>> = args
                    .iter()
                    .map(|arg| arg.element(row, column).cloned())
                    .collect();
                let value = match &previous {
                    Some((previous_key, value)) if *previous_key == key => value.clone(),
                    _ => {
                        let values: Vec<CalcResult> = args
                            .iter()
                            .zip(&key)
                            .map(|(arg, element)| match (arg, element) {
                                (CriteriaArg::Scalar(value), _) => value.clone(),
                                (_, Some(node)) => array_node_to_calc_result(node, cell),
                                (_, None) => CalcResult::EmptyCell,
                            })
                            .collect();
                        let result = compute(self, &values);
                        let value = calc_result_to_array_node(result);
                        previous = Some((key, value.clone()));
                        value
                    }
                };
                out_row.push(value);
            }
            output.push(out_row);
        }
        CalcResult::Array(output)
    }

    /// Cells of `node`'s range inside the used area (at least 1): what one criteria walk visits.
    pub(crate) fn walk_cost(&mut self, node: &Node, cell: CellReferenceIndex) -> u64 {
        let CalcResult::Range { left, right } = self.evaluate_node_with_reference(node, cell)
        else {
            return 1;
        };
        let Ok((last_row, last_column)) =
            self.clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
        else {
            return 1;
        };
        let rows = u64::try_from(last_row - left.row + 1).unwrap_or(0);
        let columns = u64::try_from(last_column - left.column + 1).unwrap_or(0);
        (rows * columns).max(1)
    }

    /// The SUMIFS-shaped functions `(value_range, criteria_range1, criteria1, ...)`: checks the
    /// argument count, then lifts `compute` over the criteria (see [`Model::lift_criteria`]).
    pub(crate) fn lift_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        compute: impl FnMut(&mut Self, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        if args.len() < 3 || args.len().is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }
        let criteria = self.criteria_values(args, 2, cell);
        self.lift_criteria(criteria, &args[0], cell, compute)
    }

    pub(crate) fn fn_countif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let arguments = vec![args[0].clone(), args[1].clone()];
            self.fn_countifs(&arguments, cell)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    /// `AVERAGEIF(criteria_range`, criteria, [`average_range`])
    /// if `average_rage` is missing then `criteria_range` will be used
    pub(crate) fn fn_averageif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arguments = match args.len() {
            2 => vec![args[0].clone(), args[0].clone(), args[1].clone()],
            // Unlike AVERAGEIFS, a differently shaped average_range is not an
            // error: it is resized to the criteria range (probes/ifs-shape-mismatch F9–F11).
            3 => vec![args[2].clone(), args[0].clone(), args[1].clone()],
            _ => return CalcResult::new_args_number_error(cell),
        };
        self.average_ifs(&arguments, cell, false)
    }

    pub(crate) fn fn_countifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || !args.len().is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }
        let criteria = self.criteria_values(args, 1, cell);
        self.lift_criteria(criteria, &args[0], cell, |model, criteria| {
            model.count_ifs(args, criteria, cell)
        })
    }

    /// Resolves the criteria ranges of COUNTIFS and the SUMIFS-shaped functions and builds their
    /// predicates: `args[first + 2k]` is the range of `criteria[k]`.
    ///
    /// The ranges are taken by reference (a single-cell range `A1` is a 1x1 range, probes/
    /// sumif-edge-single-cell F11-F15) and a cross-sheet range or a non-range is `#VALUE!`.
    /// `shape_error` then checks their shapes. A criterion over 255 characters is `#VALUE!` (from a
    /// literal, a cell or one element of a range), but only once the ranges and shapes resolved: a
    /// bad range wins (probes/long-text-args-min-criteria A10-A26, probes/ifs-long-criterion-precedence).
    fn resolve_ifs_criteria<'c>(
        &mut self,
        args: &[Node],
        first: usize,
        criteria: &'c [CalcResult],
        cell: CellReferenceIndex,
        shape_error: impl FnOnce(&[Range]) -> Option<CalcResult>,
    ) -> Result<(Vec<Range>, Vec<Criterion<'c>>), CalcResult> {
        let mut ranges = Vec::with_capacity(criteria.len());
        for case_index in 0..criteria.len() {
            let result = self.evaluate_node_with_reference(&args[first + case_index * 2], cell);
            if result.is_error() {
                return Err(result);
            }
            if let CalcResult::Range { left, right } = result {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                ranges.push(Range { left, right });
            } else {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expected a range".to_string(),
                ));
            }
        }
        if let Some(error) = shape_error(&ranges) {
            return Err(error);
        }
        if criteria.iter().any(criterion_too_long) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                TOO_LONG.to_string(),
            ));
        }
        let fn_criteria = criteria
            .iter()
            .map(|criterion| {
                build_criteria(
                    criterion,
                    self.locale,
                    &|| self.current_year(),
                    self.workbook.settings.date_system,
                )
            })
            .collect();
        Ok((ranges, fn_criteria))
    }

    /// COUNTIFS with the criteria already evaluated (`criteria[k]` is `args[2k + 1]`'s value).
    fn count_ifs(
        &mut self,
        args: &[Node],
        criteria: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let case_count = criteria.len();
        // Excel: any criteria range shaped unlike the first is #VALUE! (T-0209).
        let (ranges, fn_criteria) =
            match self.resolve_ifs_criteria(args, 0, criteria, cell, |ranges| {
                (!same_shape(ranges)).then(|| shape_mismatch(cell))
            }) {
                Ok(resolved) => resolved,
                Err(error) => return error,
            };

        let mut total = 0.0;
        let first_range = &ranges[0];
        let left_row = first_range.left.row;
        let left_column = first_range.left.column;
        let right_row = first_range.right.row;
        let right_column = first_range.right.column;

        let (max_row, max_column) = match self.workbook.worksheet(first_range.left.sheet) {
            Ok(s) => s.used_area_end(),
            Err(_) => {
                return CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    format!("Invalid worksheet index: '{}'", first_range.left.sheet),
                )
            }
        };

        let open_row = left_row == 1 && right_row == LAST_ROW;
        let open_column = left_column == 1 && right_column == LAST_COLUMN;

        // The cells beyond the used area are taken as empty without being
        // read; every range depends on them, so they go on record first.
        for range in ranges.iter() {
            let sheet = range.left.sheet;
            let (top, left) = (range.left.row, range.left.column);
            let (bottom, right) = (range.right.row, range.right.column);
            if open_row {
                let first_skipped = top + (max_row + 1 - left_row);
                self.record_seen_empty_range(sheet, first_skipped, left, bottom, right);
            }
            if open_column {
                let first_skipped = left + (max_column + 1 - left_column);
                let last_walked_row = if open_row {
                    top + (max_row - left_row)
                } else {
                    bottom
                };
                self.record_seen_empty_range(sheet, top, first_skipped, last_walked_row, right);
            }
        }

        for row in left_row..right_row + 1 {
            if open_row && row > max_row {
                // If the row is larger than the max row in the sheet then all cells are empty.
                // We compute it only once
                let mut is_true = true;
                for fn_criterion in fn_criteria.iter() {
                    if !fn_criterion(&CalcResult::EmptyCell) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    total += ((LAST_ROW - max_row) * (right_column - left_column + 1)) as f64;
                }
                break;
            }
            for column in left_column..right_column + 1 {
                if open_column && column > max_column {
                    // If the column is larger than the max column in the sheet then all cells are empty.
                    // We compute it only once
                    let mut is_true = true;
                    for fn_criterion in fn_criteria.iter() {
                        if !fn_criterion(&CalcResult::EmptyCell) {
                            is_true = false;
                            break;
                        }
                    }
                    if is_true {
                        total += (LAST_COLUMN - max_column) as f64;
                    }
                    break;
                }
                let mut is_true = true;
                for case_index in 0..case_count {
                    // We check if value in range n meets criterion n
                    let range = &ranges[case_index];
                    let fn_criterion = &fn_criteria[case_index];
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: range.left.sheet,
                        row: range.left.row + row - first_range.left.row,
                        column: range.left.column + column - first_range.left.column,
                    });
                    if !fn_criterion(&value) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    total += 1.0;
                }
            }
        }
        CalcResult::Number(total)
    }

    /// The walk of the SUMIFS-shaped functions, with the criteria already evaluated
    /// (`criteria[k]` is `args[2k + 2]`'s value; see [`Model::lift_ifs`]).
    pub(crate) fn apply_ifs<F>(
        &mut self,
        args: &[Node],
        criteria: &[CalcResult],
        cell: CellReferenceIndex,
        strict_shape: bool,
        apply: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        // By reference: a single-cell sum_range (`A1`) is a 1x1 range to resize from.
        let arg_0 = self.evaluate_node_with_reference(&args[0], cell);
        if arg_0.is_error() {
            return Err(arg_0);
        }
        let mut sum_range = if let CalcResult::Range { left, right } = arg_0 {
            if left.sheet != right.sheet {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Ranges are in different sheets".to_string(),
                ));
            }
            Range { left, right }
        } else {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected a range".to_string(),
            ));
        };

        // Every range shaped like sum_range, else #VALUE! (T-0209; SUMIFS from
        // gen-m2, AVERAGEIFS/MINIFS/MAXIFS from probes/ifs-shape-mismatch). Only
        // SUMIF's single-criterion path opts out: it never errors (T-0214).
        let (ranges, fn_criteria) =
            self.resolve_ifs_criteria(args, 1, criteria, cell, |ranges| {
                (strict_shape
                    && (!same_shape(ranges)
                        || ranges.first().map(range_shape) != Some(range_shape(&sum_range))))
                .then(|| shape_mismatch(cell))
            })?;
        if !strict_shape {
            // SUMIF/AVERAGEIF sum a sum_range resized to the shape of the criteria
            // range from its top-left cell (gen-m2/04cfd82f54dbc2b2 Calc!A344,
            // gen-m2/629e6e8b76fab8b9 Calc!A217; probes/ifs-shape-mismatch F9-F11).
            sum_range = resize_to(&sum_range, &ranges[0]);
        }

        self.run_ifs(&sum_range, &ranges, &fn_criteria, cell, apply)
    }

    /// Walks `sum_range` and applies `apply` to every numeric cell whose parallel
    /// cell in each criteria range satisfies the matching criterion. `ranges` and
    /// `fn_criteria` are parallel (one criteria range and one predicate per case).
    ///
    /// The walk behind [`Model::apply_ifs`].
    fn run_ifs<F>(
        &mut self,
        sum_range: &Range,
        ranges: &[Range],
        fn_criteria: &[Criterion<'_>],
        cell: CellReferenceIndex,
        mut apply: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        let left_row = sum_range.left.row;
        let left_column = sum_range.left.column;
        let mut right_row = sum_range.right.row;
        let mut right_column = sum_range.right.column;

        match self.clip_to_used_area(
            sum_range.left.sheet,
            left_row,
            left_column,
            right_row,
            right_column,
        ) {
            Ok((r, c)) => {
                right_row = r;
                right_column = c;
            }
            Err(message) => return Err(CalcResult::new_error(Error::ERROR, cell, message)),
        }

        for row in left_row..right_row + 1 {
            for column in left_column..right_column + 1 {
                let mut is_true = true;
                for (range, fn_criterion) in ranges.iter().zip(fn_criteria.iter()) {
                    // We check if value in range n meets criterion n
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: range.left.sheet,
                        row: range.left.row + row - sum_range.left.row,
                        column: range.left.column + column - sum_range.left.column,
                    });
                    if !fn_criterion(&value) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    let v = self.evaluate_cell(CellReferenceIndex {
                        sheet: sum_range.left.sheet,
                        row,
                        column,
                    });
                    match v {
                        CalcResult::Number(n) => apply(n),
                        CalcResult::Error { .. } => return Err(v),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// SUMIF: the SUMIFS walk on `(sum_range, criteria_range, criteria)`, minus its shape
    /// check, since SUMIF never errors on a shape mismatch (T-0214). An array or range
    /// criterion gives one sum per element (see [`Model::lift_criteria`]).
    pub(crate) fn sumif(
        &mut self,
        criteria_range: &Node,
        criteria: &Node,
        sum_range: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arguments = [sum_range.clone(), criteria_range.clone(), criteria.clone()];
        self.sum_ifs(&arguments, cell, false)
    }

    /// SUMIFS (`strict_shape`) and SUMIF on `(sum_range, criteria_range, criteria, ...)`, lifted
    /// over array criteria; a sum beyond the double range is `#NUM!` (T-0991).
    pub(crate) fn sum_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        strict_shape: bool,
    ) -> CalcResult {
        self.lift_ifs(args, cell, |model, criteria| {
            let mut total = 0.0;
            match model.apply_ifs(args, criteria, cell, strict_shape, |v| total += v) {
                Ok(()) => CalcResult::finite_number_or_num(total, cell),
                Err(e) => e,
            }
        })
    }

    pub(crate) fn fn_averageifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.average_ifs(args, cell, true)
    }

    /// AVERAGEIFS, lifted over array criteria; AVERAGEIF passes `strict_shape: false`.
    fn average_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        strict_shape: bool,
    ) -> CalcResult {
        self.lift_ifs(args, cell, |model, criteria| {
            let mut total = 0.0;
            let mut count = 0.0;
            let average = |value: f64| {
                total += value;
                count += 1.0;
            };
            if let Err(e) = model.apply_ifs(args, criteria, cell, strict_shape, average) {
                return e;
            }
            if count == 0.0 {
                return CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "division by 0".to_string(),
                };
            }
            // The division rounds twice like AVERAGE's (docs/excel-semantics/numbers.md "Division
            // rounds twice"; probes-numeric/average-family-large-count-division, T-1636).
            CalcResult::Number(flush_subnormal(extended_quotient(total, count)))
        })
    }

    pub(crate) fn fn_minifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_ifs(args, cell, |model, criteria| {
            let mut min = f64::INFINITY;
            let apply_min = |value: f64| min = value.min(min);
            if let Err(e) = model.apply_ifs(args, criteria, cell, true, apply_min) {
                return e;
            }
            CalcResult::Number(if min.is_infinite() { 0.0 } else { min })
        })
    }

    pub(crate) fn fn_maxifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_ifs(args, cell, |model, criteria| {
            let mut max = -f64::INFINITY;
            let apply_max = |value: f64| max = value.max(max);
            if let Err(e) = model.apply_ifs(args, criteria, cell, true, apply_max) {
                return e;
            }
            CalcResult::Number(if max.is_infinite() { 0.0 } else { max })
        })
    }
}
