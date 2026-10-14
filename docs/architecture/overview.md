# Architecture overview

## Today (fork point, M0)

```
                 ┌──────────────────────── bindings/python (PyO3, `import rustsheet`)
                 │
crates/rustsheet-xlsx ── import/export xlsx ──┐
                                              ▼
crates/rustsheet-engine (forked IronCalc base)
  expressions/  lexer → parser → AST (R1C1-normalized, shared per sheet), static analysis
  evaluation.rs recursive pull evaluator; full re-evaluation on every evaluate()
  functions/    496 Excel functions by category
  model.rs      Model<'a>: workbook + parsed formulas + evaluation state
  sheet_data.rs 1024-row chunked sparse storage; shared strings; styles
  user_model/   UI-oriented API (undo/redo, clipboard, autofill) — not part of the headless target

crates/rustsheet-eval   harness: suites → engine adapter → strict comparison → report / baseline
tools/ (Python)         oracle backends, spec→xlsx, generator   (call rustsheet-eval for parsing)
```

Key facts (details in `docs/research/ironcalc-assessment.md`):
- No dependency graph: recalculation is O(all cells).
- Values are a mix of typed cells and strings, and errors are flattened to strings in `CellValue`.
- `Model<'a>` carries a lifetime that leaks memory in the bindings.

## Target (M3–M4 and beyond)

```
rustsheet (public facade crate, M1)          stable API: Workbook / Txn / ChangeSet / Snapshot / Value
  └─ rustsheet-engine
       ├─ value      Value, ErrorKind{detail,origin}, Array (dense+fill), coercion rules (ONE place)
       ├─ formula    lexer, parser, AST, R1C1 normalization, printer, reference math (RefMap)
       ├─ storage    chunked columnar COW blocks; interned strings; style table; formula families
       ├─ graph      dependency graph: cell/range(interval index)/name vertices, dirty sets,
       │             volatile & dynamic deps, cycle detection, topo levels
       ├─ calc       scheduler: Automatic/Manual/OnDemand, incremental + full, iterative calc,
       │             parallel levels, cancellation; evaluator (pull-based, reused from fork)
       ├─ functions  registry: one implementation per Excel function; shared arg iteration,
       │             criteria/wildcard matcher, numeric kernels (distributions, solvers)
       └─ format     number-format engine, formatted text
  rustsheet-xlsx     streaming import, export, unknown-part passthrough
  rustsheet-ffi      flat C/UniFFI facade (M12) → Java
  bindings/python    PyO3 over the facade (bulk, GIL release)
```

### Design rules
1. **Values vs failures:** Excel errors are values; Rust `Err` is only for API misuse and refused edits.
2. **One definition per concept:** coercion, aggregate-argument iteration, criteria matching,
   reference adjustment and float formatting each have exactly one implementation.
   Aggregate-argument iteration is the walker of
   [SPEC-0009](../specs/SPEC-0009-aggregate-argument-walker.md).
3. **Laziness:** nothing is computed unless observed (OnDemand) or required by Excel's automatic
   mode. The evaluator is pull-based, so laziness is natural; the graph supplies dirtiness.
4. **Sharing:** repeated text, styles and formulas are stored once (interning plus R1C1 families).
   Snapshots share storage chunks copy-on-write.
5. **Structural edits** produce a `RefMap`; formulas are rewritten on the AST, never by
   re-parsing text.
6. **Determinism:** clock, RNG seed and timezone are injected (`WorkbookOptions`), so volatile
   functions are testable.

The migration path is incremental and test-guarded. Every step keeps `make eval-check` green; the
full-recalc path stays as the reference implementation for metamorphic tests (incremental ==
full) until M11.
