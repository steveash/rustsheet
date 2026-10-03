# SPEC-0001 — Public API v0 (facade crate `rustsheet`)

- **Status:** accepted (on merge of T-0100)
- **Milestone / lane:** M1 / api
- **Tasks:** T-0100 (this spec), T-0101, T-0102, T-0103, T-0104, T-0113

## Problem
Every consumer (eval harness, Python bindings) talks to the forked IronCalc `Model<'a>` or
`UserModel` directly. Those expose `Result<_, String>`, positional `u32` sheet indexes, a lifetime
parameter, `CellValue` without errors (errors are flattened to strings) and no change reporting.
The eval harness already re-implements value extraction from `Cell`
(`crates/rustsheet-eval/src/engine.rs`) because no typed read API exists. Decision 0004 requires
a lifetime-free, FFI-friendly surface. `docs/architecture/api-design.md` holds the full target
design; this spec fixes what **v0** implements on top of the *current* `Model`.

## Goals / non-goals
Goals:
- A new crate `crates/rustsheet` (package `rustsheet`) that is the only API external callers and
  the Python binding use. Typed values, typed errors, stable sheet ids, atomic edits with a
  `ChangeSet` (computed by diffing).
- No lifetimes or generics on public types, no `String` errors, no panics on untrusted input.

Non-goals (later phases per api-design.md "Phasing"): incremental `ChangeSet` from dirty
propagation, `CalcMode`, precedents/dependents (v0.5, M3); snapshots, scenarios, structural edits
and `RefMap` (v0.8, M4); undo/redo; styles; tables and names editing; `Calculator`; Java/FFI;
numpy/arrow bulk transfer (M12). The `UserModel` UI layer stays out of the facade.

## Excel behavior to match
The facade adds no calculation semantics. Obligations:
- `Input::Parse` must behave as Excel "typed into a cell" (`12%`, dates, `'text`, `=formula`).
  v0 delegates to `Model::set_user_input`; deviations are engine bugs filed against the engine
  lane, not worked around in the facade.
- Error values keep their Excel literal (`#DIV/0!`, `#N/A`, ...). `ErrorKind` maps 1:1 to the
  engine's `token::Error` (`REF NAME VALUE DIV NA NUM ERROR NIMPL SPILL CALC CIRC NULL`); the
  literal round-trips through `ErrorKind::literal()` / `FromStr`. `ERROR` and `NIMPL` surface as
  `ErrorKind::Unknown` and `ErrorKind::NotImplemented`. The facade never rewrites a value.
- Sheet-name lookup is case-insensitive, as in Excel.

## Design

### Crate layout
```
crates/rustsheet/src/
  lib.rs        re-exports; #![deny(missing_docs)], no unsafe
  workbook.rs   Workbook, WorkbookOptions, LoadOptions, SaveOptions, SheetMeta
  refs.rs       SheetId, CellRef, RangeRef, A1 parsing/printing
  value.rs      Value, XlError, ErrorKind, Array
  edit.rs       Txn, Input, ChangeSet, CellChange, EditError
  error.rs      RefError, LoadError, SaveError (thiserror)
```
Dependencies: `rustsheet-engine`, `rustsheet-xlsx`, `thiserror`. The eval crate and
`bindings/python` depend on `rustsheet`, never on engine internals (T-0102, T-0104).
Engine types never appear in public signatures. Prerequisite T-0101 removes `Model`'s lifetime so
`Workbook` can own a `Model`.

### Identities and references
```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)] pub struct SheetId(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)] pub struct CellRef { pub sheet: SheetId, pub row: u32, pub col: u32 }
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RangeRef { pub sheet: SheetId, pub top: u32, pub left: u32, pub bottom: u32, pub right: u32 } // inclusive, 0-based
```
- `SheetId` wraps the engine's persistent worksheet `sheet_id` (stable across rename/reorder; unique but not necessarily dense or ordered, as imported files carry their own ids),
  *not* the positional index. The facade maps id → current index on each call (O(sheets)).
- Rows/cols are bounds-checked (reusing the engine's `LAST_ROW`/`LAST_COLUMN` and column-name helpers, not re-implemented) against the Excel limits (1,048,576 × 16,384) at parse and
  construction time (`RangeRef::new` returns `Result`).
- `Workbook::cell_ref(&self, a1: &str) -> Result<CellRef, RefError>` and `range_ref` parse
  `Sheet1!A1`, `'My Sheet'!A1:B2`, `A:A`, `1:3`, absolute `$` markers (ignored). An unqualified
  reference resolves against the first sheet. Defined names and structured references are not
  accepted in v0 (`RefError::Unsupported`).
- Methods take `impl Into<Target>` where `Target` is `CellRef | RangeRef | &str`; strings are
  parsed per call. `CellRef`/`RangeRef` implement `Display` (A1 with sheet name quoted as needed).

### Values and errors
```rust
#[derive(Clone, Debug, PartialEq)] #[non_exhaustive]
pub enum Value { Empty, Number(f64), Text(Arc<str>), Bool(bool), Error(XlError) }

#[derive(Clone, Debug, PartialEq)]
pub struct XlError { pub kind: ErrorKind, pub detail: Option<Arc<str>> }   // detail from FormulaValue::Error.message
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)] #[non_exhaustive]
pub enum ErrorKind { Null, Div0, Value, Ref, Name, Num, NA, Spill, Calc, Circ, NotImplemented, Unknown }
impl ErrorKind { pub fn literal(self) -> &'static str; pub fn code(self) -> Option<u8> /* ERROR.TYPE */ }

pub struct Array { /* rows, cols, row-major Vec<Value> */ }   // returned by values()
```
- `XlError.origin` and `Value::Array` from api-design.md are deferred (the engine keeps the
  origin only as a string, `FormulaValue::Error.o`, not a typed `CellRef`; arrays only matter for `eval`). `#[non_exhaustive]` lets them arrive without a major
  bump. `GettingData`, `Field`, `Blocked`, `Connect`, `Busy` are added when the engine can
  produce them.
- Spill members read as their own scalar, `Empty` if the cell is not part of a spill.
- `values(range)` is bounded: the cell count is computed with checked multiplication *before*
  any allocation and refused with `RefError::TooLarge` when it exceeds `WorkbookOptions::max_read_cells`
  (default 1,000,000). The output `Array` is dense, but the scan visits stored cells only
  (O(stored cells), not O(rectangle)); callers should intersect with `used_range` first.
- `Value::Text` clones an `Arc<str>` built from the engine's string once per read (a v0 copy
  cost; the engine stores `String`/shared strings, not `Arc<str>`). Callers in loops should pre-parse
  references (`cell_ref`/`range_ref`) and hoist `sheet_id`; `SheetId` → index is O(sheets).

### Workbook
```rust
pub struct Workbook { /* owns engine Model; Send, !Sync */ }
pub struct WorkbookOptions { pub max_read_cells: u64, pub locale: String /* "en-US" */, pub timezone: String /* "UTC" */, pub language: String /* "en" */ }
pub struct LoadOptions { pub recalc: bool /* default true */, pub max_uncompressed_bytes: u64, pub max_zip_entries: u32 }
pub struct SaveOptions {}                                      // placeholder, #[non_exhaustive]
pub struct SheetMeta { pub id: SheetId, pub name: String, pub index: u32 }

impl Workbook {
    pub fn new(opts: WorkbookOptions) -> Result<Self, LoadError>;
    pub fn open(path: impl AsRef<Path>, opts: LoadOptions) -> Result<Self, LoadError>;
    pub fn from_xlsx_bytes(bytes: &[u8], opts: LoadOptions) -> Result<Self, LoadError>;
    pub fn save_xlsx(&self, path: impl AsRef<Path>, opts: SaveOptions) -> Result<(), SaveError>;
    pub fn to_xlsx_bytes(&self, opts: SaveOptions) -> Result<Vec<u8>, SaveError>;

    pub fn sheets(&self) -> Vec<SheetMeta>;
    pub fn sheet_id(&self, name: &str) -> Result<SheetId, RefError>;
    pub fn cell_ref(&self, a1: &str) -> Result<CellRef, RefError>;
    pub fn range_ref(&self, a1: &str) -> Result<RangeRef, RefError>;

    pub fn value(&self, r: impl Into<Target>) -> Result<Value, RefError>;     // top-left of a range
    pub fn values(&self, r: impl Into<Target>) -> Result<Array, RefError>;
    pub fn cell(&self, r: impl Into<Target>) -> Result<CellInfo, RefError>;
    pub fn formatted(&self, r: impl Into<Target>) -> Result<String, RefError>;
    pub fn used_range(&self, sheet: SheetId) -> Result<Option<RangeRef>, RefError>;
}
pub struct CellInfo { pub value: Value, pub formula: Option<String>, pub number_format: Option<String> }
```
`LoadOptions` also carries limits for untrusted workbooks (`max_uncompressed_bytes`, `max_zip_entries`);
violations return `LoadError::Options`. Loading never resolves external-link targets or part
names against the filesystem. T-0102 adds a zip-bomb test.

Reads never mutate. `open`/`from_xlsx_bytes` recalculate once when `opts.recalc`. Loading and
saving are the only I/O; formula evaluation performs none (CLAUDE.md prime directive 5).
Because the engine currently stores computed values in cells, "last computed" equals "current"
in v0 (every edit recalculates synchronously).

### Edits and change sets
```rust
pub enum Input { Value(Value), Formula(String), Parse(String), Clear }
impl From<f64>/From<bool>/From<&str> /* = Parse */ /* for Input */;

impl Workbook {
    pub fn edit<T>(&mut self, f: impl FnOnce(&mut Txn<'_>) -> Result<T, EditError>) -> Result<(T, ChangeSet), EditError>;
    pub fn set(&mut self, r: impl Into<Target>, input: impl Into<Input>) -> Result<ChangeSet, EditError>; // one-cell edit()
}
impl Txn<'_> {
    pub fn set(&mut self, r: impl Into<Target>, input: impl Into<Input>) -> Result<(), EditError>;
    pub fn add_sheet(&mut self, name: &str) -> Result<SheetId, EditError>;
    pub fn rename_sheet(&mut self, id: SheetId, new_name: &str) -> Result<(), EditError>;
    pub fn delete_sheet(&mut self, id: SheetId) -> Result<(), EditError>;
}
pub struct ChangeSet { pub inputs: Vec<InputChange>, pub values: Vec<ValueChange>, pub sheets: Vec<SheetChange> }
pub struct InputChange { pub cell: CellRef, pub before: Option<String>, pub after: Option<String> } // formula or literal text as the user sees it
pub struct ValueChange { pub cell: CellRef, pub before: Value, pub after: Value }
#[non_exhaustive] pub enum SheetChange { Added(SheetId), Renamed { id: SheetId, from: String, to: String }, Deleted { id: SheetId, name: String } }
```
Semantics:
- **`Txn` is write-only.** It has no reads and never exposes stale state: no recalculation
  happens inside `f`, every op validates against the *structure* (sheet list, bounds) only, and a
  formula referencing a sheet added earlier in the same transaction is valid because ops apply to
  the model immediately and only evaluation is deferred. After `f` returns `Ok`, one recalc runs
  and only then does anything observe values.
- **Atomic, via an undo log.** Each op records the prior stored cell (or sheet) in an undo log
  before applying; on `Err` the log is replayed in reverse (O(touched cells)) and nothing is
  recalculated. Replay goes through the engine's own cell setters so derived state
  (`parsed_formulas`, shared strings, parsed defined names) stays consistent; sheet-level ops
  that cannot be replayed that way fall back to restoring a cloned `Workbook` followed by
  `reset_parsed_structures`. `set` on one cell therefore never clones the workbook. Whole-model cloning is not
  permitted as the implementation.
- **ChangeSet.** Before applying, the facade captures the `Value` of every *formula* cell plus the
  touched cells (value changes outside the edit can only occur in formula cells); after the
  recalc it compares those. So `values` lists exactly the cells whose `Value` changed (including
  spill members and cells on added/deleted sheets), sorted by (sheet index, row, col). Memory is
  O(formula cells + touched cells); time is O(formula cells) for the compare plus the recalc.
  `Text` is compared by `Arc` pointer first, then contents. `inputs` lists cells whose stored
  input changed, last write winning within one transaction. Number equality is bitwise
  (`-0.0 != 0.0`, `NaN == NaN`).
- **Volatile cells** (NOW, RAND, INDIRECT/OFFSET with changing targets) change value on every
  recalc and are therefore reported in every `ChangeSet`; this is intended and M3 dirty tracking
  must preserve it (test with INDIRECT whose target changes).
- **Opt-out.** `Workbook::set_track_changes(false)` (default true) makes `edit`/`set` return a
  `ChangeSet` with empty `values`; the eval harness and bulk loads use it so they pay no diff cost.
- **Size envelope.** v0 targets workbooks up to ~100k formula cells with single-cell edit latency
  dominated by the recalc. T-0103 records benchmark numbers (one-cell edit and rollback on a
  workbook of ≥100k formulas) in its Outcome; an O(changed) design (dirty-set `ChangeSet`) is
  required by M3 and is not blocked by anything in this API. A very large `ChangeSet` (1M
  dependents → 1M `ValueChange`) is possible by design; callers wanting no diff use the opt-out.
- Sheet rename rewrites formulas via the engine; the facade reports the rename and any resulting
  value changes only.
- The contract (not the algorithm) is what T-0103 tests, so M3 can swap in dirty-set tracking unchanged.

### Errors
```rust
#[derive(thiserror::Error, Debug)] #[non_exhaustive]
pub enum RefError { #[error("unknown sheet {0:?}")] UnknownSheet(String), #[error("invalid reference {0:?}")] Invalid(String),
                    #[error("reference out of bounds")] OutOfBounds, #[error("range too large")] TooLarge, #[error("unsupported reference form")] Unsupported }
#[derive(thiserror::Error, Debug)] #[non_exhaustive]
pub enum EditError { Ref(#[from] RefError), InvalidInput { cell: CellRef, message: String }, SheetName(String), LastSheet, Engine(String) }
pub enum LoadError { Io(std::io::Error), Format(String), Options(String) }
pub enum SaveError { Io(std::io::Error), Engine(String) }
```
Engine `String` errors are classified at the facade boundary (never exposed raw except in the
`Engine` catch-all variants). An Excel error *result* (`=1/0`) is never an `Err`.

### Python mapping (T-0104)
```python
import rustsheet as rs
wb = rs.Workbook.open("model.xlsx")          # also rs.open(...), rs.Workbook()
wb["Sheet1!A1"]                              # Value -> float | str | bool | None | rs.XlError
wb.values("Sheet1!A1:C3")                    # list[list[...]]
with wb.edit() as tx:                        # commit on clean exit, rollback on exception
    tx["Sheet1!A1"] = 10                     # Input::Parse semantics; rs.Formula("=A1*2") forces formula
wb.changes                                   # Python-only convenience: ChangeSet of the last edit (the Rust Workbook stores none)
```
`XlError` is a returned value type (equality by kind, `str()` gives the literal), not an
exception. `RefError`/`EditError`/`LoadError` map to `rs.RefError`, `rs.EditError`,
`rs.LoadError` (all subclass `rs.RustSheetError`). `Workbook.open`, `edit` exit and `save_xlsx`
release the GIL. The existing `Model`/`UserModel` bindings stay until pytest parity (T-0104).

## Alternatives considered
- **Wrap `UserModel`:** brings selection/clipboard/undo UI concerns and its own diff queue; rejected (api-design principle 7).
- **Positional sheet indexes:** simpler, but unstable across delete/reorder and unsafe for bindings; rejected.
- **Skip the facade, fix `Model` in place:** the fork should keep tracking upstream (`UPSTREAM.md`); a thin facade confines API churn.
- **Incremental `ChangeSet` now:** needs engine dirty tracking (M3); diffing gives the same contract earlier.

## Test plan
- Unit tests: A1 parse/print round-trips (quoted sheet names, `$`, whole row/col, bounds);
  `ErrorKind` ↔ literal round-trip for every engine error (built on the engine's existing literal and ERROR.TYPE tables, with a sync test; no second table); value conversion from every `Cell` variant.
- Atomicity: a failing op mid-`edit` leaves values, formulas and sheet list byte-identical.
- Property test (proptest): random edit sequences — `ChangeSet.values` equals the full
  before/after diff, and applying `after` values to a fresh copy is consistent with recalc.
- Eval: `RustSheetEngine` switches to the facade (T-0102/T-0103); `make eval` results must be
  unchanged (0 regressions, 0 fixes).
- Python: new pytest suite for the facade (T-0104), GIL-release smoke test.
- Performance: T-0103 bench (≥100k formulas): one-cell edit and failed-edit rollback, numbers recorded.
- Untrusted input: deeply nested formulas via `Input::Parse`/`Formula` return `EditError` through the engine's parser limits, never panic; `values()` over-limit fails before allocating; fuzz-ish tests for malformed references and huge ranges (`TooLarge`, no allocation blowup).

## Rollout
| Task | Lane | Size | Content |
|---|---|---|---|
| T-0101 | engine | M | Remove `Model` lifetime (prerequisite) |
| T-0102 | api | M | Crate `rustsheet`: refs, `Value`/`XlError`, `Workbook` open/new/save/read; eval adapter reads via facade |
| T-0113 | api | S | `ErrorKind`/`XlError` conversions and literal round-trip; `RefError` classification of engine errors (split out of T-0102 to keep it M) |
| T-0103 | api | M | `edit`/`set`, `Txn`, rollback, diff-based `ChangeSet`; eval adapter uses `edit` |
| T-0104 | api | M | PyO3 bindings on the facade |

T-0113 is a hard prerequisite of T-0102. The old `Model` bindings remain until T-0104 reaches parity.
