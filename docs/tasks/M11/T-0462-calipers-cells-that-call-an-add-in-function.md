---
id: T-0462
title: "Calipers cells that call an add-in function (_xll.) are out of scope (decision 0027) and not scored (pv26_003, 492 cells)"
milestone: M11
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Cells whose formula calls an add-in function (`_xll.` prefix) are reported as out of scope and are
not scored, as decision 0027 puts code and add-in functions out of scope.

## Context
Phase 1 assessment, 2026-10-25 (decision 0114). `calipers/pv_goldens/pv26_003` and `pv26_2_003`
have 246 visibly-off cells each, all
`IFERROR(_xll.ciqfunctions.udf.CIQHI(MO.Ticker.CapIQ,"IQ_LASTSALEPRICE",...),"N/A")`: the S&P
Capital IQ add-in. Excel's cached values came from the add-in (79.64, 91.6, ...); no engine can
compute them, so ours gives the IFERROR fallback "N/A". They count against IFERROR and against the
calipers suite, and hide real misses.

The harness already knows out-of-scope functions (`out_of_scope` in report.json,
crates/rustsheet-eval/src/report.rs, eval/catalog/scope.json). Decide whether `_xll.` calls are
"unsupported" (left out of scoring like `Unsupported`) or a new out-of-scope verdict, and make the
choice in the harness, not per workbook.

## Acceptance criteria
- [x] A cell whose formula calls any `_xll.` function is not scored (not counted as failing or
      passing) and is listed in report.md's out-of-scope section with its count.
- [x] The 492 pv26_003/pv26_2_003 cells leave the calipers failures; no other calipers cell changes.
- [x] A unit test in crates/rustsheet-eval covers it.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- Evaluating add-in functions.

## Outcome
- A formula calling an add-in function (`_xll.`, outside string literals; `formula_calls::calls_add_in`) is not scored.
  It is counted as `skipped_add_in` per case and per suite, and report.md's out-of-scope section lists each suite's
  count. This is a harness rule, not per workbook, and covers xlsx and spec cases. Only the calling cell is left out;
  its dependents are scored, so a miss they show stays visible (none fails in pv26_003). Documented in
  docs/architecture/eval-harness.md "Scoring".
- Calipers: pv26_003 and pv26_2_003 each leave 554 cells unscored, 1108 in all. Of those, 246 per workbook are the
  owned failures (492 cells). The other 308 per workbook also call `_xll.` but passed by accident, because Excel's
  cached value was IFERROR's "N/A". Both workbooks now pass, and no other calipers cell changed verdict.
- The calipers baseline is not ratcheted in this PR. `rustsheet-eval check --suite calipers` on main reports 36
  regressions that predate this change (PPMT/IPMT last digits after T-2490, plus one CHISQ.INV cell), so
  `update-baseline` refuses. Follow-up T-2708 owns them and then records the ratchet. Calipers is not gated in
  `make check` or CI.
- Unit tests: `add_in_calls_are_found_outside_string_literals` (formula_calls.rs), `add_in_calls_are_not_scored`
  (runner) and `add_in_cells_are_counted_per_suite_and_listed_out_of_scope` (report).
