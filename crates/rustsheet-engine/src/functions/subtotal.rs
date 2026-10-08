use crate::arithmetic::flush_subnormal;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{parse_range, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    reference_operators::{is_multi_area, Ranges},
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
    fn get_table_for_cell(&self, sheet_index: u32, row: i32, column: i32) -> bool {
        let worksheet = match self.workbook.worksheet(sheet_index) {
            Ok(ws) => ws,
            Err(_) => return false,
        };
        for table in self.workbook.tables.values() {
            if worksheet.name != table.sheet_name {
                continue;
            }
            // (column, row, column, row)
            if let Ok((column1, row1, column2, row2)) = parse_range(&table.reference) {
                if ((column >= column1) && (column <= column2)) && ((row >= row1) && (row <= row2))
                {
                    return table.has_filters;
                }
            }
        }
        false
    }

    fn cell_hidden_status(
        &self,
        sheet_index: u32,
        row: i32,
        column: i32,
    ) -> Result<CellTableStatus, String> {
        let worksheet = self.workbook.worksheet(sheet_index)?;
        let mut hidden = false;
        for row_style in &worksheet.rows {
            if row_style.r == row {
                hidden = row_style.hidden;
                break;
            }
        }
        if !hidden {
            return Ok(CellTableStatus::Normal);
        }
        // The row is hidden we need to know if the table has filters
        if self.get_table_for_cell(sheet_index, row, column) {
            Ok(CellTableStatus::Filtered)
        } else {
            Ok(CellTableStatus::Hidden)
        }
    }

    fn cell_is_subtotal(&self, sheet_index: u32, row: i32, column: i32) -> bool {
        self.cell_formula_is_one_of(sheet_index, row, column, &[Function::Subtotal])
    }

    /// Whether the cell holds a formula whose outermost call is one of `kinds` (SUBTOTAL and
    /// AGGREGATE skip such cells).
    pub(crate) fn cell_formula_is_one_of(
        &self,
        sheet_index: u32,
        row: i32,
        column: i32,
        kinds: &[Function],
    ) -> bool {
        let cell = match self.workbook.worksheets[sheet_index as usize].cell(row, column) {
            Some(c) => c,
            None => {
                return false;
            }
        };

        match cell.get_formula() {
            Some(f) => {
                let node: &Node = &self.parsed_formulas[sheet_index as usize][f as usize].0;
                // `=(SUBTOTAL(...))` is a SUBTOTAL formula too, as it was before grouped roots.
                let node = match node {
                    Node::GroupedRootKind { child } => child,
                    other => other,
                };
                matches!(node, Node::FunctionKind { kind, .. } if kinds.contains(kind))
            }
            None => false,
        }
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
        // The row last found not hidden: a row's cells are consecutive, and the status lookup
        // scans the sheet's row styles.
        let mut visible_row: Option<(u32, i32)> = None;
        self.for_each_position_in_areas(areas, cell, |model, position| {
            if visible_row != Some((position.sheet, position.row)) {
                let status = model
                    .cell_hidden_status(position.sheet, position.row, position.column)
                    .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
                if status == CellTableStatus::Filtered
                    || (*mode == SubTotalMode::SkipHidden && status == CellTableStatus::Hidden)
                {
                    return Ok(());
                }
                if status == CellTableStatus::Normal {
                    visible_row = Some((position.sheet, position.row));
                }
            }
            if model.cell_is_subtotal(position.sheet, position.row, position.column) {
                return Ok(());
            }
            f(model.evaluate_cell(position))
        })
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

    fn subtotal_get_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut result: Vec<f64> = Vec::new();
        for arg in args {
            match arg {
                Node::FunctionKind {
                    kind: Function::Subtotal,
                    args: _,
                } => {
                    // skip
                }
                // A union walks its areas one after the other, each counted as written
                // (probes/multiarea-subtotal: SUBTOTAL(9,(A1,B2)) = 6, T-1071).
                _ if is_multi_area(arg) => {
                    let ranges = self.reference_ranges(arg, cell)?;
                    self.subtotal_walk_range(&ranges, cell, &mode, &mut result)?;
                }
                _ => {
                    match self.evaluate_node_with_reference(arg, cell) {
                        CalcResult::String(_) | CalcResult::Boolean(_) => {
                            // Skip
                        }
                        CalcResult::Number(f) => result.push(f),
                        error @ CalcResult::Error { .. } => {
                            return Err(error);
                        }
                        CalcResult::Range { left, right } => {
                            self.subtotal_walk_range(
                                &[Range { left, right }],
                                cell,
                                &mode,
                                &mut result,
                            )?;
                        }
                        CalcResult::EmptyCell | CalcResult::EmptyArg => result.push(0.0),
                        CalcResult::Array(_) | CalcResult::Lambda(_) => {
                            return Err(CalcResult::Error {
                                error: Error::NIMPL,
                                origin: cell,
                                message: "Arrays not supported yet".to_string(),
                            })
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    pub(crate) fn fn_subtotal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        match value {
            1 => self.subtotal_average(&args[1..], cell, SubTotalMode::Full),
            2 => self.subtotal_count(&args[1..], cell, SubTotalMode::Full),
            3 => self.subtotal_counta(&args[1..], cell, SubTotalMode::Full),
            4 => self.subtotal_max(&args[1..], cell, SubTotalMode::Full),
            5 => self.subtotal_min(&args[1..], cell, SubTotalMode::Full),
            6 => self.subtotal_product(&args[1..], cell, SubTotalMode::Full),
            7 => self.subtotal_stdevs(&args[1..], cell, SubTotalMode::Full),
            8 => self.subtotal_stdevp(&args[1..], cell, SubTotalMode::Full),
            9 => self.subtotal_sum(&args[1..], cell, SubTotalMode::Full),
            10 => self.subtotal_vars(&args[1..], cell, SubTotalMode::Full),
            11 => self.subtotal_varp(&args[1..], cell, SubTotalMode::Full),
            101 => self.subtotal_average(&args[1..], cell, SubTotalMode::SkipHidden),
            102 => self.subtotal_count(&args[1..], cell, SubTotalMode::SkipHidden),
            103 => self.subtotal_counta(&args[1..], cell, SubTotalMode::SkipHidden),
            104 => self.subtotal_max(&args[1..], cell, SubTotalMode::SkipHidden),
            105 => self.subtotal_min(&args[1..], cell, SubTotalMode::SkipHidden),
            106 => self.subtotal_product(&args[1..], cell, SubTotalMode::SkipHidden),
            107 => self.subtotal_stdevs(&args[1..], cell, SubTotalMode::SkipHidden),
            108 => self.subtotal_stdevp(&args[1..], cell, SubTotalMode::SkipHidden),
            109 => self.subtotal_sum(&args[1..], cell, SubTotalMode::SkipHidden),
            110 => self.subtotal_vars(&args[1..], cell, SubTotalMode::Full),
            111 => self.subtotal_varp(&args[1..], cell, SubTotalMode::Full),
            _ => CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Invalid value for SUBTOTAL: {value}"),
            ),
        }
    }

    fn subtotal_vars(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64 - 1.0)
        }

        CalcResult::Number(result)
    }

    fn subtotal_varp(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64)
        }
        CalcResult::Number(result)
    }

    fn subtotal_stdevs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64 - 1.0)
        }

        CalcResult::Number(result.sqrt())
    }

    fn subtotal_stdevp(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64)
        }
        CalcResult::Number(result.sqrt())
    }

    fn subtotal_counta(
        &mut self,
        args: &[Node],
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
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let mut count = 0;
        self.subtotal_count_with(args, cell, &mode, &mut count, |value| {
            matches!(value, CalcResult::Number(_))
        })
    }

    /// COUNT and COUNTA of the arguments: `counted` says which values a reference's cells (and a
    /// direct argument) add to `count`. A union is walked area by area like a range.
    fn subtotal_count_with(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: &SubTotalMode,
        count: &mut u32,
        counted: fn(&CalcResult) -> bool,
    ) -> CalcResult {
        for arg in args {
            let areas = match arg {
                // skip
                Node::FunctionKind {
                    kind: Function::Subtotal,
                    args: _,
                } => continue,
                _ if is_multi_area(arg) => match self.reference_ranges(arg, cell) {
                    Ok(ranges) => ranges,
                    Err(error) => return error,
                },
                _ => match self.evaluate_node_with_reference(arg, cell) {
                    CalcResult::Range { left, right } => Ranges::One(Range { left, right }),
                    // A direct argument counts as itself (an error only for COUNTA).
                    CalcResult::Array(_) | CalcResult::Lambda(_) => {
                        return CalcResult::Error {
                            error: Error::NIMPL,
                            origin: cell,
                            message: "Arrays not supported yet".to_string(),
                        }
                    }
                    value => {
                        if counted(&value) {
                            *count += 1;
                        }
                        continue;
                    }
                },
            };
            let walked = self.subtotal_walk_areas(&areas, cell, mode, |value| {
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
        args: &[Node],
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
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        for value in values {
            result += value;
        }
        CalcResult::finite_number_or_num(result, cell)
    }

    fn subtotal_product(
        &mut self,
        args: &[Node],
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
        args: &[Node],
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
        args: &[Node],
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
