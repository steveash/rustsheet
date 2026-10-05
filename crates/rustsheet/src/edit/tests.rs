use super::*;
use crate::value::{ErrorKind, XlError};
use crate::WorkbookOptions;

fn book() -> Workbook {
    Workbook::new(&WorkbookOptions::default()).expect("workbook")
}

fn cell(wb: &Workbook, a1: &str) -> CellRef {
    wb.cell_ref(a1).expect("ref")
}

/// `(sheet name, sheet id, [(formula, value)])` per sheet.
type Dump = (String, String, Vec<(Option<String>, Value)>);

/// Everything observable through the facade, for before/after equality.
fn dump(wb: &Workbook) -> Vec<Dump> {
    wb.sheets()
        .iter()
        .map(|s| {
            let cells = match wb.used_range(s.id).expect("used") {
                None => vec![],
                Some(r) => {
                    let mut out = vec![];
                    for row in r.top..=r.bottom {
                        for col in r.left..=r.right {
                            let c = CellRef::new(s.id, row, col).expect("cell");
                            let info = wb.cell(c).expect("info");
                            out.push((info.formula, info.value));
                        }
                    }
                    out
                }
            };
            (s.name.clone(), format!("{:?}", s.id), cells)
        })
        .collect()
}

#[test]
fn set_reports_inputs_and_dependent_values() {
    let mut wb = book();
    wb.set("A1", 1.0).unwrap();
    wb.set("B1", Input::Formula("=A1*2".into())).unwrap();
    let changes = wb.set("A1", 5.0).unwrap();
    assert_eq!(wb.value("B1").unwrap(), Value::Number(10.0));
    assert_eq!(changes.inputs.len(), 1);
    assert_eq!(changes.inputs[0].before.as_deref(), Some("1"));
    assert_eq!(changes.inputs[0].after.as_deref(), Some("5"));
    let values: Vec<_> = changes
        .values
        .iter()
        .map(|v| (v.cell.to_string(), v.before.clone(), v.after.clone()))
        .collect();
    assert_eq!(
        values,
        vec![
            ("A1".into(), Value::Number(1.0), Value::Number(5.0)),
            ("B1".into(), Value::Number(2.0), Value::Number(10.0)),
        ]
    );
}

#[test]
fn unchanged_values_are_not_reported() {
    let mut wb = book();
    wb.set("A1", 1.0).unwrap();
    wb.set("B1", Input::Formula("=A1>0".into())).unwrap();
    wb.set("C1", Input::Formula("=\"x\"&A1".into())).unwrap();
    let changes = wb.set("A1", 2.0).unwrap();
    let cells: Vec<_> = changes.values.iter().map(|v| v.cell.to_string()).collect();
    assert_eq!(cells, ["A1", "C1"]);
    // Rewriting the same input changes nothing.
    let again = wb.set("A1", 2.0).unwrap();
    assert!(again.inputs.is_empty() && again.values.is_empty());
}

#[test]
fn negative_zero_differs_from_zero() {
    let mut wb = book();
    wb.set("A1", 0.0).unwrap();
    let changes = wb.set("A1", -0.0).unwrap();
    assert_eq!(changes.values.len(), 1);
}

#[test]
fn failed_edit_leaves_workbook_unchanged() {
    let mut wb = book();
    wb.set("A1", 1.0).unwrap();
    wb.set("B1", Input::Formula("=A1+1".into())).unwrap();
    wb.set("C1", "hello").unwrap();
    let before = dump(&wb);
    let a1 = cell(&wb, "A1");
    let err = wb
        .edit(|tx| {
            tx.set(a1, 99.0)?;
            tx.set("B1", "plain text")?;
            tx.set("C1", Input::Clear)?;
            tx.set("D4", Input::Formula("=A1".into()))?;
            // Not a formula: rejected, which rolls the whole edit back.
            tx.set("E5", Input::Formula("nope".into()))
        })
        .unwrap_err();
    assert!(matches!(err, EditError::InvalidInput { .. }));
    assert_eq!(dump(&wb), before);
    assert_eq!(wb.value("B1").unwrap(), Value::Number(2.0));
}

#[test]
fn rollback_restores_an_overwritten_spill_anchor() {
    let mut wb = book();
    wb.set("A1", Input::Formula("=SEQUENCE(3)".into())).unwrap();
    assert_eq!(wb.value("A3").unwrap(), Value::Number(3.0));
    let before = dump(&wb);
    let a1 = cell(&wb, "A1");
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set(a1, 7.0)?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
    // A write into a spill member clears the spill; rollback brings it back.
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("A2", "x")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
}

#[test]
fn spill_growth_and_shrink_are_reported() {
    let mut wb = book();
    wb.set("B1", 2.0).unwrap();
    wb.set("A1", Input::Formula("=SEQUENCE(B1)".into()))
        .unwrap();
    let grow = wb.set("B1", 4.0).unwrap();
    let cells: Vec<_> = grow.values.iter().map(|v| v.cell.to_string()).collect();
    assert_eq!(cells, ["B1", "A3", "A4"]);
    let shrink = wb.set("B1", 1.0).unwrap();
    let cells: Vec<_> = shrink.values.iter().map(|v| v.cell.to_string()).collect();
    assert_eq!(cells, ["B1", "A2", "A3", "A4"]);
    assert_eq!(shrink.values[1].after, Value::Empty);
}

#[test]
fn sheet_ops_apply_and_roll_back() {
    let mut wb = book();
    wb.set("A1", 3.0).unwrap();
    let first = wb.sheets()[0].id;
    let (added, changes) = wb
        .edit(|tx| {
            let added = tx.add_sheet("Data")?;
            tx.set(
                CellRef::new(added, 0, 0)?,
                Input::Formula("=Sheet1!A1+1".into()),
            )?;
            tx.rename_sheet(first, "Main")?;
            Ok(added)
        })
        .unwrap();
    assert_eq!(
        wb.value(CellRef::new(added, 0, 0).unwrap()).unwrap(),
        Value::Number(4.0)
    );
    assert_eq!(
        wb.cell(CellRef::new(added, 0, 0).unwrap())
            .unwrap()
            .formula
            .as_deref(),
        Some("=Main!A1+1")
    );
    assert_eq!(changes.sheets.len(), 2);
    assert_eq!(changes.values.len(), 1);

    let before = dump(&wb);
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("Main!A1", 100.0)?;
        tx.delete_sheet(added)?;
        tx.rename_sheet(first, "Other")?;
        tx.add_sheet("Third")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
    assert_eq!(
        wb.value(CellRef::new(added, 0, 0).unwrap()).unwrap(),
        Value::Number(4.0)
    );
}

#[test]
fn deleting_a_sheet_reports_its_cells_and_dependents() {
    let mut wb = book();
    let (data, _) = wb.edit(|tx| tx.add_sheet("Data")).unwrap();
    wb.set("Data!A1", 8.0).unwrap();
    wb.set("A1", Input::Formula("=Data!A1".into())).unwrap();
    let ((), changes) = wb.edit(|tx| tx.delete_sheet(data)).unwrap();
    assert!(matches!(&changes.sheets[..], [SheetChange::Deleted { name, .. }] if name == "Data"));
    let cells: Vec<_> = changes
        .values
        .iter()
        .map(|v| (v.cell.sheet == data, v.cell.to_string()))
        .collect();
    assert_eq!(cells, [(false, "A1".into()), (true, "A1".into())]);
    assert!(matches!(
        wb.value("A1").unwrap(),
        Value::Error(XlError { .. })
    ));
}

#[test]
fn sheet_errors_are_typed() {
    let mut wb = book();
    let only = wb.sheets()[0].id;
    assert!(matches!(
        wb.edit(|tx| tx.delete_sheet(only)),
        Err(EditError::LastSheet)
    ));
    assert!(matches!(
        wb.edit(|tx| tx.add_sheet("Sheet1")),
        Err(EditError::SheetName(_))
    ));
    assert!(matches!(
        wb.edit(|tx| tx.add_sheet("bad/name")),
        Err(EditError::SheetName(_))
    ));
    assert!(matches!(
        wb.edit(|tx| tx.set(CellRef::new(SheetId::new(999), 0, 0).unwrap(), 1.0)),
        Err(EditError::Ref(RefError::UnknownSheet(_)))
    ));
    assert!(matches!(
        wb.set("A1:B2", 1.0),
        Err(EditError::Ref(RefError::Unsupported))
    ));
    assert!(matches!(wb.set("Nope!A1", 1.0), Err(EditError::Ref(_))));
}

#[test]
fn typed_values_are_not_reinterpreted() {
    let mut wb = book();
    wb.set("A1", Value::Text("=1+1".into())).unwrap();
    wb.set("A2", Value::Text("007".into())).unwrap();
    wb.set("A3", Value::Error(XlError::new(ErrorKind::Div0)))
        .unwrap();
    wb.set("A4", "007").unwrap();
    assert_eq!(wb.value("A1").unwrap(), Value::Text("=1+1".into()));
    assert_eq!(wb.value("A2").unwrap(), Value::Text("007".into()));
    assert!(matches!(wb.value("A3").unwrap(), Value::Error(e) if e.kind == ErrorKind::Div0));
    assert_eq!(wb.value("A4").unwrap(), Value::Number(7.0));
    assert!(matches!(
        wb.set("A5", f64::NAN),
        Err(EditError::InvalidInput { .. })
    ));
    assert_eq!(wb.value("A5").unwrap(), Value::Empty);
}

#[test]
fn tracking_can_be_turned_off() {
    let mut wb = book();
    wb.set("B1", Input::Formula("=A1".into())).unwrap();
    wb.set_track_changes(false);
    let changes = wb.set("A1", 4.0).unwrap();
    assert!(changes.values.is_empty());
    assert_eq!(changes.inputs.len(), 1);
    assert_eq!(wb.value("B1").unwrap(), Value::Number(4.0));
}

#[test]
fn volatile_cells_change_on_every_edit() {
    let mut wb = book();
    wb.set("A1", Input::Formula("=RAND()".into())).unwrap();
    wb.set("B1", Input::Formula("=INDIRECT(\"C\"&D1)".into()))
        .unwrap();
    wb.set("C1", 1.0).unwrap();
    wb.set("C2", 2.0).unwrap();
    wb.set("D1", 1.0).unwrap();
    let unrelated = wb.set("F9", 1.0).unwrap();
    assert!(unrelated.values.iter().any(|v| v.cell.to_string() == "A1"));
    let retarget = wb.set("D1", 2.0).unwrap();
    assert!(retarget.values.iter().any(|v| v.cell.to_string() == "B1"));
    assert_eq!(wb.value("B1").unwrap(), Value::Number(2.0));
}

/// Deterministic generator (no proptest in the offline dependency set).
struct Rng(u64);
impl Rng {
    fn next(&mut self, n: u32) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        u32::try_from(self.0 % u64::from(n)).unwrap_or(0)
    }

    fn cell(&mut self) -> String {
        let col = ["A", "B", "C", "D"][usize::try_from(self.next(4)).unwrap_or(0)];
        format!("{col}{}", 1 + self.next(4))
    }
}

fn random_input(rng: &mut Rng) -> Input {
    match rng.next(9) {
        0 => Input::Clear,
        1 => Input::from(f64::from(rng.next(5))),
        2 => Input::from("text"),
        3 => Input::Formula(format!("={}+{}", rng.cell(), rng.cell())),
        4 => Input::Formula(format!("=SEQUENCE({})", 1 + rng.next(3))),
        5 => Input::Formula("=RAND()".into()),
        6 => Input::Formula(format!("=INDIRECT({})", rng.cell())),
        7 => Input::Formula(format!("=SUM({}:{})", rng.cell(), rng.cell())),
        _ => Input::from(rng.next(2) == 0),
    }
}

/// `ChangeSet.values` must list exactly the cells whose value differs between full dumps
/// taken around the edit, and rollback must restore the full dump.
#[test]
fn change_sets_match_full_diffs_on_random_edits() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut wb = book();
    let sheet = wb.sheets()[0].id;
    let grid = |wb: &Workbook| wb.values("A1:F12").expect("grid");
    for round in 0..300 {
        let before_values = grid(&wb);
        let before_dump = dump(&wb);
        let ops: Vec<(String, Input)> = (0..=rng.next(3))
            .map(|_| {
                let a1 = rng.cell();
                (a1, random_input(&mut rng))
            })
            .collect();
        let fail = round % 4 == 3;
        let result = wb.edit(|tx| {
            for (a1, input) in &ops {
                tx.set(a1.as_str(), input.clone())?;
            }
            if fail {
                return Err(EditError::Engine("abort".into()));
            }
            Ok(())
        });
        if fail {
            assert!(result.is_err());
            assert_eq!(dump(&wb), before_dump, "round {round}: rollback");
            continue;
        }
        let ((), changes) = result.unwrap();
        let after = grid(&wb);
        let mut expected = vec![];
        for row in 0..12u32 {
            for col in 0..6u32 {
                let (b, a) = (
                    before_values.get(row as usize, col as usize).unwrap(),
                    after.get(row as usize, col as usize).unwrap(),
                );
                if !same(b, a) {
                    expected.push((row, col, b.clone(), a.clone()));
                }
            }
        }
        let actual: Vec<_> = changes
            .values
            .iter()
            .map(|v| (v.cell.row, v.cell.col, v.before.clone(), v.after.clone()))
            .collect();
        assert!(changes.values.iter().all(|v| v.cell.sheet == sheet));
        assert_eq!(actual.len(), expected.len(), "round {round}: {ops:?}");
        for (e, a) in expected.iter().zip(&actual) {
            assert_eq!((e.0, e.1), (a.0, a.1), "round {round}: {ops:?}");
            assert!(same(&e.2, &a.2) && same(&e.3, &a.3), "round {round}");
        }
    }
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.to_bits() == y.to_bits(),
        _ => a == b,
    }
}

#[test]
fn one_evaluation_per_edit() {
    let mut wb = book();
    let before = wb.model.evaluation_count();
    wb.edit(|tx| {
        for r in 1..=20 {
            tx.set(format!("A{r}").as_str(), f64::from(r))?;
        }
        Ok(())
    })
    .expect("edit");
    assert_eq!(wb.model.evaluation_count(), before + 1);
    wb.set("A1", 2.0).expect("set");
    assert_eq!(wb.model.evaluation_count(), before + 2);
}

#[test]
fn repeated_writes_to_one_cell_roll_back_to_the_original() {
    let mut wb = book();
    wb.set("A1", "=SEQUENCE(3)").expect("spill");
    let original = dump(&wb);
    let r: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("A2", 7.0)?;
        tx.set("A2", 8.0)?;
        tx.set("A1", 1.0)?;
        tx.set("A1", 2.0)?;
        Err(EditError::Engine("x".into()))
    });
    assert!(r.is_err());
    assert_eq!(dump(&wb), original);
}

#[test]
fn footprint_cells_written_after_their_anchor_report_input_changes() {
    let mut wb = book();
    wb.set("A1", "=SEQUENCE(3)").expect("spill");
    let ((), changes) = wb
        .edit(|tx| {
            tx.set("A1", 1.0)?;
            tx.set("A2", 5.0)
        })
        .expect("edit");
    let a2 = cell(&wb, "A2");
    assert!(changes.inputs.iter().any(|c| c.cell == a2), "{changes:?}");
}

#[test]
fn oversized_spill_footprints_are_rejected() {
    use rustsheet_engine::types::Cell;
    let mut wb = book();
    wb.set("A1", "=SEQUENCE(2)").expect("spill");
    // Imported array ranges are untrusted: stretch the anchor's extent past the cap.
    let ws = wb.model.workbook.worksheet_mut(0).expect("sheet");
    let mut anchor = ws.cell(1, 1).cloned().expect("anchor");
    let Cell::ArrayFormula { r, .. } = &mut anchor else {
        panic!("not an array formula: {anchor:?}");
    };
    *r = (1 << 12, (1 << 12) + 1);
    ws.sheet_data.set_cell(1, 1, anchor);
    let err = wb.set("A1", 1.0).unwrap_err();
    assert!(matches!(err, EditError::InvalidInput { .. }), "{err:?}");
}

/// Runs `f` on a 1 MiB stack (the smallest common one, WebAssembly's) in an optimized build, the
/// stack the engine's formula depth limit is sized for; an unoptimized build spends about ten
/// times more stack per nesting level, so it gets 16 MiB.
fn on_small_stack(f: impl FnOnce() + Send + 'static) {
    let stack = if cfg!(debug_assertions) {
        16 << 20
    } else {
        1 << 20
    };
    let handle = std::thread::Builder::new().stack_size(stack).spawn(f);
    assert!(handle.is_ok_and(|h| h.join().is_ok()));
}

#[test]
fn over_nested_formula_is_rejected_not_a_crash() {
    // T-0931: deep nesting used to overflow the parser's stack.
    on_small_stack(|| {
        let mut wb = book();
        wb.set("A1", 1.0).unwrap();
        for formula in [
            format!("={}1{}", "(".repeat(10_000), ")".repeat(10_000)),
            format!("={}1{}", "ABS(".repeat(10_000), ")".repeat(10_000)),
            format!("={}1{}", "{".repeat(10_000), "}".repeat(10_000)),
            format!("={}1", "-(".repeat(10_000)),
            format!("={}", vec!["1"; 10_000].join("+")),
            format!("=1{}", "%".repeat(10_000)),
        ] {
            let err = wb.set("A1", Input::Formula(formula)).unwrap_err();
            assert!(matches!(err, EditError::InvalidInput { .. }), "{err:?}");
            assert_eq!(wb.value("A1").unwrap(), Value::Number(1.0));
        }
    });
}

#[test]
fn formulas_at_the_depth_limit_evaluate() {
    on_small_stack(|| {
        let mut wb = book();
        // 127 nested calls (Excel itself allows 64) and a 255-term chain are the deepest accepted.
        let calls = format!("={}1{}", "ABS(".repeat(127), ")".repeat(127));
        wb.set("A1", Input::Formula(calls)).unwrap();
        assert_eq!(wb.value("A1").unwrap(), Value::Number(1.0));
        wb.set(
            "A2",
            Input::Formula(format!("={}", vec!["1"; 255].join("+"))),
        )
        .unwrap();
        assert_eq!(wb.value("A2").unwrap(), Value::Number(255.0));
    });
}
