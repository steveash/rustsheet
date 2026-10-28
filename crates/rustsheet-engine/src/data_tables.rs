//! What-if data tables (docs/excel-semantics/recalc.md, "Data tables (`TABLE()`) are recalculated").
//!
//! A table's result cells hold values, not formulas. After a pass has evaluated the workbook,
//! each table is recomputed in storage order: for every result cell the input cell (or the two
//! input cells) is replaced by the values from the table's header row and column, the table's
//! formula is evaluated on demand, and the inputs are put back. The caller then evaluates the
//! workbook again, so the cells that read the results see them. A table nothing it read has
//! changed for since its last recomputation keeps its results (`data_table_memo.rs`, T-0453).

use crate::{
    calc_result::CalcResult,
    data_table_memo::names_memoizable,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    model::Model,
    types::{Cell, DataTable, DataTableError, DataTableInput},
};

/// Formula evaluations all the data tables of one recalculation may make together (CLAUDE.md
/// directive 5). A table over a large model re-evaluates the model once per result cell; past
/// the budget a result is `#NUM!` (recalc.md, DEVIATION).
const MAX_DATA_TABLE_EVALUATIONS: usize = 1 << 25;

/// Result cells all the data tables of one recalculation may fill together (directive 5). Each
/// table is bounded on its own (`MAX_DATA_TABLE_CELLS`), but a sheet may hold many. A table past
/// this budget is left as it is (recalc.md, DEVIATION).
const MAX_DATA_TABLE_RESULTS: u64 = 1 << 22;

impl Model {
    /// Recomputes every data table, sheet by sheet in the order the sheets store them, except
    /// those whose results cannot have changed. Returns whether any was recomputed, in which case
    /// the readers of their results must be evaluated again.
    pub(crate) fn evaluate_data_tables(&mut self) -> bool {
        let tables: Vec<(u32, DataTable)> = (self.workbook.worksheets.iter().enumerate())
            .flat_map(|(sheet, ws)| {
                ws.data_tables
                    .iter()
                    .map(move |t| (sheet as u32, t.clone()))
            })
            .collect();
        // With circular references to iterate, every evaluation runs another round of the
        // cycles, which the tables' readers must see.
        let memoizable = !self.has_cycles() && names_memoizable(&self.workbook.defined_names);
        let context = self.data_table_context();
        let mut budget = MAX_DATA_TABLE_EVALUATIONS;
        let mut results_left = MAX_DATA_TABLE_RESULTS;
        let mut recomputed = false;
        for (index, (sheet, table)) in tables.iter().enumerate() {
            let Some(left) = results_left.checked_sub(table.cell_count()) else {
                self.forget_data_table_memo(index);
                continue;
            };
            results_left = left;
            if memoizable {
                if let Some(cost) = self.data_table_unchanged(&context, index, (*sheet, table)) {
                    budget = budget.saturating_sub(cost);
                    continue;
                }
            }
            recomputed = true;
            self.evaluation.data_tables_recomputed += 1;
            let before = budget;
            let spills = self.start_data_table_reads();
            let results = self.evaluate_data_table(*sheet, table, &mut budget);
            // A table that ran out of budget wrote `#NUM!` for what the tables before it spent.
            let memoize = memoizable && budget > 0;
            let at = (index, *sheet, table);
            self.finish_data_table_reads(&context, at, (before - budget, &spills), memoize);
            for (position, value) in &results {
                self.write_constant(*position, value);
            }
            self.seal_data_table_memo(index, *sheet, table);
        }
        self.truncate_data_table_memo(tables.len());
        recomputed || (!tables.is_empty() && !memoizable)
    }

    /// The results of one table, by position.
    fn evaluate_data_table(
        &mut self,
        sheet: u32,
        table: &DataTable,
        budget: &mut usize,
    ) -> Vec<(CellReferenceIndex, ArrayNode)> {
        let (top, left, bottom, right) = table.range;
        let at = |row, column| CellReferenceIndex { sheet, row, column };
        // The header values as the workbook has them now: an earlier table's substitutions are
        // undone, so its readers are evaluated afresh.
        self.reset_on_demand_evaluation();
        let row_values: Vec<ArrayNode> = (left..=right)
            .map(|c| self.scalar_at(at(top - 1, c)))
            .collect();
        let column_values: Vec<ArrayNode> = (top..=bottom)
            .map(|r| self.scalar_at(at(r, left - 1)))
            .collect();
        let mut results = Vec::new();
        for (i, row) in (top..=bottom).enumerate() {
            for (j, column) in (left..=right).enumerate() {
                use DataTableInput::{Cell as Input, Deleted};
                let value = match (table.row_input, table.column_input) {
                    (Some(Deleted), _) | (_, Some(Deleted)) | (None, None) => {
                        ArrayNode::Error(Error::REF)
                    }
                    _ if *budget == 0 => ArrayNode::Error(Error::NUM),
                    (Some(Input(rr, rc)), Some(Input(cr, cc))) => {
                        let inputs = [
                            (at(rr, rc), row_values[j].clone()),
                            (at(cr, cc), column_values[i].clone()),
                        ];
                        self.evaluate_substituted(at(top - 1, left - 1), &inputs, budget)
                    }
                    (Some(Input(r, c)), None) => {
                        let inputs = [(at(r, c), row_values[j].clone())];
                        self.evaluate_substituted(at(row, left - 1), &inputs, budget)
                    }
                    (None, Some(Input(r, c))) => {
                        let inputs = [(at(r, c), column_values[i].clone())];
                        self.evaluate_substituted(at(top - 1, column), &inputs, budget)
                    }
                };
                results.push((at(row, column), value));
            }
        }
        results
    }

    /// The value of `formula` with each input cell holding its value, the inputs restored after.
    fn evaluate_substituted(
        &mut self,
        formula: CellReferenceIndex,
        inputs: &[(CellReferenceIndex, ArrayNode)],
        budget: &mut usize,
    ) -> ArrayNode {
        let saved: Vec<(CellReferenceIndex, Option<Cell>)> = inputs
            .iter()
            .map(|(at, _)| (*at, self.fetch_cell(*at).cloned()))
            .collect();
        for (at, value) in inputs {
            self.write_constant(*at, value);
        }
        self.reset_on_demand_evaluation();
        let value = self.scalar_at(formula);
        *budget = budget.saturating_sub(self.evaluation.cells.len().max(1));
        for (at, cell) in saved.into_iter().rev() {
            if let Ok(ws) = self.workbook.worksheet_mut(at.sheet) {
                match cell {
                    Some(cell) => ws.sheet_data.set_cell(at.row, at.column, cell),
                    None => ws.sheet_data.remove_cell(at.row, at.column),
                };
            }
        }
        value
    }

    /// Forgets which formulas have been evaluated, so the next read evaluates them on demand.
    fn reset_on_demand_evaluation(&mut self) {
        let state = &mut self.evaluation;
        state.cells.clear();
        state.clear_pass_caches();
        state.stack.clear();
        state.seen.clear();
        state.seen_ranges.clear();
        state.circular.clear();
        state.circular_anchors.clear();
        state.circular_below = 0;
        state.blocking = false;
        state.blocked_on = None;
        state.restart = None;
        state.root = None;
        state.depth = 0;
        state.unwinding = false;
        state.in_pass = false;
        self.clear_variable_stack();
        self.clear_lambdas();
    }

    /// The single value at a position, evaluating it if it is a formula; a blank formula
    /// result reads as 0, as in a cell.
    fn scalar_at(&mut self, at: CellReferenceIndex) -> ArrayNode {
        let value = match self.evaluate_cell(at) {
            CalcResult::Range { left, .. } => self.evaluate_cell(left),
            value => value,
        };
        match value {
            CalcResult::Number(n) => ArrayNode::Number(n),
            CalcResult::Boolean(b) => ArrayNode::Boolean(b),
            CalcResult::String(s) => ArrayNode::String(s),
            CalcResult::Error { error, .. } => ArrayNode::Error(error),
            CalcResult::Array(rows) => rows
                .into_iter()
                .next()
                .and_then(|row| row.into_iter().next())
                .unwrap_or(ArrayNode::Empty),
            CalcResult::EmptyCell | CalcResult::EmptyArg => {
                if matches!(self.fetch_cell(at), None | Some(Cell::EmptyCell { .. })) {
                    ArrayNode::Empty
                } else {
                    ArrayNode::Number(0.0)
                }
            }
            CalcResult::Range { .. } | CalcResult::Lambda(_) => ArrayNode::Error(Error::VALUE),
        }
    }

    /// Stores `value` as a constant at `at`, keeping the cell's style.
    fn write_constant(&mut self, at: CellReferenceIndex, value: &ArrayNode) {
        let s = self.fetch_cell(at).map_or(0, Cell::get_style);
        let cell = match value {
            ArrayNode::Number(v) => Cell::NumberCell { v: *v, s },
            ArrayNode::Boolean(v) => Cell::BooleanCell { v: *v, s },
            ArrayNode::Error(ei) => Cell::ErrorCell { ei: ei.clone(), s },
            // A lambda a cell would hold shows `#CALC!`, as a formula cell's does (`model.rs`).
            ArrayNode::Lambda(_) => Cell::ErrorCell { ei: Error::CALC, s },
            // A nested GROUPBY cell shows `#VALUE!` (SPEC-0018 rule 2).
            ArrayNode::Nested(_) => Cell::ErrorCell {
                ei: Error::VALUE,
                s,
            },
            ArrayNode::Empty => Cell::EmptyCell { s },
            ArrayNode::String(text) => {
                let _ =
                    self.set_cell_with_string(at.sheet, at.row, at.column, &text.to_string(), s);
                return;
            }
        };
        if let Ok(ws) = self.workbook.worksheet_mut(at.sheet) {
            ws.sheet_data.set_cell(at.row, at.column, cell);
        }
    }

    /// Adds a data table to sheet `sheet` ([`DataTable::check`] says which tables it accepts).
    /// Its result cells are emptied of content; the table fills them at the next evaluation.
    ///
    /// # Errors
    /// The rule the table breaks, or [`DataTableError::NoSheet`].
    pub fn add_data_table(&mut self, sheet: u32, table: DataTable) -> Result<(), DataTableError> {
        let ws = self
            .workbook
            .worksheet_mut(sheet)
            .map_err(|_| DataTableError::NoSheet)?;
        table.check(&ws.data_tables)?;
        let (top, left, bottom, right) = table.range;
        for row in top..=bottom {
            for column in left..=right {
                if let Some(s) = ws.sheet_data.cell(row, column).map(Cell::get_style) {
                    ws.sheet_data.set_cell(row, column, Cell::EmptyCell { s });
                }
            }
        }
        ws.data_tables.push(table);
        Ok(())
    }

    /// Removes the data table whose results are `range` from sheet `sheet`; its result cells
    /// keep their values. Returns whether there was one.
    pub fn remove_data_table(&mut self, sheet: u32, range: (i32, i32, i32, i32)) -> bool {
        let Ok(ws) = self.workbook.worksheet_mut(sheet) else {
            return false;
        };
        let before = ws.data_tables.len();
        ws.data_tables.retain(|t| t.range != range);
        ws.data_tables.len() != before
    }

    /// The data table whose result range holds `at`, if any.
    pub(crate) fn data_table_at(&self, at: CellReferenceIndex) -> Option<&DataTable> {
        let ws = self.workbook.worksheet(at.sheet).ok()?;
        ws.data_tables.iter().find(|t| {
            let (top, left, bottom, right) = t.range;
            (top..=bottom).contains(&at.row) && (left..=right).contains(&at.column)
        })
    }
}
