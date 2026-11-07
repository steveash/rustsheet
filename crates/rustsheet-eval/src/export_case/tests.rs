#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests")]
//! Engine-export cases (T-4177): the build, the package edits, and the scoring of a case against
//! a sidecar of Excel's re-render, its refusal and a stale fill.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::case::{OraclePivot, ORACLE_FORMAT};
use crate::engine::{Engine, RustSheetEngine};
use crate::pivot_scoring::LayoutVerdict;
use crate::runner::{run_case, CaseResult, CaseStatus};
use crate::suite::{oracle_sidecar, CaseFile, Suite, SuiteManifest};
use crate::value::Value;
use crate::xlsx_snapshot::read_snapshot;

/// A source of Region and Sales on `Data`, a GETPIVOTDATA cell on `Calc`, and an API pivot on
/// `P` at A3; `patches` as given.
fn case_json(patches: &str) -> String {
    format!(
        r#"{{"format": "{EXPORT_CASE_FORMAT}", "tags": ["pivot"],
        "spec": {{"format": "rustsheet-case/v1", "sheets": [
          {{"name": "Data", "cells": {{"A1": "Region", "B1": "Sales", "A2": "East", "B2": 1,
            "A3": "West", "B3": 2, "A4": "East", "B4": 4}}}},
          {{"name": "P", "cells": {{}}}},
          {{"name": "Calc", "cells": {{"B1": "=GETPIVOTDATA(\"Sum of Sales\",P!$A$3,\"Region\",\"East\")"}}}}]}},
        "pivots": [{{"name": "Pv", "source": "Data!A1:B4", "location": "P!A3", "row": "Region",
          "value": "Sales"}}],
        "patches": [{patches}]}}"#
    )
}

/// A fresh directory holding the case file `json` as `a.export.json`.
fn case_dir(tag: &str, json: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("rs-export-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("a{EXPORT_CASE_EXT}"));
    std::fs::write(&path, json).unwrap();
    (dir, path)
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut text = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

const CACHE: &str = "xl/pivotCache/pivotCacheDefinition1.xml";

#[test]
fn an_api_pivot_is_exported_the_same_every_time() {
    let (dir, path) = case_dir("api", &case_json(""));
    let first = export(&path).unwrap();
    assert_eq!(first, export(&path).unwrap());
    let table = part(&first, "xl/pivotTables/pivotTable1.xml");
    assert!(table.contains(r#"<location ref="A3:B6""#), "{table}");
    assert!(table.contains("<rowItems count=\"3\">"), "{table}");
    // The API saves no refreshOnLoad.
    assert!(!part(&first, CACHE).contains("refreshOnLoad"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_patch_edits_every_part_it_names_and_must_apply() {
    let refresh = r#"{"part": "xl/pivotCache/pivotCacheDefinition*.xml",
        "find": "<pivotCacheDefinition xmlns=",
        "replace": "<pivotCacheDefinition refreshOnLoad=\"1\" xmlns="}"#;
    let (dir, path) = case_dir("patch", &case_json(refresh));
    let bytes = export(&path).unwrap();
    assert!(part(&bytes, CACHE).contains(r#"<pivotCacheDefinition refreshOnLoad="1" xmlns="#));
    let missing = r#"{"part": "xl/pivotCache/*.xml", "find": "no such text", "replace": ""}"#;
    std::fs::write(&path, case_json(missing)).unwrap();
    assert!(export(&path).unwrap_err().contains("holds no"));
    let unmatched = r#"{"part": "xl/none.xml", "find": "x", "replace": ""}"#;
    std::fs::write(&path, case_json(unmatched)).unwrap();
    assert!(export(&path).unwrap_err().contains("no part matches"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_malformed_case_is_refused() {
    let both = case_json("").replacen(r#""tags""#, r#""source": "x.xlsx", "tags""#, 1);
    let empty = r#"{"part": "a", "find": "", "replace": ""}"#;
    for (tag, json, want) in [
        (
            "format",
            case_json("").replacen("export-case/v1", "export-case/v0", 1),
            "format",
        ),
        ("both", both, "exactly one"),
        ("empty", case_json(empty), "non-empty"),
        (
            "unknown",
            case_json("").replacen(r#""tags""#, r#""tag""#, 1),
            "unknown field",
        ),
    ] {
        let (dir, path) = case_dir(tag, &json);
        let err = export(&path).unwrap_err();
        assert!(err.contains(want), "{tag}: {err}");
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn an_xlsx_source_is_loaded_recalculated_and_saved() {
    // A repository of its own: the source resolves against the ancestor holding eval/suites.
    let json =
        format!(r#"{{"format": "{EXPORT_CASE_FORMAT}", "source": "data/pivot_tables.xlsx"}}"#);
    let (dir, _) = case_dir("xlsx", "");
    let suite = dir.join("eval/suites/x");
    std::fs::create_dir_all(&suite).unwrap();
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../rustsheet-xlsx/tests/pivot_tables.xlsx"),
        dir.join("data/pivot_tables.xlsx"),
    )
    .unwrap();
    let path = suite.join(format!("a{EXPORT_CASE_EXT}"));
    std::fs::write(&path, json).unwrap();
    let snapshot = read_snapshot(&export(&path).unwrap()).unwrap();
    assert_eq!(snapshot.pivots.len(), 2);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_source_outside_the_repository_is_refused() {
    let (dir, _) = case_dir("escape", "");
    let suite = dir.join("eval/suites/x");
    std::fs::create_dir_all(&suite).unwrap();
    let path = suite.join(format!("a{EXPORT_CASE_EXT}"));
    let absolute = dir.join("data/b.xlsx").display().to_string();
    for source in [
        "../../../data/b.xlsx",
        "data/../../b.xlsx",
        absolute.as_str(),
    ] {
        let json = format!(r#"{{"format": "{EXPORT_CASE_FORMAT}", "source": "{source}"}}"#);
        std::fs::write(&path, json).unwrap();
        let err = export(&path).unwrap_err();
        assert!(err.contains("without `..`"), "{source}: {err}");
    }
    let _ = std::fs::remove_dir_all(dir);
}

/// Runs the case of [`case_json`] with no patch as a suite of kind `export`, against a sidecar
/// built from its own export by `sidecar` (its pivots and Calc's values given as the engine's).
fn run_with(tag: &str, sidecar: impl Fn(&[u8], &mut serde_json::Value)) -> CaseResult {
    let (dir, path) = case_dir(tag, &case_json(""));
    let bytes = export(&path).unwrap();
    let pivots: Vec<OraclePivot> = read_snapshot(&bytes)
        .unwrap()
        .pivots
        .into_iter()
        .map(|definition| OraclePivot {
            definition,
            cells: [("A3", "Row Labels"), ("A4", "East"), ("A6", "Grand Total")]
                .into_iter()
                .map(|(a1, t)| (a1.to_string(), Value::Text(t.into())))
                .chain([("B4".to_string(), Value::Number(5.0))])
                .collect(),
            text: std::collections::BTreeMap::new(),
        })
        .collect();
    let mut json = serde_json::json!({
        "format": ORACLE_FORMAT,
        "source": {"kind": "excel-graph", "input_sha256": sha256_hex(&bytes)},
        "sheets": {"Calc": {"B1": {"t": "n", "v": 5.0}}},
        "pivots": pivots,
    });
    sidecar(&bytes, &mut json);
    std::fs::write(oracle_sidecar(&path), json.to_string()).unwrap();
    let manifest: SuiteManifest =
        toml::from_str("name = \"x\"\nkind = \"export\"\nlicense = \"MIT\"\nroot = \".\"\n")
            .unwrap();
    let suite = Suite {
        manifest,
        dir: dir.clone(),
        cache_dir: PathBuf::new(),
    };
    let cases = suite.cases().unwrap();
    assert_eq!(cases.len(), 1);
    let case: &CaseFile = &cases[0];
    let engine: Arc<dyn Engine + Send> = Arc::new(RustSheetEngine);
    let result = run_case(&engine, &suite, case, Duration::from_secs(60));
    let _ = std::fs::remove_dir_all(dir);
    result
}

#[test]
fn an_export_case_scores_formulas_pivot_cells_and_lines() {
    let r = run_with("scored", |_, _| {});
    assert_eq!(r.status, CaseStatus::Pass, "{:?}", r.message);
    // Calc!B1 and the four pivot cells the sidecar gives.
    assert_eq!((r.checked, r.passed), (5, 5));
    assert_eq!(r.tags, ["pivot"]);
    assert!(r.pivot_layout.is_empty(), "{:?}", r.pivot_layout);
    assert!(r.message.is_none());
}

#[test]
fn lines_that_are_not_excel_are_reported() {
    let r = run_with("lines", |_, json| {
        json["pivots"][0]["row_items"] = serde_json::json!([{"x": [0]}]);
        json["pivots"][0]["cells"]["B4"] = serde_json::json!({"t": "n", "v": 6.0});
    });
    assert_eq!(r.status, CaseStatus::Fail);
    let layout = &r.pivot_layout;
    assert_eq!(layout.len(), 1);
    assert_eq!(layout[0].verdict, LayoutVerdict::LinesDiffer);
    assert_eq!(layout[0].engine_lines, Some((3, 1)));
}

#[test]
fn a_refusal_fails_the_case_and_a_changed_export_is_noted() {
    let r = run_with("refused", |_, json| {
        *json = serde_json::json!({
            "format": ORACLE_FORMAT,
            "source": {"kind": "excel-graph", "input_sha256": "0"},
            "sheets": {},
            "refused": "createSession -> 501: OpenWorkbookBlockedWorkbook",
        });
    });
    assert_eq!(r.status, CaseStatus::ExcelRefused);
    let message = r.message.unwrap();
    assert!(
        message.starts_with("excel-refused: createSession"),
        "{message}"
    );
    assert!(
        message.contains("the export changed since the oracle fill"),
        "{message}"
    );
}
