use crate::arithmetic::flush_subnormal;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::{
        math_and_trigonometry::SumParts,
        statistical::variance::{variance_result, VarianceKind},
        Function,
    },
    model::Model,
    reference_operators::{not_a_reference_array, Ranges, RefArg},
};

/// Excel has a complicated way of filtering + hidden rows
/// As a first a approximation a table can either have filtered rows or hidden rows, but not both.
/// Internally hey both will be marked as hidden rows. Hidden rows
/// The behaviour is the same for SUBTOTAL(100s,) => ignore those
/// But changes for the SUBTOTAL(1-11, ) those ignore filtered but take hidden into account.
/// In Excel filters are non-dynamic. Once you apply filters in a table (say value in column 1 should > 20) they
/// stay that way, even if you change the value of the values in the table after the fact.
/// If you try to hide rows in a table with filtered rows they will behave as if filtered
/// // Also subtotals ignore subtotals
///
#[derive(PartialEq)]
enum SubTotalMode {
    Full,
    SkipHidden,
}

#[derive(PartialEq, Debug)]
pub enum CellTableStatus {
    Normal,
    Hidden,
    Filtered,
}

impl Model {
    /// Whether the sheet has a table with filters. Excel (probes/subtotal-hidden-filtered, T-1182)
    /// calls every hidden row of such a sheet filtered, also above or below the table and in
    /// columns outside it; see docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden
    /// and filtered rows".
    fn sheet_has_filtered_table(&self, sheet_index: u32) -> bool {
        let Ok(worksheet) = self.workbook.worksheet(sheet_index) else {
            return false;
        };
        self.workbook
            .tables
            .values()
            .any(|table| table.has_filters && table.sheet_name == worksheet.name)
    }

    /// `sheet_filtered` is [`Self::sheet_has_filtered_table`] of the sheet, decided once per area walk.
    fn row_hidden_status(
        &self,
        sheet_index: u32,
        row: i32,
        sheet_filtered: bool,
    ) -> Result<CellTableStatus, String> {
        let worksheet = self.workbook.worksheet(sheet_index)?;
        let hidden = worksheet
            .rows
            .iter()
            .find(|row_style| row_style.r == row)
            .is_some_and(|row_style| row_style.hidden);
        Ok(match (hidden, sheet_filtered) {
            (false, _) => CellTableStatus::Normal,
            (true, true) => CellTableStatus::Filtered,
            (true, false) => CellTableStatus::Hidden,
        })
    }

    fn cell_is_subtotal(&self, sheet_index: u32, row: i32, column: i32) -> bool {
        self.cell_formula_is_one_of(
            sheet_index,
            row,
            column,
            &[Function::Subtotal, Function::Aggregate],
        )
    }

    /// Whether the cell holds a formula that calls one of `kinds` anywhere (SUBTOTAL and
    /// AGGREGATE skip such cells; probes/subtotal-hidden-filtered Nest sheet, T-1036).
    pub(crate) fn cell_formula_is_one_of(
        &self,
        sheet_index: u32,
        row: i32,
        column: i32,
        kinds: &[Function],
    ) -> bool {
        let Some(cell) = self.workbook.worksheets[sheet_index as usize].cell(row, column) else {
            return false;
        };
        cell.get_formula().is_some_and(|f| {
            self.parsed_formulas[sheet_index as usize][f as usize]
                .0
                .contains_function(kinds)
        })
    }

    /// Calls `f` with the value of every cell of `areas` that SUBTOTAL reads: filtered rows,
    /// hidden rows in `SkipHidden` mode and cells holding a SUBTOTAL are skipped unevaluated.
    /// The one position walk behind every SUBTOTAL function number, for a range and a union alike
    /// (the areas are walked as [`Self::for_each_position_in_areas`] orders them).
    fn subtotal_walk_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mode: &SubTotalMode,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(CalcResult) -> Result<(), CalcResult>,
    {
        if areas.iter().any(|area| area.left.sheet != area.right.sheet) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Ranges are in different sheets".to_string(),
            ));
        }
        let of_union = areas.len() > 1;
        for area in areas {
            // A row is skipped or kept whole, whatever its column (unevidenced per column, see
            // T-1036); whether its hide counts as a filter is decided per sheet, once per area
            // (the areas are on one sheet, checked above). The row last seen is cached: its cells
            // are consecutive and the lookup scans the sheet's row styles.
            let mut last_row: Option<(u32, i32, bool)> = None;
            let sheet_filtered = self.sheet_has_filtered_table(area.left.sheet);
            // A single area is clipped to the used area, and a union's areas are walked over
            // their existing cells: the cells left out are empty, which no function number reads.
            self.for_each_position_in_area(
                area,
                of_union,
                cell,
                &mut |model: &mut Self, position: CellReferenceIndex| {
                    let skip = match last_row {
                        Some((sheet, row, skip))
                            if (sheet, row) == (position.sheet, position.row) =>
                        {
                            skip
                        }
                        _ => {
                            let status = model
                                .row_hidden_status(position.sheet, position.row, sheet_filtered)
                                .map_err(|message| {
                                    CalcResult::new_error(Error::ERROR, cell, message)
                                })?;
                            let skip = status == CellTableStatus::Filtered
                                || (*mode == SubTotalMode::SkipHidden
                                    && status == CellTableStatus::Hidden);
                            last_row = Some((position.sheet, position.row, skip));
                            skip
                        }
                    };
                    if skip || model.cell_is_subtotal(position.sheet, position.row, position.column)
                    {
                        return Ok(());
                    }
                    f(model.evaluate_cell(position))
                },
            )?;
        }
        Ok(())
    }

    /// Pushes the numbers of `areas` as [`Self::subtotal_walk_areas`] reads them; text and
    /// booleans are ignored, an error cell is the result.
    fn subtotal_walk_range(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mode: &SubTotalMode,
        result: &mut Vec<f64>,
    ) -> Result<(), CalcResult> {
        self.subtotal_walk_areas(areas, cell, mode, |value| match value {
            CalcResult::Number(value) => {
                result.push(value);
                Ok(())
            }
            error @ CalcResult::Error { .. } => Err(error),
            // We ignore booleans and strings
            _ => Ok(()),
        })
    }

    /// The numbers of `args` (already resolved by [`Self::subtotal_reference_args`]) as
    /// [`Self::subtotal_walk_areas`] reads them.
    fn subtotal_get_values(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut result: Vec<f64> = Vec::new();
        for areas in args {
            self.subtotal_walk_range(areas, cell, &mode, &mut result)?;
        }
        Ok(result)
    }

    /// The areas of every SUBTOTAL argument after the function number, resolved before any cell
    /// is read. The parser refuses an argument that is not written as a reference, as Excel
    /// does; one that evaluates to something else (`IF(TRUE,5)`, `LAMBDA(x,x)`) is `#VALUE!`
    /// and an error is itself, whatever the other arguments hold: `SUBTOTAL(9,C1,IF(TRUE,5))`
    /// is `#VALUE!` with C1 = `#DIV/0!`. An array is mapped element by element and spills
    /// (`SUBTOTAL(9,IF(TRUE,{1,#N/A}))` is `#VALUE!`, `#N/A`). Scratch probes on excel-graph,
    /// docs/excel-semantics/references.md "SUBTOTAL and AREAS take references only" (T-1163).
    fn subtotal_reference_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<Ranges>, CalcResult> {
        let mut resolved = Vec::with_capacity(args.len());
        for arg in args {
            // A union walks its areas one after the other, each counted as written
            // (probes/multiarea-subtotal: SUBTOTAL(9,(A1,B2)) = 6, T-1071).
            match self.evaluate_picked_reference_arg(arg, cell) {
                RefArg::Ranges(ranges) => resolved.push(ranges),
                RefArg::Value(error @ CalcResult::Error { .. }) => return Err(error),
                // A cached range of another workbook reads as an array here; Excel sums it as a
                // reference (fixtures/external-link-cached-ranges Refs!B59 is 15). Until that is
                // matched (TODO(T-1526)), it is one `#VALUE!`, not an array that would spill.
                RefArg::Value(CalcResult::Array(_))
                    if self.external_reference(arg, cell).is_some() =>
                {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "SUBTOTAL of a cached external range".to_string(),
                    ))
                }
                RefArg::Value(CalcResult::Array(rows)) => return Err(not_a_reference_array(rows)),
                RefArg::Value(_) => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "SUBTOTAL takes references".to_string(),
                    ))
                }
            }
        }
        Ok(resolved)
    }

    pub(crate) fn fn_subtotal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // The arguments are read first: an error or an array there beats an invalid function
        // number, which is `#VALUE!` only next to arguments that resolve to references
        // (`SUBTOTAL(99,#REF!)` is `#REF!`, `SUBTOTAL(12,IF(TRUE,1/0))` is `#DIV/0!`;
        // probes/subtotal-reference-operands-b H5:H10, T-1175).
        let invalid = || {
            CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Invalid value for SUBTOTAL: {value}"),
            )
        };
        let resolved = match self.subtotal_reference_args(&args[1..], cell) {
            Ok(resolved) => resolved,
            Err(error) => return error,
        };
        if !matches!(value, 1..=11 | 101..=111) {
            return invalid();
        }
        let args = resolved.as_slice();
        match value {
            1 => self.subtotal_average(args, cell, SubTotalMode::Full),
            2 => self.subtotal_count(args, cell, SubTotalMode::Full),
            3 => self.subtotal_counta(args, cell, SubTotalMode::Full),
            4 => self.subtotal_max(args, cell, SubTotalMode::Full),
            5 => self.subtotal_min(args, cell, SubTotalMode::Full),
            6 => self.subtotal_product(args, cell, SubTotalMode::Full),
            7 => self.subtotal_stdevs(args, cell, SubTotalMode::Full),
            8 => self.subtotal_stdevp(args, cell, SubTotalMode::Full),
            9 => self.subtotal_sum(args, cell, SubTotalMode::Full),
            10 => self.subtotal_vars(args, cell, SubTotalMode::Full),
            11 => self.subtotal_varp(args, cell, SubTotalMode::Full),
            101 => self.subtotal_average(args, cell, SubTotalMode::SkipHidden),
            102 => self.subtotal_count(args, cell, SubTotalMode::SkipHidden),
            103 => self.subtotal_counta(args, cell, SubTotalMode::SkipHidden),
            104 => self.subtotal_max(args, cell, SubTotalMode::SkipHidden),
            105 => self.subtotal_min(args, cell, SubTotalMode::SkipHidden),
            106 => self.subtotal_product(args, cell, SubTotalMode::SkipHidden),
            107 => self.subtotal_stdevs(args, cell, SubTotalMode::SkipHidden),
            108 => self.subtotal_stdevp(args, cell, SubTotalMode::SkipHidden),
            109 => self.subtotal_sum(args, cell, SubTotalMode::SkipHidden),
            110 => self.subtotal_vars(args, cell, SubTotalMode::SkipHidden),
            111 => self.subtotal_varp(args, cell, SubTotalMode::SkipHidden),
            _ => invalid(),
        }
    }

    fn subtotal_vars(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        match self.subtotal_get_values(args, cell, mode) {
            Ok(values) => variance_result(&values, VarianceKind::Sample, false, "SUBTOTAL", cell),
            Err(error) => error,
        }
    }

    fn subtotal_varp(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        match self.subtotal_get_values(args, cell, mode) {
            Ok(values) => {
                variance_result(&values, VarianceKind::Population, false, "SUBTOTAL", cell)
            }
            Err(error) => error,
        }
    }

    fn subtotal_stdevs(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        match self.subtotal_get_values(args, cell, mode) {
            Ok(values) => variance_result(&values, VarianceKind::Sample, true, "SUBTOTAL", cell),
            Err(error) => error,
        }
    }

    fn subtotal_stdevp(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        match self.subtotal_get_values(args, cell, mode) {
            Ok(values) => {
                variance_result(&values, VarianceKind::Population, true, "SUBTOTAL", cell)
            }
            Err(error) => error,
        }
    }

    fn subtotal_counta(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let mut counta = 0;
        self.subtotal_count_with(args, cell, &mode, &mut counta, |value| {
            !matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg)
        })
    }

    fn subtotal_count(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let mut count = 0;
        self.subtotal_count_with(args, cell, &mode, &mut count, |value| {
            matches!(value, CalcResult::Number(_))
        })
    }

    /// COUNT and COUNTA of the resolved arguments: `counted` says which cell values add to
    /// `count`. A union is walked area by area like a range.
    fn subtotal_count_with(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: &SubTotalMode,
        count: &mut u32,
        counted: fn(&CalcResult) -> bool,
    ) -> CalcResult {
        for areas in args {
            let walked = self.subtotal_walk_areas(areas, cell, mode, |value| {
                if counted(&value) {
                    *count += 1;
                }
                Ok(())
            });
            if let Err(error) = walked {
                return error;
            }
        }
        CalcResult::Number(f64::from(*count))
    }

    fn subtotal_average(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in values {
            result += value;
        }
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        CalcResult::finite_number_or_num(flush_subnormal(result / (l as f64)), cell)
    }

    fn subtotal_sum(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        // SUBTOTAL(9/109) snaps a near-zero final addition like SUM does
        // (probes/near-zero-aggregate-forms B3-B5).
        let mut sum = SumParts::default();
        for value in values {
            sum.add(value);
        }
        CalcResult::finite_number_or_num(sum.snapped(), cell)
    }

    fn subtotal_product(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 1.0;
        for value in values {
            result *= value;
        }
        CalcResult::finite_number_or_num(result, cell)
    }

    fn subtotal_max(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = f64::NAN;
        for value in values {
            result = value.max(result);
        }
        if result.is_nan() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    fn subtotal_min(
        &mut self,
        args: &[Ranges],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = f64::NAN;
        for value in values {
            result = value.min(result);
        }
        if result.is_nan() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }
}
