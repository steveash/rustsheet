//! End to end: `rustsheet-eval accuracy`, the Gate A check `make eval-gates` (and so `make check`)
//! runs (SPEC-0029, decision 0193), on a fixture repository under `tests/data/accuracy-fixture`.
//!
//! The fixture is not a real table: its Excel and math values are made up to put one point in
//! each class. SQRT/s1 is filled and inside (an R3 point among them), LN/s1 is pending, and EXP/s1
//! holds a known failure, a stale known-failure line, a failing point and a stale math value.
//! Its catalog view lists ABS, an in-scope numeric function with no table and no exemption (the
//! missing table), and `exempt.tsv` exempts COUNT.

// Helpers live in a test module so clippy's test allowances (clippy.toml) cover them.
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/accuracy-fixture")
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("rs-accuracy-gate-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    /// Runs the check on the repository at `root`; returns the exit code, stdout and the JSON.
    fn check(root: &Path, json: &Path) -> (Option<i32>, String, serde_json::Value) {
        let out = Command::new(env!("CARGO_BIN_EXE_rustsheet-eval"))
            .arg("--root")
            .arg(root)
            .arg("accuracy")
            .arg("--json")
            .arg(json)
            .output()
            .expect("runs");
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr);
        let text =
            std::fs::read_to_string(json).unwrap_or_else(|e| panic!("{e}\n{stdout}{stderr}"));
        let parsed = serde_json::from_str(&text).expect("json");
        (out.status.code(), stdout, parsed)
    }

    fn copy_tree(from: &Path, to: &Path) {
        for entry in walk(from) {
            let rel = entry.strip_prefix(from).expect("inside");
            let dest = to.join(rel);
            std::fs::create_dir_all(dest.parent().expect("parent")).expect("mkdir");
            std::fs::copy(&entry, &dest).expect("copy");
        }
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.extend(walk(&path));
            } else {
                out.push(path);
            }
        }
        out
    }

    #[test]
    fn a_failing_point_a_stale_line_and_a_table_defect_fail_the_gate() {
        let tmp = scratch("fail");
        let (code, stdout, json) = check(&fixture(), &tmp.join("accuracy.json"));
        std::fs::remove_dir_all(&tmp).expect("cleanup");
        assert_eq!(code, Some(1), "{stdout}");
        let summary = stdout.lines().next().unwrap_or_default();
        assert!(
            summary.starts_with(
                "accuracy: 9 points in 3 tables (pending 2): 6 in envelope (Excel 15 digits only 1, math only 3), 1 known failures (T-3356 1), 1 failing, 1 table defects; missing tables 1; "
            ),
            "{stdout}"
        );
        assert!(summary.ends_with(" s"), "{summary}");
        assert!(
            stdout.contains("FAIL accuracy/EXP/s1!Calc!A3 edge =EXP(0)\n  engine 1E0  excel 2E0  math 2E0  rel(math) 5.0E-1"),
            "{stdout}"
        );
        assert!(
            stdout.contains("KNOWN (T-3356) accuracy/EXP/s1!Calc!A1 typical =EXP(1)"),
            "{stdout}"
        );
        assert!(
            stdout.contains("STALE known-failures.tsv line 3: accuracy/EXP/s1!Calc!A2: the point is inside the envelope: remove the line"),
            "{stdout}"
        );
        assert!(
            stdout.contains("TABLE accuracy/EXP/s1!Calc!A4 =EXP(3): stale math value for =EXP(4)"),
            "{stdout}"
        );
        assert!(!stdout.contains("BAD MARK"), "{stdout}");
        // The failure is listed before the known failure.
        let fail = stdout.find("FAIL ").unwrap_or(usize::MAX);
        let known = stdout.find("KNOWN ").unwrap_or(0);
        assert!(fail < known, "{stdout}");

        assert_eq!(json["passes"], false);
        assert_eq!(json["holds"], false);
        assert_eq!(json["check"]["missing_tables"], serde_json::json!(["ABS"]));
        let points = json["check"]["points"].as_array().expect("points");
        assert_eq!(points.len(), 9);
        let point = |id: &str| {
            points
                .iter()
                .find(|p| p["id"] == id)
                .unwrap_or_else(|| panic!("{id}"))
        };
        let r3 = point("accuracy/SQRT/s1!Calc!A3");
        assert_eq!(r3["r3"], true);
        assert_eq!(r3["class"], "math");
        assert_eq!(point("accuracy/SQRT/s1!Calc!A2")["class"], "both");
        assert_eq!(
            point("accuracy/LN/s1!Calc!A2")["excel"],
            serde_json::Value::Null
        );
        assert_eq!(point("accuracy/LN/s1!Calc!A2")["class"], "math");
        assert_eq!(
            point("accuracy/EXP/s1!Calc!A1")["standing"],
            "known_failure"
        );
        assert_eq!(point("accuracy/EXP/s1!Calc!A3")["standing"], "failing");
        assert_eq!(point("accuracy/EXP/s1!Calc!A4")["class"], "table");
        assert_eq!(
            point("accuracy/LN/s1!Calc!A1")["math"],
            serde_json::json!({"t": "n", "v": std::f64::consts::LN_10})
        );
    }

    #[test]
    fn tables_inside_the_envelope_pass_the_gate() {
        let tmp = scratch("pass");
        let root = tmp.join("repo");
        copy_tree(&fixture(), &root);
        std::fs::remove_dir_all(root.join("eval/accuracy/tables/EXP")).expect("rm");
        std::fs::remove_file(root.join("eval/accuracy/known-failures.tsv")).expect("rm");
        let (code, stdout, json) = check(&root, &tmp.join("accuracy.json"));
        std::fs::remove_dir_all(&tmp).expect("cleanup");
        assert_eq!(code, Some(0), "{stdout}");
        assert!(
            stdout.starts_with("accuracy: 5 points in 2 tables (pending 2): 5 in envelope (Excel 15 digits only 0, math only 3), 0 known failures, 0 failing, 0 table defects; missing tables 2;"),
            "{stdout}"
        );
        assert_eq!(stdout.lines().count(), 1, "{stdout}");
        assert_eq!(json["passes"], true);
        // Missing tables (EXP, ABS) and pending points keep Gate A from holding, not the check.
        assert_eq!(json["holds"], false);
        assert_eq!(
            json["check"]["missing_tables"],
            serde_json::json!(["ABS", "EXP"])
        );
    }

    #[test]
    fn a_stale_exemption_fails_the_gate() {
        let tmp = scratch("exempt");
        let root = tmp.join("repo");
        copy_tree(&fixture(), &root);
        std::fs::remove_dir_all(root.join("eval/accuracy/tables/EXP")).expect("rm");
        std::fs::remove_file(root.join("eval/accuracy/known-failures.tsv")).expect("rm");
        // SQRT has a table, UPPER returns text: neither may be exempt. ABS's line clears it.
        let exempt = root.join("eval/accuracy/exempt.tsv");
        let text = std::fs::read_to_string(&exempt).expect("read");
        std::fs::write(
            &exempt,
            format!("{text}SQRT\texact\tx\nUPPER\texact\tx\nABS\tcomposite\tprobes\n"),
        )
        .expect("write");
        let (code, stdout, json) = check(&root, &tmp.join("accuracy.json"));
        std::fs::remove_dir_all(&tmp).expect("cleanup");
        assert_eq!(code, Some(1), "{stdout}");
        assert!(stdout.contains("missing tables 1;"), "{stdout}");
        assert!(
            stdout.contains(
                "STALE exempt.tsv line 3: SQRT: the function has a table: remove the line"
            ),
            "{stdout}"
        );
        assert!(
            stdout.contains("STALE exempt.tsv line 4: UPPER: not an in-scope catalog function returning a number"),
            "{stdout}"
        );
        assert_eq!(json["check"]["missing_tables"], serde_json::json!(["EXP"]));
    }

    #[test]
    fn an_r3_mark_without_evidence_fails_the_gate() {
        let tmp = scratch("mark");
        let root = tmp.join("repo");
        copy_tree(&fixture(), &root);
        std::fs::remove_dir_all(root.join("eval/accuracy/tables/EXP")).expect("rm");
        std::fs::remove_file(root.join("eval/accuracy/known-failures.tsv")).expect("rm");
        // Mark A1, where Excel agrees with the math, and a point no table holds.
        let marks = root.join("eval/accuracy/math-over-excel.tsv");
        let text = std::fs::read_to_string(&marks).expect("read");
        let entry = "docs/excel-semantics/fixture.md#fixture-region-math-over-excel";
        std::fs::write(
            &marks,
            format!("{text}accuracy/SQRT/s1!Calc!A1\t{entry}\naccuracy/SQRT/s1!Calc!A9\t{entry}\n"),
        )
        .expect("write");
        let (code, stdout, _) = check(&root, &tmp.join("accuracy.json"));
        std::fs::remove_dir_all(&tmp).expect("cleanup");
        assert_eq!(code, Some(1), "{stdout}");
        assert!(
            stdout.contains(
                "BAD MARK math-over-excel.tsv line 3: accuracy/SQRT/s1!Calc!A1: no evidence"
            ),
            "{stdout}"
        );
        assert!(
            stdout.contains("BAD MARK math-over-excel.tsv line 4: accuracy/SQRT/s1!Calc!A9: no table holds the point"),
            "{stdout}"
        );
    }
}
