//! Taking a sheet out of a model and putting it back, so a caller can undo a delete.

use crate::{
    formula_rewrite::RewrittenFormula,
    model::Model,
    pivot::PivotTable,
    types::{Table, Worksheet},
};

/// A sheet taken out of a model by [`Model::take_sheet`], with the tables and pivots that were on
/// it.
#[derive(Debug, Clone)]
pub struct TakenSheet {
    /// The sheet.
    pub worksheet: Worksheet,
    /// Its tables, by name; [`Model::restore_sheet`] puts them back (SPEC-0016).
    tables: Vec<(String, Table)>,
    /// Its pivot tables; [`Model::restore_sheet`] puts them back (SPEC-0015, T-2086).
    pivots: Vec<PivotTable>,
    /// The other formulas whose references to its tables became `#REF!`; [`Model::restore_sheet`]
    /// puts their text back (T-2340).
    rewritten: Vec<RewrittenFormula>,
}

impl TakenSheet {
    /// The other formulas the delete rewrote, for an undo that does not restore the
    /// [`TakenSheet`] itself ([`crate::UserModel`]'s history).
    pub(crate) fn into_rewritten(self) -> Vec<RewrittenFormula> {
        self.rewritten
    }
}

impl Model {
    /// How many times [`Model::evaluate`] has run on this model. Lets a caller (the facade's
    /// edit transactions) pin that it recalculates once per batch of edits.
    #[must_use]
    pub fn evaluation_count(&self) -> u64 {
        self.evaluation.evaluations
    }

    /// Removes the sheet at `sheet_index` and hands it back, so a caller can undo the delete
    /// with [`Model::restore_sheet`]. Fails like [`Model::delete_sheet`].
    pub fn take_sheet(&mut self, sheet_index: u32) -> Result<TakenSheet, String> {
        if self.workbook.worksheets.len() == 1 {
            return Err("Cannot delete only sheet".to_string());
        }
        if sheet_index as usize >= self.workbook.worksheets.len() {
            return Err("Sheet index too large".to_string());
        }
        let pivots = self.take_pivots_on(sheet_index)?;
        let (tables, rewritten) = self.take_tables_of_sheet(sheet_index);
        let worksheet = self.workbook.worksheets.remove(sheet_index as usize);
        self.reset_tables();
        Ok(TakenSheet {
            worksheet,
            tables,
            pivots,
            rewritten,
        })
    }

    /// Puts a sheet obtained from [`Model::take_sheet`] back at `sheet_index`.
    /// Fails if the index is out of range or the name/id is already in use.
    pub fn restore_sheet(&mut self, sheet_index: u32, sheet: TakenSheet) -> Result<(), String> {
        let TakenSheet {
            worksheet: sheet,
            tables,
            pivots,
            rewritten,
        } = sheet;
        if sheet_index as usize > self.workbook.worksheets.len() {
            return Err("Sheet index out of range".to_string());
        }
        let name = sheet.get_name().to_uppercase();
        if self
            .workbook
            .worksheets
            .iter()
            .any(|w| w.get_name().to_uppercase() == name || w.sheet_id == sheet.sheet_id)
        {
            return Err("A worksheet already exists with that name".to_string());
        }
        self.workbook.worksheets.insert(sheet_index as usize, sheet);
        self.restore_rewritten_formulas(rewritten);
        self.restore_tables(tables);
        self.restore_pivots(pivots);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Model;

    fn model() -> Model {
        let mut model = Model::new_empty("book", "en", "UTC", "en").expect("model");
        model.add_sheet("Two").expect("sheet");
        model
            .set_user_input(0, 1, 1, "=Two!A1+1".to_string())
            .expect("input");
        model
            .set_user_input(1, 1, 1, "41".to_string())
            .expect("input");
        model.evaluate();
        model
    }

    #[test]
    fn take_then_restore_round_trips() {
        let mut model = model();
        let sheet = model.take_sheet(1).expect("take");
        assert_eq!(model.workbook.worksheets.len(), 1);
        model.restore_sheet(1, sheet).expect("restore");
        model.evaluate();
        assert_eq!(model.workbook.worksheets.len(), 2);
        assert_eq!(
            model.get_formatted_cell_value(0, 1, 1).expect("value"),
            "42"
        );
    }

    #[test]
    fn take_sheet_rejects_the_only_sheet_and_bad_indexes() {
        let mut model = model();
        assert_eq!(model.take_sheet(5).unwrap_err(), "Sheet index too large");
        model.take_sheet(1).expect("take");
        assert_eq!(model.take_sheet(0).unwrap_err(), "Cannot delete only sheet");
    }

    #[test]
    fn restore_sheet_rejects_bad_index_and_duplicates() {
        let mut model = model();
        let sheet = model.take_sheet(1).expect("take");
        let copy = sheet.clone();
        assert_eq!(
            model.restore_sheet(9, sheet.clone()).unwrap_err(),
            "Sheet index out of range"
        );
        model.restore_sheet(1, sheet).expect("restore");
        assert_eq!(
            model.restore_sheet(1, copy).unwrap_err(),
            "A worksheet already exists with that name"
        );
    }

    #[test]
    fn evaluation_count_counts_evaluate_calls() {
        let mut model = model();
        let before = model.evaluation_count();
        model.evaluate();
        model.evaluate();
        assert_eq!(model.evaluation_count(), before + 2);
    }
}
