# 0107 — Allow one LINEST spill-cell regression unmasked by T-1741

- **Status:** accepted (2026-10-24, T-1741)
- **Context:** T-1741 makes LINEST and LOGEST accept one point, as Excel does: a scalar known_y
  with no known_x is one point, so `LINEST(2)` is `{0,2}` (gen-m9 8d3588d8df0812df A186, 05d44f6dd6cfc567
  A378:B378, linest-modes K141; docs/excel-semantics/statistics.md "LINEST, LOGEST and GROWTH in
  the gen-m9 sweep (T-2255)"). One cell passed before only by accident: gen-m9/c468f5046aadfbbc
  Calc!B122, the spill cell of `=LINEST(EVEN(BESSELJ(1000000000000000.0,1)))`. Excel's anchor A122 is
  `#NUM!` (already a known failure) because Excel's BESSELJ is `#NUM!` at x = 1e15, so nothing
  spills into B122. Ours computes a BESSELJ value, so LINEST of the one point now spills `{0,2}` and
  B122 holds 2 where Excel has nothing. LINEST returned `#VALUE!` for one point before, so nothing
  spilled.
- **Options considered:**
  - *Keep LINEST rejecting one point.* It contradicts the oracle (some 60 gen-m9 cells and K141,
    K204 of probes/linest-modes) to keep one accidental pass.
  - *Fix BESSELJ at x = 1e15 in this PR.* The cause of Excel's large-x `#NUM!` is not established
    (docs/excel-semantics/engineering.md); T-2563 owns it, and it is another lane's work.
  - *Record the cell as a known failure and give it to the task that owns its root cause.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-1741 PR, for
  exactly this cell. T-2563 owns it: A122 and B122 are added to its acceptance list.
- **Consequences:** `eval/baseline/gen-m9.txt` gains the one line gen-m9/c468f5046aadfbbc Calc!B122
  and loses the cells T-1741 fixes. Reviewers confirm the baseline diff adds no other line. When
  T-2563 makes BESSELJ(1e15,1) `#NUM!`, A122 and B122 pass together.
