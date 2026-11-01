# SPEC-0026 — The Python package's inherited IronCalc surface

- **Status:** accepted
- **Milestone / lane:** M12 / api (with engine and harness tasks)
- **Tasks:** T-2882 (this spec); T-2913, T-2895, T-2896, T-2897, T-2898 (implementation)

## Problem
T-0104 put the Python binding on API v0 "keeping the old Model bindings until parity", and nothing has revisited that
since. `import rustsheet` exports two surfaces side by side (bindings/python/python/rustsheet/__init__.py):

- the facade (`Workbook`, `Transaction`), which goes through `crates/rustsheet` with its typed errors and load limits;
- IronCalc's raw `Model` and `UserModel` classes with their loaders, which reach the engine directly.

The README (bindings/python/README.md) is still IronCalc's ("pip install ironcalc"). It presents `UserModel` and `Model`
as *the* two APIs and does not mention `Workbook`.

The legacy surface breaks directive 5 (untrusted input never reaches a panic) in three places:

1. **Decoded bytes reach panics.** `load_from_bytes` and `create_user_model_from_bytes` (bindings/python/src/lib.rs),
   and `UserModel.from_bytes` (user_model.rs), run `bitcode::decode` on a `Workbook` and pass it to
   `Model::from_workbook` (crates/rustsheet-engine/src/model.rs). That function checks no index invariant, and neither
   do the icalc file loaders (`load_from_icalc`, `create_user_model_from_icalc`, `UserModel.from_icalc` →
   `rustsheet_xlsx::import::load_from_icalc`). Reproduced on `main` at 7bd9090e (T-2882): encode a one-cell workbook,
   change the bytes, decode, `Model::from_workbook`, then:
   - **R1:** cell style index 9999, then `set_user_input(0,1,1,"y")` panics at crates/rustsheet-engine/src/styles.rs:478
     (`style_is_quote_prefix` indexes `cell_xfs` unchecked);
   - **R2:** a `CellFormula { f: 77 }` with no shared formulas, then `evaluate()`, panics at
     crates/rustsheet-engine/src/evaluation.rs:819 (index out of bounds, len 0, index 77).

   PyO3 turns these into `PanicException`, but directive 5 forbids the panic itself. `get_style` (styles.rs) also
   indexes `fills`, `fonts` and `borders` unchecked. No fuzz target decodes bitcode: the targets are
   fuzz/fuzz_targets/{formula_eval,formula_parse,xlsx_import}.rs.
2. **External diffs.** `UserModel.apply_external_diffs` (user_model.rs → `UserModel::apply_external_diffs`,
   crates/rustsheet-engine/src/user_model/common.rs) decodes peer-supplied bitcode `Vec<QueueDiffs>` and applies each
   `Diff` as it is.
3. **A shipped panic.** `test_panic` is a public `#[pyfunction]` (`#[allow(clippy::panic)]`, lib.rs), exported in
   `__all__` and stubbed in `_ironcalc.pyi`; `Model.test_panic` (raw_model.rs) is a second copy. No Python test
   calls either.

The legacy classes also skip what the facade enforces at load (`LoadOptions`: the read-cell cap `max_read_cells`, typed
errors, and the per-workbook clock of SPEC-0008 at construction). Their xlsx loaders do go through the fuzzed importer.
This gap is accepted until the legacy module is removed at the v1 freeze: `rustsheet.legacy` is an explicit,
deprecated opt-in, its import warning says it skips the facade's load limits, and the supported path
(`Workbook.open`/`from_bytes`) has them.

## Goals / non-goals
Goals:
- No untrusted bytes reach a panic through the Python package. Untrusted bytes means any workbook bytes, icalc file or
  diff list.
- One documented public surface in `import rustsheet`: the facade. The legacy classes stay reachable, but only as an
  explicit, deprecated opt-in, until each feature they carry is moved to the facade or dropped.
- Defense in depth in the engine: the Rust decoders that stay (`Model::from_bytes`, `UserModel::from_bytes`,
  `load_from_icalc`, `apply_external_diffs`) return `Err` on crafted input and are fuzzed.

Non-goals:
- Implementing any of this (tasks below).
- New facade features (styles, undo/redo, CF, clipboard; SPEC-0001 non-goals). The feature table below only says
  where each one goes.
- Java/FFI and the API v1 semver freeze (other M12 work).
- `Model::from_bytes` callers inside the Rust crates that only read bytes the engine wrote. They are covered by T-2913's
  check anyway.

## Excel behavior to match
None: this spec changes no calculation. The facade's obligations stay as SPEC-0001 states them.

## Design

### Decision 1. Are icalc/bitcode bytes and external diffs untrusted? Yes, both.
Directive 5 makes workbooks untrusted, and an icalc file or a bitcode buffer is a workbook in another encoding. A
Python caller cannot tell an engine-written buffer from a crafted one any better than an xlsx file from a crafted one,
and the binding has no integrity check (no MAC, no version guard beyond bitcode's own framing). A diff list comes from
a peer over a channel the engine does not control (the collaboration mechanism exchanges diffs between users' models,
docs/examples/collaboration.py), so it is as untrusted as a workbook. Neither may reach a panic. The binding resolves
this by not accepting such bytes (decision 2). The engine resolves it by validating them (decisions 3 and 4), because
Rust callers keep the decoders.

### Decision 2. Symbol table
Every name `rustsheet/__init__.py` exports, plus the legacy methods that take bytes or panic:

| symbol | decision | reason | task |
|---|---|---|---|
| `Workbook`, `Transaction`, `Formula`, `XlError` | keep (the API) | the facade: typed values and errors, load limits | — |
| `RustSheetError`, `WorkbookError`, `LoadError`, `SaveError`, `EditError`, `RefError` | keep | the facade's typed errors | — |
| `open` | keep | shorthand for `Workbook.open` | — |
| `__version__`, `column_name_from_number`, `column_number_from_name`, `quote_name`, `get_all_timezones`, `get_supported_locales` | keep | pure helpers, take no workbook bytes | — |
| `load_from_bytes`, `create_user_model_from_bytes`, `UserModel.from_bytes` | remove | decode bitcode into an unchecked `Model` (R1, R2); the facade reads xlsx bytes (`Workbook.from_bytes`) | T-2897 |
| `load_from_icalc`, `create_user_model_from_icalc`, `UserModel.from_icalc` | remove | same decoder over a file | T-2897 |
| `Model.save_to_icalc`, `Model.to_bytes`, `UserModel.save_to_icalc`, `UserModel.to_bytes` | remove | writers with no reader left in the package; xlsx is the exchange format | T-2897 |
| `UserModel.apply_external_diffs`, `UserModel.flush_send_queue` | remove | peer bytes applied unchecked; collaboration belongs to the `UserModel` UI layer, which SPEC-0001 keeps out of the facade | T-2897 |
| `test_panic`, `Model.test_panic` | remove | a shipped panic that no test calls; PyO3's panic translation needs no test of ours | T-2897 |
| `Model`, `create`, `load_from_xlsx` | move to `rustsheet.legacy` (deprecated) | the raw API carries features the facade lacks (table below); its xlsx loader is the fuzzed importer | T-2898 |
| `UserModel`, `create_user_model`, `create_user_model_from_xlsx` | move to `rustsheet.legacy` (deprecated) | same, plus undo/redo, CF, clipboard and named styles | T-2898 |
| `CellType` | move to `rustsheet.legacy` | only the legacy `get_cell_type` returns it | T-2898 |

Removing the byte loaders also removes the panics from the Python package (R1 and R2 become unreachable from Python).
Decisions 3 and 4 still make them `Err` in the engine.

Kept legacy methods (`set_user_input`, `get_cell_value`, row and column ops and so on) take indices and text, not
workbook bytes. Their arguments are untrusted too. They reach the engine's ordinary argument checks, which return
`Err` for a bad sheet index. They are not in this spec's scope beyond that note: they go with the classes at the v1
freeze.

### Decision 3. Engine: `Model::from_workbook` validates (T-2913)
`Model::from_workbook` runs one linear pass over the decoded `Workbook` before it builds anything, and returns `Err` on:

- a cell, row or column style index outside `styles.cell_xfs`;
- a `cell_xfs` entry whose font, fill, border or number-format id dangles;
- a shared-string index outside `shared_strings`;
- a `CellFormula { f }` outside its sheet's `shared_formulas`;
- a row or column outside the grid;
- a sheet id or index (defined names, the selected sheet, views) that names no sheet;
- a CF dxf id outside `styles.dxfs`.
- a decoded count or dimension (sheets, rows, columns, cells per row, shared strings) past the engine's grid limits,
  checked before anything is allocated from it, so a crafted buffer cannot request a huge allocation.

As built (T-2913): only indices some read trusts unchecked are refused. A shared-string index, a CF dxf id, a defined
name's sheet id, a view's selected sheet, the frozen panes and a workbook with no sheets are read through checked
lookups that tolerate them (empty text, no format, a global name, sheet 0), so refusing them would only stop
workbooks that load today (poi/sample-beta.xlsx, an Office 2007 beta package, imports with no shared strings). The check runs on the decoded `Workbook`, so the
bitcode decoder has already allocated what the bytes encode (bounded by the input's size); what the check guarantees
is that nothing the model builds afterwards (row chunks, spill ranges, merged ranges, column records) is sized from a
count past the grid.

The validator's doc comment lists the invariants, so that what it misses is the fuzz target's to find. Because the
check sits in `from_workbook`, it covers every decoder: `Model::from_bytes`, `UserModel::from_bytes`,
`rustsheet_xlsx::import::load_from_icalc`, and the xlsx importer's final step. An xlsx-built workbook that fails it was
a latent panic too.

Alternative rejected: making every reader bounds-checked (`get` instead of `[]` in styles.rs, evaluation.rs and the
others). The readers are many and spread out, a missed one is still a panic, and the cost lands on every read instead
of once per load.

### Decision 4. Engine: diffs are validated before any applies (T-2896)
`UserModel::apply_external_diffs` checks every decoded `Diff` against the model's current sheets, styles and grid
before applying the first one, and returns `Err` with the model unchanged on any dangling index. It reuses T-2913's
index checks. The Python package no longer calls it (decision 2), but the Rust API keeps it for the web UI layer.

As built (T-2896): each diff is checked against the model as it is when that diff applies, not before the first one,
because a list can depend on itself (it can add the sheet a later diff edits, or create the style a later undo
restores). "Unchanged" comes from a copy of the workbook: the model is rebuilt from it when any diff is refused. The
structural actions and the range clears also refuse counts and areas off the grid, which overflowed in `i32`, and
an area costs the cells it holds, not its size.

### Decision 5. Fuzzing (T-2895)
A new target, fuzz/fuzz_targets/bitcode_workbook.rs, decodes arbitrary bytes through `Model::from_bytes` and, when the
model loads, runs `evaluate()` and a few reads and edits. Its seed corpus is `to_bytes()` of small engine-written
workbooks, because random bytes rarely decode. It joins the M12 exit criterion "fuzzers run 24 h with no crash".

### Decision 6. What the facade must gain before the legacy module goes
The legacy module is removed at the API v1 freeze (M12). Each feature it carries is either moved to an M12 facade
task first or dropped:

| legacy feature | facade today | decision |
|---|---|---|
| cell input, values, formulas, formatted values | `Workbook.set/value/formula/formatted` | covered; drop the legacy copy |
| xlsx load and save | `Workbook.open/from_bytes/save_xlsx/to_xlsx_bytes` | covered |
| sheets: add, rename, delete | `Transaction.add_sheet/rename_sheet/delete_sheet` | covered |
| sheet move, hide, color, grid lines | Rust `Txn::move_sheet` only | move: expose `move_sheet` in Python (M12 facade task); drop hide, color and grid lines (view state) |
| insert and delete rows and columns | Rust `Txn::insert_rows/delete_rows/insert_columns/delete_columns` | move: expose in Python (M12 facade task) |
| hidden rows; number formats | Rust `Txn::set_row_hidden/set_number_format` | move: expose in Python (M12 facade task) |
| defined names | Rust `Txn::define_name/define_sheet_name` | move: expose in Python (M12 facade task) |
| row heights, column widths, frozen panes, selection | none | drop (view state, not calculation) |
| cell styles, named styles, borders, themes | none (SPEC-0001 non-goal) | keep in legacy until v1, then drop unless a styles spec lands first |
| conditional formatting | none | keep in legacy until v1, then drop unless a spec lands first |
| undo/redo, clipboard, CSV paste, auto-fill | none (UI layer) | drop with the module |
| hyperlinks | none | drop with the module |
| collaboration diffs | none | removed from Python now (decision 2) |

The "move" rows become M12 api tasks when the legacy module's removal is scheduled. This spec does not file them
because each depends on the Python facade's API review at the v1 freeze.

### Decision 7. Deprecation path
The package is pre-v1 (0.1.0), so breaking changes are allowed:

- **T-2897** removes the byte loaders, the diff methods and `test_panic` with no warning release. A security fix
  should not wait a release, and a caller has the xlsx path.
- **T-2898** moves the classes to `rustsheet.legacy`. Importing that module raises a `DeprecationWarning` that names the
  v1 freeze as the removal point. This is the only warning step.
- The README is rewritten around `rustsheet.Workbook` in T-2898. `_ironcalc.pyi` is renamed to match the extension
  module (`_rustsheet.pyi`), loses the removed symbols, and its legacy part moves under the `legacy` module's stubs.

## Alternatives considered
- **Keep the bitcode/icalc loaders and harden them (decision 3 alone).** Rejected for Python: it keeps a second,
  undocumented exchange format whose only Python use is a round-trip test, and it ties the package's safety to the
  completeness of one validator. The validator still lands (T-2913), as defense in depth for Rust callers.
- **Move the legacy classes behind a non-default cargo feature.** Rejected: a wheel is built once for every user, so a
  feature-gated class means a second wheel or a source build. The `legacy` submodule gives the same opt-in with one
  artifact.
- **Remove the legacy classes now.** Rejected: 111 of the 149 Python tests (all but test_facade.py) and six of the
  seven examples use them, and the facade lacks the features in the table above. Removal waits for v1.

## Test plan
- T-2913: R1 and R2 as Rust unit tests that get `Err` from `Model::from_workbook`, never a panic, plus one test per
  other invariant. The xlsx and icalc round-trip tests and the eval suites stay green.
- T-2895: `bitcode_workbook` runs at least 10 minutes locally with no crash, seeded with engine-written workbooks.
- T-2896: a crafted diff list with a dangling style index and one with a dangling sheet index each get `Err`, and the
  model is unchanged.
- T-2897: a Python test asserts that each removed name is not importable. The remaining tests pass.
- T-2898: a Python test asserts `rustsheet.__all__`, and another that `import rustsheet.legacy` warns.

## Rollout
1. T-2897 (api, S): drop the byte loaders, the diffs and `test_panic` from Python. This closes the Python exposure.
2. T-2913 (engine, M): validate in `Model::from_workbook`.
3. T-2895 (harness, S, after T-2913) and T-2896 (engine, M, after T-2913).
4. T-2898 (api, M, after T-2897): the `legacy` module, the deprecation warning, and the README.
5. At the API v1 freeze: file the "move" rows of the feature table as facade tasks, then remove `rustsheet.legacy`.
