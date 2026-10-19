use crate::arithmetic::{extended_quotient, extended_sum, flush_subnormal};
use crate::cast::number_text_too_long;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::criteria::build_criteria;
use crate::functions::lift::{LiftCost, LiftOptions, LiftedArg};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::Node,
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

const TOO_LONG: &str = "Criterion too long";

fn criterion_too_long(criterion: &CalcResult) -> bool {
    matches!(criterion, CalcResult::String(text) if number_text_too_long(text))
}

/// The ranges of a SUMIFS-shaped call: the (resized) sum range and the criteria ranges.
struct SumRanges {
    sum_range: Range,
    ranges: Vec<Range>,
}

fn shape_mismatch(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "Ranges must have the same shape".to_string(),
    )
}

/// How the ranges of a SUMIFS-shaped call relate to the sum range.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IfsShape {
    /// Every range must be shaped like the sum range, else `#VALUE!` (SUMIFS, AVERAGEIFS, ...).
    Strict,
    /// The sum range is resized to the criteria range, never an error (SUMIF, AVERAGEIF).
    Resized,
    /// [`IfsShape::Resized`] on the 2-argument form, where `args[0]` and `args[1]` are the one
    /// range argument: Excel evaluates it once (probes/sumif-averageif-two-arg-range-once).
    ResizedSelf,
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

    /// Runs an *IF(S) function once per criteria element through [`Model::lift_values`]: when
    /// any criterion is an array or a multi-cell range, the result is an array of the criteria
    /// broadcast against each other (a position past a shorter array uses an `#N/A`
    /// criterion). Otherwise `compute` runs once with the criteria as they are.
    ///
    /// `walked` is the range every element walks, as written and evaluated by reference (the value
    /// range, or COUNTIFS's first criteria range): the call is `#ERROR!` when its used cells
    /// ([`Model::range_walk_cost`], measured only when the call lifts) times the criteria count
    /// times the elements to compute exceed `limits::MAX_LIFTED_CELL_VISITS`.
    /// `compute` reads only its criteria and the ranges resolved before the lift, so every
    /// repeated criteria element (the blank tail of a whole-column criteria range, the
    /// occurrence-count idiom `COUNTIF(A2:A10000,A2:A10000)`) reuses its first result (T-0408).
    fn lift_criteria(
        &mut self,
        criteria: Vec<CalcResult>,
        walked: &CalcResult,
        cell: CellReferenceIndex,
        mut compute: impl FnMut(&mut Self, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        // An unlifted call runs once and is not charged: measuring would clip the walked range
        // to the used area on every plain SUMIFS.
        let walk_cost = if criteria.iter().any(LiftedArg::is_grid) {
            self.range_walk_cost(walked)
        } else {
            1
        };
        // Each element walks the range once per criterion.
        let walks = u64::try_from(criteria.len()).unwrap_or(u64::MAX);
        self.lift_values(
            criteria,
            LiftCost::Cells(walk_cost.saturating_mul(walks)),
            &LiftOptions::REUSE_ANY_EQUAL,
            cell,
            &mut compute,
        )
    }

    /// Cells of `node`'s range inside the used area (at least 1): what one criteria walk visits.
    pub(crate) fn walk_cost(&mut self, node: &Node, cell: CellReferenceIndex) -> u64 {
        let value = self.evaluate_node_with_reference(node, cell);
        self.range_walk_cost(&value)
    }

    /// Cells of `value`'s range inside the used area, 1 for a value that is not a range.
    fn range_walk_cost(&mut self, value: &CalcResult) -> u64 {
        let CalcResult::Range { left, right } = *value else {
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
    /// argument count, resolves the ranges once ([`Model::resolve_sum_ranges`]), then lifts
    /// `compute` over the criteria (see [`Model::lift_criteria`]). `compute` gets the resolved
    /// ranges, or the error every element is.
    fn lift_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        shape: IfsShape,
        mut compute: impl FnMut(&mut Self, &Result<SumRanges, CalcResult>, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        if args.len() < 3 || args.len().is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }
        let criteria = self.criteria_values(args, 2, cell);
        // By reference: a single-cell sum_range (`A1`) is a 1x1 range to resize from.
        let sum_value = self.evaluate_node_with_reference(&args[0], cell);
        let ranges = self.resolve_sum_ranges(args, sum_value.clone(), cell, shape);
        self.lift_criteria(criteria, &sum_value, cell, |model, criteria| {
            compute(model, &ranges, criteria)
        })
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
        let (arguments, shape) = match args.len() {
            2 => (
                vec![args[0].clone(), args[0].clone(), args[1].clone()],
                IfsShape::ResizedSelf,
            ),
            // Unlike AVERAGEIFS, a differently shaped average_range is not an
            // error: it is resized to the criteria range (probes/ifs-shape-mismatch F9–F11).
            3 => (
                vec![args[2].clone(), args[0].clone(), args[1].clone()],
                IfsShape::Resized,
            ),
            _ => return CalcResult::new_args_number_error(cell),
        };
        self.average_ifs(&arguments, cell, shape)
    }

    pub(crate) fn fn_countifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || !args.len().is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }
        let criteria = self.criteria_values(args, 1, cell);
        // Excel: any criteria range shaped unlike the first is #VALUE! (T-0209).
        let first = self.evaluate_node_with_reference(&args[0], cell);
        let ranges = self
            .resolve_criteria_ranges(args, 0, first.clone(), cell)
            .and_then(|ranges| {
                if same_shape(&ranges) {
                    Ok(ranges)
                } else {
                    Err(shape_mismatch(cell))
                }
            });
        self.lift_criteria(criteria, &first, cell, |model, criteria| match &ranges {
            Ok(ranges) => model.count_ifs(ranges, criteria, cell),
            Err(error) => error.clone(),
        })
    }

    /// An *IF(S) range argument already evaluated by reference (a single-cell range `A1` is a 1x1
    /// range, probes/sumif-edge-single-cell F11-F15): an error as it is, a cross-sheet range or a
    /// non-range `#VALUE!`.
    fn ifs_range(value: CalcResult, cell: CellReferenceIndex) -> Result<Range, CalcResult> {
        match value {
            CalcResult::Range { left, right } if left.sheet != right.sheet => {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Ranges are in different sheets".to_string(),
                ))
            }
            CalcResult::Range { left, right } => Ok(Range { left, right }),
            error if error.is_error() => Err(error),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected a range".to_string(),
            )),
        }
    }

    /// The criteria ranges `args[first]`, `args[first + 2]`, ... of COUNTIFS and the
    /// SUMIFS-shaped functions, each evaluated once, before the lift (T-0408): the first bad one
    /// is the error, and the ranges after it are not evaluated. `first_value` is `args[first]`
    /// already evaluated by reference.
    fn resolve_criteria_ranges(
        &mut self,
        args: &[Node],
        first: usize,
        first_value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        let mut ranges = vec![Self::ifs_range(first_value, cell)?];
        for node in args.iter().skip(first + 2).step_by(2) {
            let value = self.evaluate_node_with_reference(node, cell);
            ranges.push(Self::ifs_range(value, cell)?);
        }
        Ok(ranges)
    }

    /// The ranges of the SUMIFS-shaped functions, resolved once before the lift (T-0408);
    /// `sum_value` is `args[0]` evaluated by reference. Every range must be shaped like the sum
    /// range, else `#VALUE!` (T-0209; SUMIFS from gen-m2, AVERAGEIFS/MINIFS/MAXIFS from
    /// probes/ifs-shape-mismatch). Only SUMIF's and AVERAGEIF's single-criterion path
    /// (`shape` not `Strict`) opts out: it never errors (T-0214), and walks a sum range resized
    /// to the shape of the criteria range from its top-left cell (gen-m2/04cfd82f54dbc2b2
    /// Calc!A344, gen-m2/629e6e8b76fab8b9 Calc!A217; probes/ifs-shape-mismatch F9-F11).
    fn resolve_sum_ranges(
        &mut self,
        args: &[Node],
        sum_value: CalcResult,
        cell: CellReferenceIndex,
        shape: IfsShape,
    ) -> Result<SumRanges, CalcResult> {
        let shared = (shape == IfsShape::ResizedSelf).then(|| sum_value.clone());
        Self::ifs_range(sum_value, cell).and_then(|sum_range| {
            let first = shared.unwrap_or_else(|| self.evaluate_node_with_reference(&args[1], cell));
            let ranges = self.resolve_criteria_ranges(args, 1, first, cell)?;
            if shape == IfsShape::Strict {
                let sum_shape = range_shape(&sum_range);
                if ranges.iter().any(|range| range_shape(range) != sum_shape) {
                    return Err(shape_mismatch(cell));
                }
                Ok(SumRanges { sum_range, ranges })
            } else {
                let sum_range = resize_to(&sum_range, &ranges[0]);
                Ok(SumRanges { sum_range, ranges })
            }
        })
    }

    /// Builds the predicates of `criteria` (one per criteria range, resolved before the lift).
    /// A criterion over 255 characters is `#VALUE!` (from a literal, a cell or one element of a
    /// range), but only once the ranges and shapes resolved: a bad range wins
    /// (probes/long-text-args-min-criteria A10-A26, probes/ifs-long-criterion-precedence).
    fn build_ifs_criteria<'c>(
        &self,
        criteria: &'c [CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<Vec<Criterion<'c>>, CalcResult> {
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
        Ok(fn_criteria)
    }

    /// COUNTIFS over its resolved criteria ranges, with the criteria already evaluated
    /// (`criteria[k]` is the criterion of `ranges[k]`).
    fn count_ifs(
        &mut self,
        ranges: &[Range],
        criteria: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let case_count = criteria.len();
        let fn_criteria = match self.build_ifs_criteria(criteria, cell) {
            Ok(fn_criteria) => fn_criteria,
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

    /// The walk of the SUMIFS-shaped functions over their resolved ranges, with the criteria
    /// already evaluated (`criteria[k]` is the criterion of `ranges.ranges[k]`; see
    /// [`Model::lift_ifs`]).
    fn apply_ifs<F>(
        &mut self,
        ranges: &Result<SumRanges, CalcResult>,
        criteria: &[CalcResult],
        cell: CellReferenceIndex,
        apply: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        let SumRanges { sum_range, ranges } = ranges.as_ref().map_err(Clone::clone)?;
        let fn_criteria = self.build_ifs_criteria(criteria, cell)?;
        self.run_ifs(sum_range, ranges, &fn_criteria, cell, apply)
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
        sum_range: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arguments = [
            sum_range.unwrap_or(criteria_range).clone(),
            criteria_range.clone(),
            criteria.clone(),
        ];
        let shape = if sum_range.is_some() {
            IfsShape::Resized
        } else {
            IfsShape::ResizedSelf
        };
        self.sum_ifs(&arguments, cell, shape)
    }

    /// SUMIFS (`IfsShape::Strict`) and SUMIF on `(sum_range, criteria_range, criteria, ...)`, lifted
    /// over array criteria; a sum beyond the double range is `#NUM!` (T-0991).
    pub(crate) fn sum_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        shape: IfsShape,
    ) -> CalcResult {
        self.lift_ifs(args, cell, shape, |model, ranges, criteria| {
            let mut total = 0.0;
            // Each addition rounds twice like SUM's (probes-numeric/double-rounding-accumulators).
            match model.apply_ifs(ranges, criteria, cell, |v| {
                total = extended_sum(total, v);
            }) {
                Ok(()) => CalcResult::finite_number_or_num(total, cell),
                Err(e) => e,
            }
        })
    }

    pub(crate) fn fn_averageifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.average_ifs(args, cell, IfsShape::Strict)
    }

    /// AVERAGEIFS, lifted over array criteria; AVERAGEIF passes `Resized` or `ResizedSelf`.
    fn average_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        shape: IfsShape,
    ) -> CalcResult {
        self.lift_ifs(args, cell, shape, |model, ranges, criteria| {
            let mut total = 0.0;
            let mut count = 0.0;
            let average = |value: f64| {
                total = extended_sum(total, value);
                count += 1.0;
            };
            if let Err(e) = model.apply_ifs(ranges, criteria, cell, average) {
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
        self.lift_ifs(args, cell, IfsShape::Strict, |model, ranges, criteria| {
            let mut min = f64::INFINITY;
            let apply_min = |value: f64| min = value.min(min);
            if let Err(e) = model.apply_ifs(ranges, criteria, cell, apply_min) {
                return e;
            }
            CalcResult::Number(if min.is_infinite() { 0.0 } else { min })
        })
    }

    pub(crate) fn fn_maxifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_ifs(args, cell, IfsShape::Strict, |model, ranges, criteria| {
            let mut max = -f64::INFINITY;
            let apply_max = |value: f64| max = value.max(max);
            if let Err(e) = model.apply_ifs(ranges, criteria, cell, apply_max) {
                return e;
            }
            CalcResult::Number(if max.is_infinite() { 0.0 } else { max })
        })
    }
}
