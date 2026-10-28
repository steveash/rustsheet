//! The report's out-of-scope section (decision 0027): out-of-scope catalog functions with the
//! cells the run checked for them, and each suite's unscored add-in cells (T-0462).

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

use crate::report::SuiteSummary;

/// An out-of-scope catalog function (decision 0027) and the cells the run checked for it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutOfScope {
    pub name: String,
    /// The catalog's reason for leaving it out.
    pub reason: String,
    /// Cells calling the function that the run checked (passing or not).
    pub cells: usize,
    /// Of those, the failing ones.
    pub failing: usize,
}

/// Appends the "Out of scope" section: the catalog functions in `out` and, before them, every
/// suite with add-in cells left unscored. Nothing when both are empty.
pub fn write_markdown(out: &[OutOfScope], suites: &[SuiteSummary], md: &mut String) {
    let add_in: Vec<&SuiteSummary> = suites.iter().filter(|s| s.skipped_add_in > 0).collect();
    if out.is_empty() && add_in.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "\n## Out of scope ({})\n\nCatalog functions whose results depend on systems outside the workbook (decision 0027). They are left out of every count above.\n",
        out.len()
    );
    // Add-in calls are not catalog functions: their cells are not scored at all (T-0462).
    if !add_in.is_empty() {
        let _ = writeln!(
            md,
            "Add-in calls (`_xll.`) are out of scope too, and their cells are not scored at all:\n"
        );
    }
    for s in add_in {
        let _ = writeln!(
            md,
            "- {}: {} cells call an add-in function (`_xll.`) and are not scored.",
            s.name, s.skipped_add_in
        );
    }
    if out.is_empty() {
        return;
    }
    let _ = writeln!(md);
    let _ = writeln!(md, "| function | cells checked | failing | reason |");
    let _ = writeln!(md, "|---|---|---|---|");
    for f in out {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} |",
            f.name,
            f.cells,
            f.failing,
            f.reason.replace('|', "\\|")
        );
    }
}
