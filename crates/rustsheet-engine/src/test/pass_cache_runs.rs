#![allow(clippy::unwrap_used, reason = "tests")]

//! A workbook evaluated with the pass caches on and off (`Evaluation::no_pass_caches`), for the
//! tests that a cache gives what the walk gives: every cell's stored value to the bit, its
//! circular mark, and the work units the recalculation spent.

use crate::expressions::types::CellReferenceIndex;
use crate::expressions::utils::column_to_number;
use crate::test::util::new_empty_model;
use crate::types::Iteration;
use crate::Model;

/// A cell to set: its name on Sheet1 and its input.
pub(crate) fn cell(name: String, value: &str) -> (String, String) {
    (name, value.to_string())
}

/// What [`cached_and_walked`] measured: `(name, cached, walked)` per cell set, with a last
/// `(work)` row; and what the probe read of the cached run's model and of the walked run's.
pub(crate) struct Runs {
    pub(crate) rows: Vec<(String, String, String)>,
    pub(crate) cached: usize,
    pub(crate) walked: usize,
}

/// Sets `cells` and evaluates, then sets `edits` and evaluates again (when there are any), with
/// the pass caches on and off, under iterative calculation when `iterate`; `probe` reads what the
/// caller counts from each finished model.
pub(crate) fn cached_and_walked(
    cells: &[(String, String)],
    edits: &[(String, String)],
    iterate: bool,
    probe: &dyn Fn(&Model) -> usize,
) -> Runs {
    let run = |no_pass_caches: bool| {
        let mut model = new_empty_model();
        if iterate {
            model.workbook.settings.iteration = Some(Iteration {
                max_iterations: 20,
                max_change: 0.001,
            });
        }
        model.evaluation.no_pass_caches = no_pass_caches;
        for (cell, value) in cells {
            model._set(cell, value);
        }
        model.evaluate();
        for (cell, value) in edits {
            model._set(cell, value);
        }
        if !edits.is_empty() {
            model.evaluate();
        }
        let mut values: Vec<String> = cells.iter().map(|(cell, _)| state(&model, cell)).collect();
        let spent = model
            .evaluation
            .last_recalc
            .as_ref()
            .map(|last| last.spent());
        values.push(format!("spent {spent:?}"));
        (values, probe(&model))
    };
    let (cached, cached_probe) = run(false);
    let (walked, walked_probe) = run(true);
    let names = cells
        .iter()
        .map(|(cell, _)| cell.clone())
        .chain(["(work)".to_string()]);
    Runs {
        rows: names
            .zip(cached.into_iter().zip(walked))
            .map(|(cell, (cached, walked))| (cell, cached, walked))
            .collect(),
        cached: cached_probe,
        walked: walked_probe,
    }
}

/// [`cached_and_walked`], asserting that both runs agree everywhere.
pub(crate) fn assert_alike(
    cells: &[(String, String)],
    edits: &[(String, String)],
    iterate: bool,
    probe: &dyn Fn(&Model) -> usize,
) -> Runs {
    let runs = cached_and_walked(cells, edits, iterate, probe);
    for (cell, cached, walked) in &runs.rows {
        assert_eq!(cached, walked, "{cell}");
    }
    runs
}

/// The stored value of Sheet1's `cell`, exactly, its text, and whether it is marked circular.
fn state(model: &Model, cell: &str) -> String {
    let split = cell.find(|c: char| c.is_ascii_digit()).unwrap();
    let at = CellReferenceIndex {
        sheet: 0,
        row: cell[split..].parse().unwrap(),
        column: column_to_number(&cell[..split]).unwrap(),
    };
    let circular = model
        .evaluation
        .circular
        .contains(&crate::evaluation::key(at));
    format!(
        "{:?} {} circular={circular}",
        model._get_cell(cell),
        model._get_text(cell)
    )
}
