---
id: T-2882
title: "Spec: the Python package's inherited IronCalc surface (raw Model, UserModel, icalc/bytes loaders, external diffs, test_panic) under directive 5"
milestone: M12
lane: api
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A spec decides what happens to the Python package's IronCalc-era API that bypasses the facade: the raw `Model`
class, the `UserModel` class, their `load_from_bytes`/`load_from_icalc`/`create_user_model_from_bytes`/
`create_user_model_from_icalc` loaders, `UserModel.apply_external_diffs`, and the shipped `test_panic` function.
Each one is either retired, put behind an opt-in, or hardened so that untrusted bytes cannot reach a panic
(directive 5). The spec also names the implementation tasks. The spec is a new SPEC under docs/specs/ (the next
free number) or a SPEC-0001 amendment.

## Context
T-0104 put the Python binding on API v0 "keeping the old Model bindings until parity", and no task has revisited
that since. `import rustsheet` still exports both surfaces (bindings/python/python/rustsheet/__init__.py). The
README (bindings/python/README.md) is still the IronCalc one ("pip install ironcalc") and presents `UserModel` and
`Model` as *the* two APIs. It does not mention the facade `Workbook`.

What the legacy surface does that the facade does not allow:
- **Decoded bytes reach panics.** `load_from_bytes` and `create_user_model_from_bytes` (bindings/python/src/lib.rs)
  `bitcode::decode` a `Workbook` and hand it to `Model::from_workbook` (crates/rustsheet-engine/src/model.rs). That
  function checks no index invariant, and neither do the icalc file loaders (`rustsheet_xlsx::import::load_from_icalc`).
  Reproduced on `main` at 7bd9090e with a scratch binary: encode a one-cell workbook, change the bytes, decode,
  `Model::from_workbook`, then:
  - cell style index 9999, then `set_user_input(0,1,1,"y")`: panics at crates/rustsheet-engine/src/styles.rs:478
    (`style_is_quote_prefix` indexes `cell_xfs` unchecked);
  - a `CellFormula { f: 77 }` with no shared formulas, then `evaluate()`: panics at
    crates/rustsheet-engine/src/evaluation.rs:819 (index out of bounds, len 0, index 77).
  PyO3 turns these into `PanicException`, but directive 5 forbids the panic itself. `get_style` (styles.rs) also
  indexes `fills`/`fonts`/`borders` unchecked. The xlsx importer and its fuzz target (fuzz/fuzz_targets/xlsx_import.rs)
  do not cover this path, and no fuzz target decodes bitcode.
- **External diffs.** `UserModel.apply_external_diffs` (bindings/python/src/user_model.rs:95 →
  `UserModel::apply_external_diffs`, crates/rustsheet-engine/src/user_model/common.rs) decodes peer-supplied bitcode
  `QueueDiffs` and applies them as they are. The spec decides whether these bytes count as untrusted input.
- **No facade limits.** The legacy classes skip what `WorkbookOptions`/`LoadOptions` enforce in the facade: the
  read-cell cap (`max_read_cells`), typed errors, and the per-workbook clock option (SPEC-0008) at construction.
- **`test_panic`** is a public `#[pyfunction]` (`#[allow(clippy::panic)]`, lib.rs) exported in `__all__` and
  stubbed in `_ironcalc.pyi`. It is in the release module only so that a test can call it.
- Size: bindings/python/src/user_model.rs (1,049 lines) is on the oversized-files list (quality/oversized-files.json).
  raw_model.rs is 643 lines. Tests and examples use the legacy API: bindings/python/tests/test_{io,diffs,errors,
  rows_columns,conditional_formatting,...}.py and bindings/python/docs/examples/*.py.

Questions the spec answers:
1. For each legacy entry point: remove it, move it behind a non-default cargo feature or a `rustsheet.legacy`
   submodule, or keep it. If it is kept, how is it made panic-free on crafted bytes? (Options: a validating
   `Workbook` check after decode that `Model::from_workbook` runs, giving `Err` on any dangling style, shared-string,
   formula, sheet or row/column index; or refusing bitcode/icalc input from Python altogether.)
2. Whether icalc/bitcode bytes and external diffs are untrusted input under directive 5, with the reasoning written
   down, and the fuzz target that covers the kept decoders (a `bitcode_workbook` target in fuzz/).
3. What the facade or the Python `Workbook` must gain before a legacy feature can go: styles, undo/redo, CF,
   clipboard and diffs are SPEC-0001 non-goals. List each one as kept-legacy, moved to an M12 facade task, or
   dropped.
4. Where `test_panic` goes (a test-only build of the module, or removal together with its test).
5. The README and `.pyi` changes, and the deprecation path for Python callers (pre-v1, so a breaking change is
   allowed; say whether there is a warning release first).

## Acceptance criteria
- [ ] Spec merged that answers questions 1–5 for every legacy symbol exported by `rustsheet/__init__.py`, listed in
      a table (symbol, decision, reason). It cites the two reproductions above, and its test plan includes a Rust
      test for each one: the crafted style index and the dangling formula index give `Err` or are unreachable, never
      a panic.
- [ ] The spec names its implementation tasks, filed with `make new-task` (api lane for the binding, engine lane
      for any `Model::from_workbook` validation, harness lane for a new fuzz target). Each is size S/M with
      checkable criteria, and the decoder-validation task carries the two reproductions as tests.
- [ ] docs/architecture/api-design.md "Phasing" or SPEC-0001 points to the new spec for the Python legacy surface.
- [ ] No code changes; `make check` green; reviewers (including reviewer-architecture and reviewer-security) approve.

## Out of scope
- The implementation: removing classes, adding validation, adding fuzz targets (the tasks the spec files).
- Java/FFI bindings and the API v1 semver freeze (other M12 work).
- `Model::from_bytes` callers inside the Rust crates that only read bytes the engine wrote itself (the spec may
  mention them, but this task does not change them).
