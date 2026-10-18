# 0069 — Allow two legacy T spill-row regressions unmasked by T-1729

- **Status:** accepted (2026-10-16, T-1729)
- **Context:** T-1729 makes T lift over an array value element-wise, as Excel does in a dynamic
  formula (docs/excel-semantics/information.md "T over an array lifts element-wise";
  probes/legacy-array-result S!G1:G2 and the new probes/t-array-lift, excel-graph). Two cells
  passed before only by accident. They are the spill rows of *legacy* cells
  `=T(IF(1,{"a";"b"}))` and `=T(IF(1,{2;3}))`, where Excel gives `#VALUE!` in the anchor
  (already a known failure) and nothing below it:
  - probes/legacy-array-result S!G11
  - probes/legacy-n-t-reference-args S!S11
  T returned one "" for any array before, so nothing spilled into them. Now it lifts, and these
  legacy cells spill like the dynamic ones. N is in the same state since T-1616: its spill rows
  (legacy-n-t-reference-args S!G11, H11, I11) are already in the baseline. The engine cannot tell
  a legacy cell from a dynamic one with the same text until the legacy marker of T-1727 lands.
- **Options considered:**
  - *Keep T from lifting.* It contradicts the oracle and loses the dynamic fixes.
  - *Land T-1727's legacy marker in this PR.* It is an architectural change that needs a spec
    first (CLAUDE.md directive 4).
  - *Record the two as known failures and give them to the task that owns their root cause.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-1729 PR,
  for exactly these two cells. T-1713 owns both: S11 is already in its acceptance list (R10:T11),
  and G11 is added to it.
- **Consequences:** `eval/baseline/probes.txt` loses probes/legacy-array-result S!G1 and S!G2 and
  gains these two lines, plus the one known failure of the new probes/t-array-lift case
  (S!E7, owner T-0985). Reviewers confirm the baseline diff adds no other line. T-1713 must make
  G11 and S11 pass.
