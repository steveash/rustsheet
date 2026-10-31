//! A keyed lookup's lift continued from its head to its tail (SPEC-0020 Amendment 2, section 2;
//! T-3221): VLOOKUP, HLOOKUP or MATCH over a whole-column key, read as a head and a tail by
//! `tailed_array.rs`, charges and checks what its one lift over the whole key would.

use super::driver::{broadcast_shape, Kernel, LiftCost, LiftOptions, LiftedArg};
use crate::calc_result::CalcResult;
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::tailed_array::{KeyedLift, LiftRecord};

#[cfg(test)]
thread_local! {
    /// Tests count the lifted kernel runs and the visits charged up front, to compare a keyed
    /// lookup's head and tail lifts with its one lift (SPEC-0020 Amendment 2, section 2).
    pub(crate) static LIFT_TALLY: std::cell::Cell<(u64, u64)> =
        const { std::cell::Cell::new((0, 0)) };
}

/// Counts one lifted kernel run (tests only).
pub(super) fn tally_run() {
    #[cfg(test)]
    LIFT_TALLY.with(|tally| {
        let (runs, visits) = tally.get();
        tally.set((runs + 1, visits));
    });
}

/// Counts `visits` charged up front (tests only).
pub(super) fn tally_visits(visits: u64) {
    #[cfg(test)]
    LIFT_TALLY.with(|tally| {
        let (runs, charged) = tally.get();
        tally.set((runs, charged.saturating_add(visits)));
    });
    #[cfg(not(test))]
    let _ = visits;
}

impl Model {
    /// [`Self::lift_values`] for the keyed lookup whose arguments are `call` (VLOOKUP, HLOOKUP,
    /// MATCH): while its tailed argument's head or tail is evaluated, the two lifts charge, check
    /// and compute what its one lift over the logical key would (SPEC-0020 Amendment 2, section
    /// 2). The head lifts as usual and records its up-front visits, its last element's slots and
    /// that element's result. The tail's one element, when its slots equal the recorded ones,
    /// is the recorded result, as [`EqualRuns`](super::driver::Reuse::EqualRuns) reuses it, and
    /// nothing is charged; otherwise the per-call cap is checked on the head's visits and the
    /// tail's together, and only the tail's are charged. Any other shape lifts as usual.
    pub(crate) fn lift_values_keyed(
        &mut self,
        call: &[Node],
        args: Vec<CalcResult>,
        cost: LiftCost,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        let Some(keyed) = self.keyed_lift(call, cell) else {
            return self.lift_values(args, cost, options, cell, kernel);
        };
        if !args.iter().any(LiftedArg::is_grid) {
            return kernel(self, &args);
        }
        let lifted = match LiftedArg::read_all(self, args, cell) {
            Ok(lifted) => lifted,
            Err(error) => return error,
        };
        match keyed {
            KeyedLift::Head => {
                let last = broadcast_shape(&lifted, cell).ok().map(|(rows, columns)| {
                    let (row, column) = (rows.saturating_sub(1), columns.saturating_sub(1));
                    lifted
                        .iter()
                        .map(|arg| arg.element(row, column).cloned())
                        .collect()
                });
                let (result, visits) =
                    self.lift_grids_after(lifted, cost, options, cell, (kernel, None), 0);
                let element = match &result {
                    CalcResult::Array(grid) => grid.last().and_then(|row| row.last()),
                    _ => None,
                };
                if let (Some(last), Some(element)) = (last, element) {
                    let record = LiftRecord {
                        visits,
                        last,
                        result: element.clone(),
                    };
                    self.record_keyed_lift(call, cell, record);
                }
                result
            }
            KeyedLift::Tail(Some(record)) if lifted.iter().all(|arg| arg.dims() == (1, 1)) => {
                let merged = lifted
                    .iter()
                    .zip(&record.last)
                    .all(|(arg, last)| arg.element(0, 0) == last.as_ref());
                if merged && lifted.len() == record.last.len() {
                    return CalcResult::Array(vec![vec![record.result]]);
                }
                let prior = record.visits;
                self.lift_grids_after(lifted, cost, options, cell, (kernel, None), prior)
                    .0
            }
            KeyedLift::Tail(_) => self.lift_grids(lifted, cost, options, cell, kernel, None),
        }
    }
}
