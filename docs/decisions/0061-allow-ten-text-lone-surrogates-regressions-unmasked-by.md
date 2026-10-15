# 0061 — Allow ten text-lone-surrogates regressions unmasked by T-1579

- **Status:** accepted (2026-10-15, T-1579)
- **Context:** T-1579 (SPEC-0010 step 5) makes MID and REPLACE keep the lone half when a position
  splits a surrogate pair, as Excel does (docs/excel-semantics/text.md "MID and REPLACE positions
  inside a surrogate pair keep the lone half"; probes/text-surrogate-positions, excel-graph). That
  fixes 39 probe cells (5 in text-surrogate-positions, 34 in text-lone-surrogates). Ten cells of
  `probes/text-lone-surrogates` passed before only by accident: MID of half a pair used to return `""`, and the functions that consume it gave Excel's answer
  for the empty string by coincidence. MID now returns the lone half, which these consumers still
  read through the lossy U+FFFD view, so they now disagree with Excel:
  - Sheet1!A3 (`UNICODE` of a lone high surrogate): owner T-1581.
  - Sheet1!A10, A11 (`EXACT`), A14, A15 (`=`), A27, A28 (`MATCH`): owner T-1585.
  - Sheet1!A55, A56, A57 (`FIND`/`SEARCH`): owner T-1586.
- **Options considered:**
  - *Keep returning `""` for a cut pair.* It contradicts the oracle and loses the 39 fixes.
  - *Do T-1581, T-1585 and T-1586 in the same PR.* Three separate tasks in two lanes' worth of
    functions; CLAUDE.md directive 4.
  - *Record the ten as known failures and give each to the task that owns its root cause.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-1579 PR, for
  exactly these ten cells. A3 is added to T-1581's acceptance list; the others are already in
  T-1585's (A10–A28) and T-1586's (A41–A60).
- **Consequences:** `eval/baseline/probes.txt` loses 39 lines and gains these ten. Reviewers
  confirm the baseline diff adds no other line. T-1581, T-1585 and T-1586 must make them pass.
