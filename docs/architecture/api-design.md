# Public API design (proposal → becomes SPEC-0001 in M1)

The API is **inspired by** the GRID spreadsheet engine's public docs (a Model/Workbook/Sheet/Cell
object model, `runFormula`-style ad-hoc evaluation, typed edit-blocking errors, error details with
origin) but not copied. It also fixes the gaps of GRID, IronCalc, HyperFormula and Formualizer:
no atomicity, no change reporting, no lazy compute, no graph introspection, string errors, and
lifetimes on public types. The full comparison and Rust signatures are in
`docs/research/api-landscape.md` §4. This page is the condensed contract.

## Principles
1. **Excel errors are values; `Err` is API misuse.** `Value::Error(XlError{kind, detail, origin})`.
2. **Every mutation is a transaction returning a `ChangeSet`.** It is atomic (rollback on error)
   and lists inputs changed, values changed (before/after), spill changes, structural changes,
   and the stale set in lazy modes. The same type drives undo, events, audit and sync.
3. **Stable identity:** `SheetId` survives renames. Internal coordinates are 0-based `u32`; A1 strings are parsed once.
4. **Cheap immutable snapshots** (`Send + Sync`). Readers run in parallel with a writer.
   `Scenario` gives side-effect-free what-if; `Calculator` turns a workbook into a pure function
   for batch evaluation.
5. **Lazy where allowed:** `CalcMode::{Automatic, AutomaticExceptTables, Manual, OnDemand}`, with `watch()` ranges.
6. **No lifetimes or generics on public types**, so the API maps 1:1 to Python, UniFFI/Java and C.

## Surface (abridged)
```rust
let mut wb = Workbook::new(WorkbookOptions::default());           // or Workbook::open(path, LoadOptions)
let s = wb.sheet_id("Sheet1")?;
let (_, cs) = wb.edit(|tx| {
    tx.set("Sheet1!A1", 10.0)?;
    tx.set("Sheet1!A2", "=A1*2")?;                                // Input::Parse semantics
    tx.insert_rows(s, 0, 1)                                       // returns RefMap in cs.structure
})?;
wb.value("Sheet1!A3")?;                                           // Value::Number(20.0)
wb.eval("=SUM(Sheet1!A:A)*rate", EvalContext::bind("rate", 0.1))?;
wb.precedents("Sheet1!A3", Depth::Direct)?;
let snap = wb.snapshot();                                          // O(1), Send + Sync
let what_if = snap.with_inputs(&[("Sheet1!A2".into(), 5.0.into())])?;
wb.save_xlsx(file, SaveOptions::default())?;
```

## Python mapping
```python
import rustsheet as rs
wb = rs.open("model.xlsx", calc_mode="on_demand")
with wb.edit() as tx:
    tx["Inputs!B3"] = 0.08
print(wb["Summary!B5"], tx.changes.values)
wb.values("Data!A1:D1000").to_numpy()
```
Excel errors become `rs.XlError` *values*; API failures raise typed exceptions. Recalc, load and
save release the GIL.

## Java (M12)
A flat FFI facade (ids, strings, f64, enums, flat vectors, opaque handles), exposed through
UniFFI (→ Kotlin/Java) or Panama FFM via `cbindgen` for bulk transfer. JNI is the fallback.

## Phasing
| Phase | Milestone | Content |
|---|---|---|
| v0 | M1 | facade over the current `Model`; `ChangeSet` by diffing; `eval` via AST; Python on facade |
| v0.5 | M3 | `ChangeSet` from dirty propagation; CalcMode; precedents/dependents |
| v0.8 | M4 | snapshots, scenarios, `RefMap` structural edits |
| v1 | M12 | freeze + semver checks; Calculator; Java |

The Python package's IronCalc-era surface (raw `Model`, `UserModel`, the bitcode/icalc loaders, external diffs,
`test_panic`) is decided by SPEC-0026: the byte loaders, diffs and `test_panic` go, and the classes move to a
deprecated `rustsheet.legacy` module until the v1 freeze.
