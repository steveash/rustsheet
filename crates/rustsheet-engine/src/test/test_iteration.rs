#![allow(clippy::unwrap_used, reason = "tests")]

//! Iterative calculation (docs/excel-semantics/recalc.md; probes/circular-iteration-*, T-0447).
//! The probes' values are for the oracle's three calculations; one `evaluate()` is one.

use crate::test::util::new_empty_model;
use crate::types::Iteration;
use crate::Model;

fn model_with(max_iterations: u32, max_change: f64, cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    model.workbook.settings.iteration = Some(Iteration {
        max_iterations,
        max_change,
    });
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model
}

/// probes/circular-iteration-on: Gauss-Seidel sweeps in row-major order, resuming each
/// calculation from the previous values.
#[test]
fn cycles_sweep_in_sheet_order_and_resume() {
    let mut model = model_with(
        100,
        0.001,
        &[
            ("A1", "=A1+1"),
            ("A3", "=A1+10"),
            ("B1", "=C1+1"),
            ("C1", "=B1+1"),
            ("C2", "=D2+1"),
            ("D2", "=E2+1"),
            ("E2", "=C2+1"),
            ("D1", "=D1/2+1"),
            ("E1", "=MIN(E1+1,5)"),
            ("A2", "=IF(A2<50,A2+1,A2)"),
            ("B2", "=B2"),
        ],
    );
    for _ in 0..3 {
        model.evaluate();
    }
    let t = |c: &str| model._get_text(c);
    assert_eq!([t("B1"), t("C1")], ["599", "600"]);
    assert_eq!([t("C2"), t("D2"), t("E2")], ["449", "450", "450"]);
    assert_eq!([t("A1"), t("A3")], ["300", "310"]);
    assert_eq!([t("D1"), t("E1"), t("A2"), t("B2")], ["2", "5", "50", "0"]);
}

/// probes/circular-iteration-limits: iterateCount bounds every calculation.
#[test]
fn iterate_count_bounds_each_calculation() {
    let mut model = model_with(
        5,
        0.5,
        &[("A1", "=A1+1"), ("D1", "=D1+0.1"), ("F1", "=F1+0.6")],
    );
    for _ in 0..3 {
        model.evaluate();
    }
    assert_eq!(model._get_text("A1"), "15");
    assert_eq!(model._get_text("D1"), "1.5");
}

/// probes/circular-iteration-delta-boundary: a calculation stops at the first sweep whose largest
/// change is strictly below the delta; each later calculation runs one more sweep.
#[test]
fn the_delta_stops_strictly_below() {
    let mut model = model_with(100, 0.25, &[("A1", "=A1/2+1")]);
    for _ in 0..3 {
        model.evaluate();
    }
    assert_eq!(model._get_text("A1"), "1.96875");
}

/// Without iteration a cycle is 0, not iterated (recalc.md "A circular reference without
/// iteration is 0", T-0445).
#[test]
fn without_iteration_a_cycle_is_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=A1+1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "0");
}

/// Evaluates `cells` (and the data tables) three times with the pass caches and three times
/// without, and checks every cell of A1:N220 after each evaluation: a cached read under iteration
/// joins the readers the walk joins (T-0485). Returns the cached model.
fn cached_reads_match_the_walk(
    cells: &[(String, String)],
    tables: &[crate::types::DataTable],
) -> Model {
    let build = |no_pass_caches: bool| {
        let mut model = model_with(100, 0.001, &[]);
        model.evaluation.no_pass_caches = no_pass_caches;
        for (cell, value) in cells {
            model._set(cell, value);
        }
        for table in tables {
            model.add_data_table(0, table.clone()).unwrap();
        }
        model
    };
    let mut cached = build(false);
    let mut walked = build(true);
    for round in 0..3 {
        cached.evaluate();
        walked.evaluate();
        for row in 1..=220 {
            for column in 'A'..='N' {
                let cell = format!("{column}{row}");
                assert_eq!(
                    cached._get_text(&cell),
                    walked._get_text(&cell),
                    "{cell} after evaluation {round}"
                );
            }
        }
    }
    cached
}

fn cells(list: impl IntoIterator<Item = (String, String)>) -> Vec<(String, String)> {
    list.into_iter().collect()
}

/// A cycle through a criteria range: A50 counts the range it sits in, and the readers below
/// count, sum and match it with criteria that need different cells.
#[test]
fn a_cycle_through_a_criteria_range_reads_as_the_walk() {
    let mut list = cells((1..=100).map(|row| (format!("A{row}"), row.to_string())));
    list.push(("A50".into(), "=COUNTIFS($A$1:$A$100,\">10\")/10".into()));
    for row in 1..=30 {
        list.push((
            format!("B{row}"),
            format!("=COUNTIFS($A$1:$A$100,\">\"&{row})"),
        ));
        list.push((
            format!("C{row}"),
            format!(
                "=SUMIFS($A$1:$A$100,$A$1:$A$100,\"<\"&{}) + COUNTIF($A$1:$A$100,{row})",
                row * 3
            ),
        ));
        list.push((
            format!("D{row}"),
            format!("=COUNTIFS($A$1:$A${},\">0\")", 70 + row),
        ));
    }
    cached_reads_match_the_walk(&list, &[]);
}

/// A cycle through a lookup range: D70 looks up the range it sits in.
#[test]
fn a_cycle_through_a_lookup_range_reads_as_the_walk() {
    let mut list = cells((1..=100).map(|row| (format!("E{row}"), (row * 2).to_string())));
    list.push(("E70".into(), "=MATCH(40,$E$1:$E$100,0)+E69".into()));
    for row in 1..=30 {
        list.push((
            format!("F{row}"),
            format!("=MATCH({},$E$1:$E$100,0)", row * 2),
        ));
        list.push((
            format!("G{row}"),
            format!("=IFERROR(VLOOKUP({},$E$1:$E$100,1,FALSE),-1)", row * 4),
        ));
        list.push((
            format!("H{row}"),
            format!("=XMATCH({},$E$1:$E$100)", 140 - row),
        ));
    }
    cached_reads_match_the_walk(&list, &[]);
}

/// Cycles that close after a range was cached: I1:I30 read the range first, then K5 and K6 close
/// a cycle through a reader, and I95 joins one through a later reader of its own range.
#[test]
fn cycles_that_close_after_a_range_was_cached_read_as_the_walk() {
    let mut list = cells((1..=100).map(|row| (format!("J{row}"), (row % 17).to_string())));
    for row in 1..=30 {
        list.push((
            format!("I{row}"),
            format!(
                "=COUNTIFS($J$1:$J$100,\">\"&{}) + MATCH({},$J$1:$J$100,0)",
                row % 9,
                row % 16
            ),
        ));
    }
    list.push(("K5".into(), "=COUNTIFS($J$1:$J$100,\">3\")+K6".into()));
    list.push(("K6".into(), "=IF(K5>200,0,K5/2)".into()));
    list.push(("J95".into(), "=L200".into()));
    list.push((
        "L200".into(),
        "=COUNTIFS($J$1:$J$100,\">5\")+MATCH(3,$J$1:$J$100,0)".into(),
    ));
    for row in 31..=60 {
        list.push((
            format!("I{row}"),
            format!(
                "=COUNTIFS($J$1:$J$100,\"<\"&{}) + SUMIFS($J$1:$J$100,$J$1:$J$100,\">\"&{})",
                row % 11,
                row % 7
            ),
        ));
    }
    cached_reads_match_the_walk(&list, &[]);
}

/// The round after data tables: a table's formula counts a range holding a cycle, and readers of
/// the table and the range read as the walk in both rounds.
#[test]
fn the_data_table_round_reads_as_the_walk() {
    let mut list = cells((1..=100).map(|row| (format!("M{row}"), row.to_string())));
    list.push(("M80".into(), "=COUNTIFS($M$1:$M$100,\">\"&A1)/4".into()));
    list.push(("A1".into(), "10".into()));
    list.push(("B3".into(), "=COUNTIFS($M$1:$M$100,\">\"&A1)".into()));
    for (row, value) in [(4, "5"), (5, "50"), (6, "90")] {
        list.push((format!("A{row}"), value.into()));
    }
    for row in 10..=40 {
        list.push((
            format!("N{row}"),
            format!("=COUNTIFS($M$1:$M$100,\"<\"&{row})+SUM($B$4:$B$6)"),
        ));
    }
    let table = crate::types::DataTable {
        range: (4, 2, 6, 2),
        row_input: None,
        column_input: Some(crate::types::DataTableInput::Cell(1, 1)),
    };
    cached_reads_match_the_walk(&list, &[table]);
}

/// A workbook saved with iteration on but no cycle keeps the pass caches (calipers `pv26_002`,
/// T-0485).
#[test]
fn without_a_cycle_iteration_keeps_the_caches() {
    let mut list = cells((1..=100).map(|row| (format!("A{row}"), (row % 13).to_string())));
    for row in 1..=30 {
        list.push((
            format!("B{row}"),
            format!(
                "=COUNTIFS($A$1:$A$100,\">\"&{}) + MATCH({},$A$1:$A$100,0)",
                row % 7,
                row % 12
            ),
        ));
    }
    let model = cached_reads_match_the_walk(&list, &[]);
    assert!(model.evaluation.ifs_cache.indexed() > 0);
}

/// Who joins a cycle shows in the sweep count: with a delta of 1.5, `K1 = K1+1` changes by 1 and
/// stops after one sweep, unless a member reader changed by more since the last calculation.
/// `cells` are set, evaluated, `D1` is set to 100 and evaluated again, with and without the pass
/// caches; returns K1 both ways.
fn k1_after_an_edit(cells: &[(String, String)]) -> (String, String) {
    let run = |no_pass_caches: bool| {
        let mut model = model_with(100, 1.5, &[("K1", "=K1+1"), ("D1", "1")]);
        model.evaluation.no_pass_caches = no_pass_caches;
        for (cell, value) in cells {
            model._set(cell, value);
        }
        model.evaluate();
        model._set("D1", "100");
        model.evaluate();
        model._get_text("K1")
    };
    (run(false), run(true))
}

/// A sum range's cycle member in a row whose criterion fails: the walk never reads it, so its
/// readers do not join, and the cached range values must not read it either (T-0485).
#[test]
fn a_member_the_lazy_walk_skips_joins_no_cached_reader() {
    let mut list = cells((1..=100).flat_map(|row| {
        [
            (format!("A{row}"), "x".to_string()),
            (format!("B{row}"), row.to_string()),
        ]
    }));
    list.push(("A50".into(), "y".into()));
    list.push(("B50".into(), "=K1".into()));
    for row in 101..=110 {
        list.push((
            format!("C{row}"),
            "=SUMIFS($B$1:$B$100,$A$1:$A$100,\"x\")+$D$1".into(),
        ));
    }
    let (cached, walked) = k1_after_an_edit(&list);
    assert_eq!(cached, walked);
    assert_eq!(walked, "2");
}

/// A lookup range holding a cycle member: every reader joins, so a vector whose build joined is
/// not cached for the readers after it (T-0485).
#[test]
fn a_lookup_vector_whose_build_joined_is_not_cached() {
    let mut list = cells((1..=100).map(|row| (format!("E{row}"), (row + 10).to_string())));
    list.push(("E50".into(), "=K1".into()));
    list.push(("F101".into(), "=MATCH(15,$E$1:$E$100,0)".into()));
    for row in 102..=110 {
        list.push((format!("F{row}"), "=MATCH(15,$E$1:$E$100,0)+$D$1".into()));
    }
    let (cached, walked) = k1_after_an_edit(&list);
    assert_eq!(cached, walked);
    assert_eq!(walked, "4");
}
