use super::*;
use crate::case::ORACLE_FORMAT;
use crate::suite::{oracle_sidecar, SuiteManifest};

/// Engine whose behaviour is chosen by the spec's `description` ("panic", "error", else ok).
struct Scripted;

struct NoValues;
impl Evaluated for NoValues {
    fn value(&self, _: &str, _: Addr) -> Option<Value> {
        None
    }
    fn stored_cells(&self, _: &str) -> Vec<Addr> {
        Vec::new()
    }
}

impl Engine for Scripted {
    fn name(&self) -> &'static str {
        "scripted"
    }
    fn evaluate_xlsx(&self, _: &[u8]) -> Result<Box<dyn Evaluated>, String> {
        Err("unused".into())
    }
    fn evaluate_spec(&self, spec: &CaseSpec) -> Result<Box<dyn Evaluated>, String> {
        match spec.description.as_str() {
            "panic" => panic!("boom"),
            "error" => Err("cannot materialize".into()),
            _ => Ok(Box::new(NoValues)),
        }
    }
    fn implemented_functions(&self) -> Vec<String> {
        Vec::new()
    }
}

fn run_pending(description: &str) -> CaseResult {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-{description}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("a.case.json");
    let json = format!(
        r#"{{"format":"{CASE_FORMAT}","description":"{description}","sheets":[{{"name":"S","cells":{{"A1":"=1+1"}}}}]}}"#
    );
    std::fs::write(&path, json).expect("write case");
    let manifest: SuiteManifest =
        toml::from_str("name = \"t\"\nkind = \"spec\"\nlicense = \"MIT\"\n").expect("manifest");
    let suite = Suite {
        manifest,
        dir: dir.clone(),
        cache_dir: PathBuf::new(),
    };
    let case = CaseFile {
        id: "t/a".into(),
        sidecar: oracle_sidecar(&path),
        path,
    };
    let engine: Arc<dyn Engine + Send> = Arc::new(Scripted);
    let r = run_case(&engine, &suite, &case, Duration::from_secs(5));
    let _ = std::fs::remove_dir_all(dir);
    r
}

#[test]
fn pending_case_that_evaluates_cleanly_stays_pending() {
    assert_eq!(run_pending("ok").status, CaseStatus::Pending);
}

#[test]
fn pending_case_that_panics_is_reported_as_panic() {
    let r = run_pending("panic");
    assert_eq!(r.status, CaseStatus::Panic);
    assert_eq!(r.message.as_deref(), Some("boom"));
}

/// A `cached_overrides` line replaces exactly the named cached expectation with the golden
/// value it points at, and naming an unscored cell is an error (decision 0032).
#[test]
fn cached_override_replaces_the_named_expectation() {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-overrides", std::process::id()));
    let suite_dir = dir.join("calc");
    std::fs::create_dir_all(suite_dir.join("cases")).expect("mkdir");
    std::fs::create_dir_all(dir.join("probes")).expect("mkdir");
    std::fs::write(
        dir.join("probes/p.oracle.json"),
        format!(
            r#"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"excel-graph"}},
            "sheets":{{"Sheet1":{{"B1":{{"t":"n","v":30.0}}}}}}}}"#
        ),
    )
    .expect("write oracle");
    let manifest: SuiteManifest = toml::from_str(
        "name = \"calc\"\nkind = \"xlsx\"\nlicense = \"MIT\"\n\
         cached_overrides = \"o.tsv\"\n",
    )
    .expect("manifest");
    let suite = Suite {
        manifest,
        dir: suite_dir.clone(),
        cache_dir: PathBuf::new(),
    };
    let case = CaseFile {
        id: "calc/D/x.xlsx".into(),
        path: suite_dir.join("cases/D/x.xlsx"),
        sidecar: PathBuf::new(),
    };
    let cached = |a1: &str| Expectation {
        sheet: "S".into(),
        addr: Addr::parse(a1).expect("addr"),
        formula: None,
        anchor_formula: None,
        expected: Value::Error("#VALUE!".into()),
        from_excel: true,
        pinned: None,
        from_graph: false,
    };
    let write_overrides = |cell: &str| {
        let line = format!("D/x.xlsx\tS!{cell}\tprobes/p.oracle.json\tSheet1!B1\n");
        std::fs::write(suite_dir.join("o.tsv"), line).expect("write overrides");
    };
    write_overrides("C71");
    let got =
        apply_cached_overrides(&suite, &case, vec![cached("C70"), cached("C71")]).expect("apply");
    assert_eq!(got[0].expected, Value::Error("#VALUE!".into()));
    assert_eq!(got[1].expected, Value::Number(30.0));
    write_overrides("Z9");
    let unscored = apply_cached_overrides(&suite, &case, vec![cached("C70")]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(unscored.is_err_and(|e| e.contains("unscored cell S!Z9")));
}

/// An excel-graph 0 hides a kept subnormal (decision 0042); other sources and values do not.
#[test]
fn graph_zero_hides_only_a_subnormal() {
    let e = |expected: f64, from_graph| Expectation {
        sheet: "S".into(),
        addr: Addr::parse("A1").expect("addr"),
        formula: None,
        anchor_formula: None,
        expected: Value::Number(expected),
        from_excel: true,
        pinned: None,
        from_graph,
    };
    let subnormal = Value::Number(5e-309);
    assert!(graph_hides_subnormal(&e(0.0, true), &subnormal));
    assert!(!graph_hides_subnormal(&e(0.0, false), &subnormal));
    assert!(!graph_hides_subnormal(&e(1.0, true), &subnormal));
    assert!(!graph_hides_subnormal(
        &e(0.0, true),
        &Value::Number(1e-300)
    ));
    assert!(!graph_hides_subnormal(&e(0.0, true), &Value::Number(0.0)));
}

/// A listed cell is a deviation when it gives its pinned value, a pass when it matches Excel
/// and a failure otherwise; a deviation leaves the case passing, and a line naming an
/// unscored cell is an error (decision 0036).
#[test]
fn listed_deviations_score_apart_from_bit_exact() {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-deviations", std::process::id()));
    std::fs::create_dir_all(dir.join("cases")).expect("mkdir");
    let path = dir.join("cases/a.case.json");
    std::fs::write(
        &path,
        format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=1","A2":"=2","A3":"=3"}}}}],
               "expected":{{"S!A1":{{"t":"n","v":9}},"S!A2":{{"t":"n","v":2}},"S!A3":{{"t":"n","v":9}}}}}}"#
        ),
    )
    .expect("write case");
    let manifest: SuiteManifest = toml::from_str(
        "name = \"t\"\nkind = \"spec\"\nlicense = \"MIT\"\nroot = \"cases\"\n\
         deviations = \"deviations\"\n",
    )
    .expect("manifest");
    let suite = Suite {
        manifest,
        dir: dir.clone(),
        cache_dir: PathBuf::new(),
    };
    let case = CaseFile {
        id: "t/a.case.json".into(),
        sidecar: oracle_sidecar(&path),
        path,
    };
    let write = |cells: &[(&str, &str)]| {
        let lines: Vec<String> = cells
            .iter()
            .map(|(cell, pinned)| {
                format!(
                    "a.case.json\tS!{cell}\tdecision 0014\t\
                     docs/excel-semantics/arrays.md#x\t{pinned}"
                )
            })
            .collect();
        std::fs::write(dir.join("deviations"), lines.join("\n")).expect("write deviations");
    };
    let engine: Arc<dyn Engine + Send> = Arc::new(crate::engine::RustSheetEngine);
    let run = || run_case(&engine, &suite, &case, Duration::from_secs(60));
    write(&[("A1", "1"), ("A2", "5")]);
    let r = run();
    let verdicts: Vec<(Verdict, bool)> = r
        .cells
        .iter()
        .map(|c| (c.verdict, c.listed_deviation))
        .collect();
    assert_eq!(
        verdicts,
        [
            (Verdict::Deviation, true),
            (Verdict::Pass, true), // stale: A2 matches Excel
            (Verdict::NumericMismatch, false),
        ]
    );
    assert_eq!((r.checked, r.passed, r.deviations), (3, 1, 1));
    assert_eq!(r.status, CaseStatus::Fail);
    write(&[("A1", "1"), ("A3", "5")]);
    let r = run();
    assert_eq!(r.cells[2].verdict, Verdict::NumericMismatch); // 3 is neither 9 nor 5
    write(&[("A1", "1"), ("A3", "3")]);
    assert_eq!(run().status, CaseStatus::Pass);
    write(&[("Z9", "1")]);
    let unscored = run();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(unscored.status, CaseStatus::InvalidCase);
    assert!(unscored
        .message
        .is_some_and(|m| m.contains("unscored cell S!Z9")));
}

/// An oracle sidecar replaces the hand expectation for the same cell, however the hand key
/// was written; it is never scored next to it (smoke/d-function-criteria J2, 2026-10-04).
#[test]
fn oracle_value_replaces_hand_value_for_the_same_cell() {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-oracle-wins", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("a.case.json");
    let case = format!(
        r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=6","A2":"=7","B1":"=8"}}}},
           {{"name":"My S","cells":{{"A1":"=9"}}}}],
           "expected":{{"S!A1":{{"t":"n","v":6}},"'S'!$A$2":{{"t":"n","v":7}},"S!B1":{{"t":"n","v":8}},
                        "'My S'!A1":{{"t":"n","v":9}}}}}}"#
    );
    std::fs::write(&path, case).expect("write case");
    let oracle = format!(
        r#"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"excel-graph","application":"x"}},
           "sheets":{{"S":{{"A1":{{"t":"n","v":0}},"A2":{{"t":"n","v":1}}}},"My S":{{"A1":{{"t":"n","v":2}}}}}}}}"#
    );
    std::fs::write(dir.join("a.oracle.json"), oracle).expect("write oracle");
    let prepared = prepare_spec(&CaseFile {
        id: "t/a".into(),
        sidecar: oracle_sidecar(&path),
        path,
    });
    let _ = std::fs::remove_dir_all(&dir);
    let prepared = prepared.expect("prepare");
    let excel_flags: Vec<(String, String, bool)> = prepared
        .expectations
        .iter()
        .map(|e| (e.sheet.clone(), e.addr.to_string(), e.from_excel))
        .collect();
    let mut got: Vec<(String, String, Value)> = prepared
        .expectations
        .into_iter()
        .map(|e| (e.sheet, e.addr.to_string(), e.expected))
        .collect();
    got.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let excel: Vec<(String, String)> = excel_flags
        .into_iter()
        .filter(|(_, _, from_excel)| *from_excel)
        .map(|(sheet, cell, _)| (sheet, cell))
        .collect();
    // The hand value B1 is not Excel evidence although the sidecar covers its neighbours.
    assert_eq!(excel.len(), 3);
    assert!(!excel.contains(&("S".to_string(), "B1".to_string())));
    assert_eq!(
        got,
        vec![
            ("My S".into(), "A1".into(), Value::Number(2.0)),
            ("S".into(), "A1".into(), Value::Number(0.0)),
            ("S".into(), "A2".into(), Value::Number(1.0)),
            ("S".into(), "B1".into(), Value::Number(8.0)), // no oracle value: hand stays
        ]
    );
    assert_eq!(prepared.expected_source, "excel-graph");
}

/// A rich error without an evidence-backed mapping (`#UNDECODED!`, `xlsx_snapshot.rs`) is
/// never scored: the cell stays pending (poi/link-external-workbook-b Sheet0!A1, T-0203).
#[test]
fn undecoded_rich_errors_are_not_scored() {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-undecoded", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("a.case.json");
    let case = format!(
        r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=1","A2":"=2"}}}}]}}"#
    );
    std::fs::write(&path, case).expect("write case");
    let oracle = format!(
        r##"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"excel-graph","application":"x"}},
           "sheets":{{"S":{{"A1":{{"t":"e","v":"#UNDECODED!"}},"A2":{{"t":"n","v":2}}}}}}}}"##
    );
    std::fs::write(dir.join("a.oracle.json"), oracle).expect("write oracle");
    let prepared = prepare_spec(&CaseFile {
        id: "t/a".into(),
        sidecar: oracle_sidecar(&path),
        path,
    });
    let _ = std::fs::remove_dir_all(&dir);
    let cells: Vec<String> = prepared
        .expect("prepare")
        .expectations
        .into_iter()
        .map(|e| e.addr.to_string())
        .collect();
    assert_eq!(cells, vec!["A2".to_string()]);
}

/// A spec's `invariant` cells are scored although they call a volatile function; other
/// volatile cells stay skipped, and listing a cell that is not volatile is an error (T-1283).
#[test]
fn invariant_volatile_cells_are_scored() {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-invariant", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("a.case.json");
    let case_with = |invariant: &str| {
        format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=RANDBETWEEN(5,5)","A2":"=RAND()","A3":"=A1+1","A4":"=1"}}}}],
               "expected":{{"S!A1":{{"t":"n","v":5}},"S!A2":{{"t":"n","v":0.5}},"S!A3":{{"t":"n","v":6}},"S!A4":{{"t":"n","v":1}}}},
               "invariant":[{invariant}]}}"#
        )
    };
    let file = CaseFile {
        id: "t/a".into(),
        path: path.clone(),
        sidecar: oracle_sidecar(&path),
    };
    std::fs::write(&path, case_with(r#""S!A1""#)).expect("write case");
    let listed = prepare_spec(&file);
    std::fs::write(&path, case_with(r#""S!A4""#)).expect("write case");
    let stale = prepare_spec(&file).err();
    let _ = std::fs::remove_dir_all(&dir);
    let listed = listed.expect("prepare");
    let mut cells: Vec<String> = listed
        .expectations
        .iter()
        .map(|e| e.addr.to_string())
        .collect();
    cells.sort();
    assert_eq!(cells, ["A1", "A3", "A4"], "A3 reads only the listed A1");
    assert_eq!(listed.skipped_volatile, 1);
    assert!(stale.is_some_and(|e| e.contains("not a volatile formula cell")));
}

#[test]
fn duplicate_hand_keys_and_bad_oracle_keys_are_errors() {
    let dir = std::env::temp_dir().join(format!("rs-runner-{}-bad-keys", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("a.case.json");
    let case_with = |expected: &str| {
        format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=1"}}}}],"expected":{{{expected}}}}}"#
        )
    };
    let file = CaseFile {
        id: "t/a".into(),
        path: path.clone(),
        sidecar: oracle_sidecar(&path),
    };
    std::fs::write(
        &path,
        case_with(r#""S!A1":{"t":"n","v":1},"'S'!$A$1":{"t":"n","v":2}"#),
    )
    .expect("write case");
    let duplicate = prepare_spec(&file).err();
    std::fs::write(&path, case_with(r#""S!A1":{"t":"n","v":1}"#)).expect("write case");
    let oracle = format!(
        r#"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"k","application":"x"}},"sheets":{{"S":{{"nope":{{"t":"n","v":1}}}}}}}}"#
    );
    std::fs::write(dir.join("a.oracle.json"), oracle).expect("write oracle");
    let bad_oracle = prepare_spec(&file).err();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(duplicate.is_some_and(|e| e.contains("duplicate expectation")));
    assert!(bad_oracle.is_some_and(|e| e.contains("in oracle sidecar")));
}

#[test]
fn pending_case_with_engine_error_is_reported() {
    let r = run_pending("error");
    assert_eq!(r.status, CaseStatus::EngineError);
}
