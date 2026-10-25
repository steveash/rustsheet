---
id: T-2656
title: "XMATCH reads a search_mode within 15 digits of 1 as 1 (0.5^1E-15 is 1, not #VALUE!)"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
XMATCH reads a search_mode that is within 15 significant digits of a valid mode as that mode, as Excel does:
`XMATCH(FALSE,{1,2,3},0,0.5^1E-15)` is `#N/A` (the mode reads 1, nothing matches), not `#VALUE!`.

## Context
- T-2496 probe (probes/t2496-fin-subexpressions P!A2, excel-graph, 2026-10-25), 1 gen-m9 cell, visibly off:

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/01523d247594952e Calc!A51 | `XMATCH((Data!C$12=0.5),{1,2,3},0,(0.5^1e-15))` (C12 = 0.3; the mode is 0.9999999999999993) | #N/A | #VALUE! |

- The enclosing PRICEDISC passes the error on. Ours truncates 0.9999999999999993 to 0, an invalid mode (`#VALUE!`).
  Excel's `#N/A` fits a mode cut to 15 significant digits (1.00000000000000) before the truncation, the way INT and TRUNC
  cut (docs/excel-semantics/numbers.md "INT cuts to 15 significant digits first"). Whether match_mode and XLOOKUP's
  modes do the same, and whether 1.9999999999999998 reads 2, is for a probe.
- Recorded in docs/excel-semantics/lookup.md "XMATCH: a search_mode within 15 digits of 1 reads 1" (UNVERIFIED).

## Acceptance criteria
- [ ] An excel-graph probe (`excel-probe` skill) of XMATCH and XLOOKUP match_mode and search_mode at 1-2^-52, 2-2^-51,
      -1+2^-53, 0.9999999999999 and 1.5 pins the rule.
- [ ] The engine matches the probe.
- [ ] Visible parity (gen-m9, T-2496): the known-failure line above disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] lookup.md entry updated with the probe evidence and `Status: VERIFIED`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MATCH's match_type (its own sign rule).
