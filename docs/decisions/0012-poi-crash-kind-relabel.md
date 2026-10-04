# 0012 — Relabel four POI crash baseline lines (Panic → typed error)

- **Status:** accepted (2026-10-04)
- **Context:** T-0118 removes every importer panic on untrusted workbooks. Baseline lines for
  crash-only cases carry the crash kind (`!Panic`, `!EngineError`, `!InvalidCase`), so a case
  that stops panicking and fails with a typed error shows up as a "regression" (new line) plus a
  "fix" (old line gone). Four POI cases are genuinely malformed and have no Excel evidence that
  they should load, so they stay failing, now as typed errors: `47889.xlsx` (no `cellXfs`),
  `60825.xlsx` (empty `r:id` on a sheet), `invalid_cell_reference.xlsx` (cell ref `B`), and
  `poc-shared-strings.xlsx` (shared string over 32,767 chars, `!InvalidCase`). Every other
  formerly crashing case either loads now or was already a typed error.
- **Decision:** Run `rustsheet-eval update-baseline --allow-regressions` once, for exactly these four
  lines. Each was already a known failure (`!Panic`); no passing cell is lost and no cell that
  passed before fails.
- **Consequences:** `eval/baseline/poi.txt` shrinks (fewer crash lines) and no `!Panic` remains.
  Reviewers confirm the diff to the baseline contains only these relabels and removals.
