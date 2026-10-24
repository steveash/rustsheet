//! Calculation settings of a [`Workbook`].

use rustsheet_engine::types::Iteration;

use super::Workbook;

impl Workbook {
    /// The iterative-calculation settings (`calcPr@iterate`), `None` when iteration is off.
    #[must_use]
    pub fn iteration(&self) -> Option<Iteration> {
        self.model.workbook.settings.iteration
    }

    /// Turns iterative calculation on with `iteration`'s count and delta, or off with `None`.
    /// It applies from the next recalculation (docs/excel-semantics/recalc.md).
    pub fn set_iteration(&mut self, iteration: Option<Iteration>) {
        self.model.workbook.settings.iteration = iteration;
    }
}
