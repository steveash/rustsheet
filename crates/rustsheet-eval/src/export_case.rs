//! Engine-export cases (T-4177; docs/research/pivot-tables-2026-11-04/plan.md §6 layer 4).
//!
//! An export case (`<name>.export.json`) names a workbook, a spec case or an xlsx file of the
//! repository or an inline spec, pivots to add to it through the API (`Txn::add_pivot_table`), and text edits of
//! the saved package. The harness builds the workbook as the cell suites do, recalculates it (a
//! `refreshOnLoad` pivot renders, as on open), adds the pivots, saves it with the engine's xlsx
//! export and applies the edits: [`export`]. That package is what the oracle opens
//! (`rustsheet-eval export`, which the oracle fill calls) and what the case scores: the engine
//! loads it as Excel does, and its formula cells, pivot areas and pivot `rowItems`/`colItems` are
//! compared with Excel's re-render of the same file (the sidecar's `pivots`). A sidecar with
//! `refused` records that Excel would not open the file, SPEC-0033's `excel-refused` verdict.
//!
//! The case is built from the current engine on every run. The sidecar's `input_sha256` names
//! the package Excel opened, so a run notes a case whose export has changed since its fill.
//! The edits exist because the API cannot express everything a case needs: a `refreshOnLoad`
//! cache (the API saves none), or a file without attributes the export now writes.

use std::io::{Cursor, Read as _, Write as _};
use std::path::{Component, Path, PathBuf};

use globset::Glob;
use rustsheet::{LoadOptions, PivotFunction, PivotTableInput, SaveOptions, Workbook};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use crate::case::{split_qualified, CaseSpec, PivotGroupSpec, CASE_FORMAT};
use crate::engine::{build_spec, harness_options, range_ref};
use crate::spec_pivots::range_group;

/// The `format` of an export case.
pub const EXPORT_CASE_FORMAT: &str = "rustsheet-export-case/v1";

/// The file name suffix of an export case.
pub const EXPORT_CASE_EXT: &str = ".export.json";

/// The most pivots and package edits one case may list: the case files are committed, but the
/// limits keep a mistaken one bounded (directive 5). A patched package is read under the
/// public API's per-package limit, as the fetch reads an archive.
const MAX_PIVOTS: usize = 16;
const MAX_PATCHES: usize = 64;

/// An export case file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportCase {
    /// [`EXPORT_CASE_FORMAT`].
    pub format: String,
    #[serde(default)]
    pub description: String,
    /// Copied onto the case result, as a spec case's are.
    #[serde(default)]
    pub tags: Vec<String>,
    /// The workbook to load: a `.case.json` spec case or an `.xlsx` file, by a path relative to
    /// the repository root, without `..`. Exactly one of `source` and `spec` is given.
    #[serde(default)]
    pub source: Option<String>,
    /// An inline spec case, built as the cell suites build one.
    #[serde(default)]
    pub spec: Option<Box<CaseSpec>>,
    /// Pivots added through the API after the load, in order.
    #[serde(default)]
    pub pivots: Vec<ApiPivot>,
    /// Text edits of the saved package, in order.
    #[serde(default)]
    pub patches: Vec<Patch>,
}

/// A pivot added with `Txn::add_pivot_table` ([`PivotTableInput`]).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiPivot {
    pub name: String,
    /// The source range, sheet-qualified: `Data!A1:C9`.
    pub source: String,
    /// The anchor, sheet-qualified: `P!A3`.
    pub location: String,
    pub row: String,
    #[serde(default)]
    pub column: Option<String>,
    pub value: String,
    /// `sum` when absent; the names of `PivotFunction::from_name`.
    #[serde(default)]
    pub function: Option<String>,
    /// A range or date grouping of the row field, as a spec pivot's `group`.
    #[serde(default)]
    pub group: Option<PivotGroupSpec>,
}

/// Replaces every `find` with `replace` in each package part whose name matches the glob
/// `part`. A part that matches without holding `find`, or an edit that matches no part, fails
/// the case, so an export that changes under an edit is noticed.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub part: String,
    pub find: String,
    pub replace: String,
}

/// Reads and checks the export case at `path`.
///
/// # Errors
/// An unreadable or malformed case, or one past the limits.
pub fn read(path: &Path) -> Result<ExportCase, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let case: ExportCase =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if case.format != EXPORT_CASE_FORMAT {
        return Err(format!("unsupported export case format {:?}", case.format));
    }
    if case.source.is_some() == case.spec.is_some() {
        return Err("an export case gives exactly one of `source` and `spec`".into());
    }
    if case.pivots.len() > MAX_PIVOTS || case.patches.len() > MAX_PATCHES {
        return Err(format!(
            "at most {MAX_PIVOTS} pivots and {MAX_PATCHES} patches"
        ));
    }
    if case.patches.iter().any(|p| p.find.is_empty()) {
        return Err("a patch needs a non-empty `find`".into());
    }
    Ok(case)
}

/// The package the export case at `path` saves: see the module documentation.
///
/// # Errors
/// A malformed case or source, the engine refusing a pivot or the save, or an edit that does
/// not apply.
pub fn export(path: &Path) -> Result<Vec<u8>, String> {
    let case = read(path)?;
    let mut workbook = match (&case.source, &case.spec) {
        (Some(source), None) => load_source(&source_path(path, source)?)?,
        (None, Some(spec)) => build_spec(spec, None)?,
        _ => return Err("an export case gives exactly one of `source` and `spec`".into()),
    };
    for pivot in &case.pivots {
        let input = api_input(&workbook, pivot)?;
        workbook
            .edit(|tx| tx.add_pivot_table(&input))
            .map_err(|e| format!("pivot {}: {e}", pivot.name))?;
    }
    let bytes = (workbook.to_xlsx_bytes(SaveOptions::default())).map_err(|e| e.to_string())?;
    apply_patches(&bytes, &case.patches)
}

/// The hex sha256 of `bytes`, as the oracle fill names the package it sent
/// (`source.input_sha256`).
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    crate::fetch::hex(&Sha256::digest(bytes))
}

/// The file a case's `source` names: a relative path without `..`, resolved against the
/// repository holding the case (the nearest ancestor of the case file with `eval/suites`), so a
/// case reads only committed files beside the suites and the crates' test workbooks.
///
/// # Errors
/// An absolute `source`, one with `..` or a root, or a case outside any repository.
fn source_path(case: &Path, source: &str) -> Result<PathBuf, String> {
    if !Path::new(source)
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(format!(
            "source {source:?} must be a relative path inside the repository, without `..`"
        ));
    }
    let case = std::fs::canonicalize(case).map_err(|e| format!("{}: {e}", case.display()))?;
    let root = (case.ancestors().skip(1))
        .find(|dir| dir.join("eval/suites").is_dir())
        .ok_or_else(|| format!("{}: no eval/suites above the case", case.display()))?;
    Ok(root.join(source))
}

/// A spec case built, or an xlsx file loaded and recalculated, as the cell suites evaluate it.
fn load_source(path: &Path) -> Result<Workbook, String> {
    let name = path.to_string_lossy();
    if name.ends_with(".case.json") {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
        let spec: CaseSpec = serde_json::from_str(&text).map_err(|e| format!("{name}: {e}"))?;
        if spec.format != CASE_FORMAT {
            return Err(format!("{name}: unsupported case format {:?}", spec.format));
        }
        return build_spec(&spec, None);
    }
    if !name.ends_with(".xlsx") {
        return Err(format!("{name}: a source is a .case.json or an .xlsx"));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{name}: {e}"))?;
    let opts = LoadOptions {
        recalc: false,
        workbook: harness_options(None)?,
        ..LoadOptions::default()
    };
    let mut workbook =
        Workbook::from_xlsx_bytes(&bytes, &opts).map_err(|e| format!("{name}: {e}"))?;
    workbook.pin_clock_from_metadata();
    workbook.recalculate();
    Ok(workbook)
}

fn api_input(workbook: &Workbook, pivot: &ApiPivot) -> Result<PivotTableInput, String> {
    let range = |qualified: &str| {
        let (sheet, a1) = split_qualified(qualified)
            .ok_or_else(|| format!("pivot {}: {qualified} names no sheet", pivot.name))?;
        let id = workbook.sheet_id(&sheet).map_err(|e| e.to_string())?;
        range_ref(id, &a1).map_err(|e| format!("pivot {}: {qualified}: {e}", pivot.name))
    };
    let name = pivot.function.as_deref().unwrap_or("sum");
    let function = PivotFunction::from_name(name)
        .ok_or_else(|| format!("pivot {}: function {name:?}", pivot.name))?;
    let mut input = PivotTableInput::new(
        &pivot.name,
        range(&pivot.source)?,
        range(&pivot.location)?,
        &pivot.row,
        &pivot.value,
        function,
    );
    input.column.clone_from(&pivot.column);
    input.group = (pivot.group.as_ref().map(range_group).transpose()).map_err(|e| e.to_string())?;
    Ok(input)
}

/// `bytes` with `patches` applied, rezipped; `bytes` itself when there are none.
fn apply_patches(bytes: &[u8], patches: &[Patch]) -> Result<Vec<u8>, String> {
    if patches.is_empty() {
        return Ok(bytes.to_vec());
    }
    let globs = patches
        .iter()
        .map(|p| Glob::new(&p.part).map(|g| g.compile_matcher()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut used = vec![false; patches.len()];
    let mut left = LoadOptions::default().max_uncompressed_bytes;
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().map_err(|e| e.to_string())?.to_string();
        let mut data = Vec::new();
        let read = (entry.take(left.saturating_add(1)).read_to_end(&mut data))
            .map_err(|e| format!("{name}: {e}"))?;
        left = (left.checked_sub(read as u64)).ok_or("the package passes the size limit")?;
        for ((patch, glob), used) in patches.iter().zip(&globs).zip(&mut used) {
            if !glob.is_match(&name) {
                continue;
            }
            let text = String::from_utf8(data).map_err(|e| format!("{name}: {e}"))?;
            if !text.contains(&patch.find) {
                return Err(format!(
                    "patch of {}: {name} holds no {:?}",
                    patch.part, patch.find
                ));
            }
            data = text.replace(&patch.find, &patch.replace).into_bytes();
            *used = true;
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        out.write_all(&data).map_err(|e| e.to_string())?;
    }
    if let Some((patch, _)) = patches.iter().zip(&used).find(|(_, used)| !**used) {
        return Err(format!("patch of {}: no part matches it", patch.part));
    }
    Ok(out.finish().map_err(|e| e.to_string())?.into_inner())
}

#[cfg(test)]
mod tests;
