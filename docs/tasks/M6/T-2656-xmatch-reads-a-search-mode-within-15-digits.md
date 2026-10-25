---
id: T-2656
title: "XMATCH reads a search_mode within 15 digits of 1 as 1 (0.5^1E-15 is 1, not #VALUE!)"
milestone: M6
lane: functions-lookup
status: done
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
- Recorded in docs/excel-semantics/lookup.md "XMATCH: a search_mode within 15 digits of 1 reads 1" (UNVERIFIED; now "XMATCH and XLOOKUP floor a fractional mode, snapping near an integer", VERIFIED).

## Acceptance criteria
- [x] An excel-graph probe (`excel-probe` skill) of XMATCH and XLOOKUP match_mode and search_mode at 1-2^-52, 2-2^-51,
      -1+2^-53, 0.9999999999999 and 1.5 pins the rule.
- [x] The engine matches the probe.
- [x] Visible parity (gen-m9, T-2496): the known-failure line above disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`). No regressions. The new probes add 4 new-coverage known-failure lines to
      probes.txt, all XLOOKUP's missing regex match_mode 3 (T-2669); every other new cell passes.
- [x] lookup.md entry updated with the probe evidence and `Status: VERIFIED`.
- [x] `make check` green; reviewers approve.

## Out of scope
- MATCH's match_type (its own sign rule).

## Outcome
- The hypothesis (a 15-digit cut) was wrong. Eight excel-graph probes (probes/t2656-xmatch-mode-*, 1070 cells,
  2026-10-25) show that XMATCH and XLOOKUP floor both modes toward minus infinity. A value within 2^-22 + 2^-33 below an
  integer counts as that integer, or within half that for a negative value. That is the rule and the constants DATE
  and TIME already use, so `date_time_whole` moved to functions/util/near_integer.rs (with the other near-integer
  helpers, out of util.rs, which would have passed 1000 lines) as `near_integer_whole` and now serves all
  four functions (one definition). The acceptance points 1-2^-52, 2-2^-51, -1+2^-53, 0.9999999999999 and 1.5 read 1, 2,
  -1, 1 and 1; the rest of the grid (ties, negatives, the exact edges) is in lookup.md.
- Eval: gen-m9/01523d247594952e Calc!A51 is fixed, and so are probes/t2496-fin-subexpressions P!A1 and P!A2 (the
  same XMATCH sub-expression; T-2496 found them). `make eval-baseline` removed all three lines. Of the new
  probe cells (nine cases, 1138 cells), all pass except 4 XLOOKUP cells whose match_mode reads 3, Excel's regex match
  mode, which our XLOOKUP lacks (follow-up T-2669). The ninth probe (t2656-xlookup-mode-snap-edge) was added at review:
  it shows that XLOOKUP snaps at the same edges as XMATCH.
- Unit test `fractional_modes_floor_but_snap_just_below_an_integer` (test_fn_xmatch.rs).
