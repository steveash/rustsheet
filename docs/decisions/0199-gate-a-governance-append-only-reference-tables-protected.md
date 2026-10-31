# 0199 — Gate A governance: append-only reference tables, protected exempt and R3 lists, known-failure hygiene

- **Status:** accepted (2026-10-31, T-3379; implements SPEC-0029 "Gate A holds" and "The check" steps 4–5)
- **Context:** Gate A (decision 0188 R2, SPEC-0029) checks the engine against committed reference tables
  under `eval/accuracy/`. Its rules are only as strong as the files they read: a batch that is edited, an
  exemption that hides a function from the missing-table count, an R3 mark that lets a point ignore Excel,
  or a known-failure line that carries a regression of a committed point would each weaken the gate with
  no ratchet to catch it. SPEC-0029 assigns these mechanical rules to T-3379, since a reviewer cannot be
  trusted to apply them every time.
- **Decision:** `scripts/check_governance.py` (via `scripts/accuracy_governance.py`) treats a change,
  deletion or rename of a committed file under `eval/accuracy/tables/` and any change to
  `eval/accuracy/exempt.tsv` or `eval/accuracy/math-over-excel.tsv` as a protected-path change (needs a
  decision record; adding a new batch's files does not), and fails a diff in which a `known-failures.tsv`
  line names a task that does not exist or is `done`, or a newly listed point whose batch the same diff
  does not add. `rustsheet-eval accuracy` reads `exempt.tsv` (`<FUNCTION>\t<exact|composite>\t<reason>`),
  prints the missing-table count (in-scope catalog functions with `returns: number` and neither a table
  directory nor an exemption; the catalog view `eval/catalog/scope.json` now carries `returns`), fails on a
  stale exemption (a function with a table, or not an in-scope numeric function), and writes `holds` to its
  JSON. A missing table does not fail the check; it keeps Gate A from holding (R1(a)).
- **Consequences:** Table batches are append-only: a corrected batch is a new batch, and refilling or
  replacing a committed one (`fill --force`, a definition change) needs a decision record. Each new
  exemption or R3 mark needs a decision record; a new R3 case also needs an owner ruling (decision 0188
  R3). A known-failure line goes stale for governance as soon as its owner task is marked `done`, so the
  task that fixes the points removes their lines in the same PR. `eval/accuracy/**` and the generator in
  `tools/src/rustsheet_tools/accuracy/**` are code for `scripts/ci_changes.py`, so they run the eval job
  (tested in `scripts/test_ci_changes.py`). The first `exempt.tsv` lists 41 `exact` functions (counts,
  positions, codes, integer date parts, bit operations); `composite` exemptions come with the table slices
  of T-3358.
