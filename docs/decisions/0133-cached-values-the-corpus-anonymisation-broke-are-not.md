# 0133 — Cached values the corpus anonymisation broke are not Excel evidence

- **Status:** accepted (2026-10-27, T-2867). It adds a third class of `cached_overrides` line under
  decision 0032 ("or for a later decision that says so"), after decision 0090. No baseline line is
  hand-edited and no gate is weakened.
- **Context:**
  - The calipers `pv_goldens` workbooks were anonymised by their authors before Calipers imported them
    (eval/suites/calipers/suite.toml). The anonymisation rewrote cell text and formulas in the saved
    files without recalculating, so some cached values no longer belong to the workbook as shipped.
  - `pv26_020` and its byte-identical twin `pv26_2_020`, sheet Summary: the text in B17 was renamed to
    `Company0691`, but the sheet it named kept its name (`ConAgra`, whose sheet-scoped `TargetName`
    holds the cached `Company2277`). The shipped workbook has no `Company0691` sheet, so C17:O17
    (`=INDIRECT($B17&"!TargetName")` and the like) and M23:O26 (`=MAX/AVERAGE/MEDIAN/MIN(M7:M21)`) cache
    the pre-anonymisation numbers and texts, while Excel computes `#REF!` for them (25 cells each).
  - `pv26_010` Cover!C16 is stored as `<f ca="1">45678</f>` with no `<v>`: a volatile formula was
    replaced by a literal and its value dropped, so the expectation is empty while Excel computes 45678.
  - T-2660 found the cause and left the cells "not actionable". They are the only visibly-off attributed
    cells of MIN, MAX and AVERAGE, M2 functions, so a corpus artifact keeps the current milestone's
    functions under the 99.9% bar of decision 0114 Phase 1.
  - Evidence: probes/t2867-calipers-anonymised-inputs (excel-graph, 2026-10-27) holds the shipped
    Summary formulas over B17 `Company0691` and no such sheet, the other rows of M7:O21 as the workbook's
    cached INDIRECT results (each names an existing sheet), and `=45678` on Cover!C16. Excel gives
    `#REF!` for all 25 Summary cells and 45678 for C16, the engine's values.
- **Options considered:**
  - *(a) Leave them as known failures.* Records a correct engine as failing and blocks the M2 Phase 1
    count on a corpus defect. Rejected.
  - *(b) Exclude the three cases.* Drops their other 55,912 scored cells (1,324 + 1,324 + 53,315 checked,
    less these 51) with them. Rejected.
  - *(c) A `cached_overrides` line per cell, pointing at the excel-graph probe.* The existing mechanism,
    no harness change; the harness already refuses a hand-written value. Chosen.
- **Decision:** A `cached_overrides` line may replace a cached value that the corpus's own anonymisation
  made inconsistent with the workbook as shipped (renamed text, a formula rewritten without
  recalculating, a dropped value), pointing at an excel-graph or excel-desktop probe cell of the same
  formula on the shipped inputs. The overrides file comment names the anonymisation edit for each group
  of lines, and the PR cites the probe; reviewers check sameness as for every override.
- **Consequences:**
  - eval/suites/calipers/cached-overrides.tsv gains the 51 lines above; the cells pass and leave
    eval/baseline/calipers.txt through `make eval-baseline`.
  - The rule is narrow: it applies only where the shipped file itself shows the edit (a renamed text
    naming a missing sheet, a value-less literal under `ca="1"`). A cached value that is merely unlike
    ours is never an anonymisation artifact without that evidence, and the override is never a way to
    make a failing cell pass: the probe value is a golden the engine must match.
