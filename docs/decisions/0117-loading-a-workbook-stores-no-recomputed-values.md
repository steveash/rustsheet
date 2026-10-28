# 0117 — Loading a workbook stores no recomputed values

- **Status:** accepted (2026-10-25, T-0468)
- **Context:** Conditional formats used to be evaluated at load (`Model::from_workbook*`,
  new_empty.rs). Their rules read cells through `evaluate_cell` outside a pass, so loading
  evaluated the formula cells those rules read and stored the results. A value read after load
  could therefore be a recomputed one rather than the file's cached one, and `evaluate()` started
  from those recomputed values. SPEC-0021 moves conditional-format evaluation to the first style
  read (corpus-bench 41978: load 184 s -> 5 ms).
- **Decision:** Loading only parses. Every value read after load, until `evaluate()` or a
  conditional-format style read, is the value the file stores, which is what a workbook opened
  without recalculation shows. `evaluate()` starts from those stored values, including for the
  results that keep the values they start from (cycle members, the start of iterative
  calculation).
- **Consequences:**
  - Tests that compare a loaded model with an evaluated one now compare against the file's cached
    values for every cell, including the cells conditional formats read.
  - `rustsheet_xlsx::compare` pins a workbook without a METADATA timestamp to its last-saved time
    (`dcterms:modified`), so `TODAY()` and `NOW()` give the values Excel cached (templates/wordle.xlsx
    `Wordle!N2`).
  - The gated eval suites show no change from this (T-0468 Outcome).
