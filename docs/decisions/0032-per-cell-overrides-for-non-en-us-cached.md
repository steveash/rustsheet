# 0032 — Per-cell overrides for non-en-US cached xlsx values

- **Status:** accepted (2026-10-05, T-0024)
- **Context:**
  - Decision 0030 rule 2: cached values from a workbook saved under a non-en-US locale are not
    evidence for locale-dependent behavior (date and number text); the en-US oracle wins there.
    It gave no way to *score* that. A sidecar replaces a workbook's cached values wholesale, and
    `rustsheet-oracle fill` drops cells where web and desktop disagree (decision 0019), so the
    cells in question would end up unscored.
  - Calibration (docs/research/oracle-calibration.md) found 7 such cells in ironcalc-calc,
    which was saved by a day-first Excel: `DATEVALUE("26-01-01")` is cached as 36917, "July 1,
    2026" as #VALUE! in DAYS/DAYS360/NETWORKDAYS(.INTL), and `VALUE("2/3/2023")` and
    `VALUE("10-12-1999")` as day-first serials. probes/date-text-locale holds the same formulas
    on the same inputs, answered by the en-US oracle.
  - Fixing the engine to en-US (T-0024) turns the 5 cells that pass today into baseline
    "regressions" and makes `rustsheet_xlsx`'s generated test fail for 3 workbooks. Accepting
    regressions (`--allow-regressions`) or deleting the workbooks' tests would hide real
    regressions in the other ~600 cells of those workbooks.
- **Decision:** An xlsx suite may name a `cached_overrides` file (suite.toml). Each line
  replaces the expectation of one scored cell with one cell of a committed `excel-graph` or
  `excel-desktop` oracle sidecar (a probe computing the same formula on the same inputs). The
  harness errors if the cell is not scored, the sidecar lacks the cell, or the sidecar is not
  from a live Excel, so an override can never carry a hand-written value. `rustsheet_xlsx`'s
  generated tests skip comparing exactly those cells (build.rs reads the same file); the eval
  gate scores them.
- **Consequences:**
  - ironcalc-calc gets `eval/suites/ironcalc-calc/cached-overrides.tsv` with the 7
    calibration cells. The engine follows en-US for them, they pass against en-US values, and
    the two VALUE cells (`T_VALUE_VALUETOTEXT.xlsx` C15/I5) leave the known-failure list.
  - An override is only for evidence that decision 0030 rule 2 disqualifies (non-en-US locale
    for locale-dependent behavior), or for a later decision that says so. The PR adding one cites
    the calibration or probe that shows the cached value is not en-US evidence. It is never a
    way to make a failing cell pass: the replacement is a golden that the engine must match.
  - Reviewers check that the evidence cell computes the same formula on the same inputs; the
    harness cannot check that mechanically.
  - This record does not change baseline semantics: `make eval-baseline` still refuses
    regressions.
