# 0062 — Evidence-floor ratchet in quality/evidence-floor.json

- **Status:** accepted (2026-10-15, T-0056)
- **Context:** The owner asked (2026-10-15) that Phase 1's test suite be finished before the
  Phase 2 re-architecture (decision 0027), and that no in-scope function rest on thin Excel
  evidence. On main, 73 in-scope catalog functions the oracle can evaluate have fewer than 20
  Excel-evidence cells (oracle sidecar or Excel-cached values), from GETPIVOTDATA (1) to
  QUARTILE.EXC (19). The report listed thin functions at 25 cells (T-0020) but nothing kept the
  number from growing.
- **Decision:**
  - The evidence floor is 20 Excel cells (`EVIDENCE_FLOOR` in crates/rustsheet-eval/src/
    evidence_floor.rs). It replaces the report's earlier 25-cell threshold, so the report has one
    definition of "thin".
  - `quality/evidence-floor.json` locks `{floor, below_floor}`. `rustsheet-eval evidence-floor`
    (`make evidence-floor`, part of `make check` and CI's eval job) counts the in-scope,
    oracle-evaluable catalog functions below the floor over the vendored suites only, so a
    fetched external suite (poi) cannot move the count between machines. It fails when the count
    rises, when it falls without the lock being updated, and when the lock was taken at another
    floor.
  - `make eval-baseline` also runs `evidence-floor --update`, which records a drop and refuses a
    rise. A rise is recorded only with `--update --allow-increase` under its own decision record
    (for example, a catalog change bringing functions into scope). The file is never hand-edited.
  - The catalog view `eval/catalog/scope.json` gains each function's `category`, which groups
    the report's "Below the evidence floor" list.
- **Consequences:** A PR that removes Excel-evidence cells or brings a function into scope
  without probes fails CI. Probe PRs run `make eval-baseline` and commit the lower lock. The 73
  functions are filed as research-lane probe batches T-1620..T-1628 (≤ 10 functions each).
  `make check` runs the vendored suites a second time (about 15 s locally).
