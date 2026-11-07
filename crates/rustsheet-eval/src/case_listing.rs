//! `rustsheet-eval cases <suite>`: one suite's cases as the harness discovers them, as JSON, so
//! tools never re-implement discovery (the oracle fill reads it). Moved out of main.rs.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::suite::{FetchState, Suite, SuiteKind};
use crate::xlsx_snapshot::read_snapshot;

/// One line of `rustsheet-eval cases`.
#[derive(serde::Serialize)]
struct CaseListing {
    id: String,
    path: PathBuf,
    sidecar: PathBuf,
    /// xlsx only: number of formula cells (0 = nothing to score).
    formula_cells: Option<usize>,
    /// xlsx only: the suite scores this case against its own cached values when it has no sidecar.
    trusts_cached_values: Option<bool>,
    /// xlsx only: the suite's `use_cached_values`. When false its caches are no Excel evidence, so
    /// the oracle fill never lets one replace a web number (decision 0073).
    suite_uses_cached_values: Option<bool>,
    /// xlsx only: why the workbook could not be read.
    unreadable: Option<String>,
}

/// The cases of the suite `name` under `root`, as pretty-printed JSON.
///
/// # Errors
/// An unknown or unfetched suite, or a discovery failure.
pub fn list(root: &Path, name: &str) -> Result<String> {
    let suite = Suite::load_named(root, name)?;
    if suite.fetch_state() == FetchState::NotFetched {
        bail!("suite {name} is not fetched (run `rustsheet-eval fetch --suite {name}`)");
    }
    let xlsx = suite.manifest.kind == SuiteKind::Xlsx;
    let mut out = Vec::new();
    for case in suite.cases()? {
        let mut listing = CaseListing {
            id: case.id,
            path: case.path,
            sidecar: case.sidecar,
            formula_cells: None,
            trusts_cached_values: None,
            suite_uses_cached_values: xlsx.then_some(suite.manifest.use_cached_values),
            unreadable: None,
        };
        if xlsx {
            let snapshot = std::fs::read(&listing.path)
                .with_context(|| listing.path.display().to_string())
                .and_then(|bytes| Ok(read_snapshot(&bytes)?));
            match snapshot {
                Ok(snap) => {
                    listing.formula_cells = Some(
                        snap.sheets
                            .iter()
                            .flat_map(|s| s.cells.values())
                            .filter(|c| c.formula.is_some())
                            .count(),
                    );
                    listing.trusts_cached_values = Some(suite.manifest.trusts_cached_values(&snap));
                }
                Err(e) => listing.unreadable = Some(e.to_string()),
            }
        }
        out.push(listing);
    }
    Ok(serde_json::to_string_pretty(&out)?)
}
