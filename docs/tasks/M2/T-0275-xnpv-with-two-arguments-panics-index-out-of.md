---
id: T-0275
title: "XNPV with two arguments panics (index out of bounds on args[2])"
milestone: M2
lane: functions-financial
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`=XNPV(rate, values)` (two arguments) returns Excel's result or error instead of panicking.

## Context
Found by the T-0272 audit: `fn_xnpv` accepts 2..=3 arguments but indexes `args[2]` (`crates/rustsheet-engine/src/functions/financial/mod.rs`, `fn_xnpv`), so `=XNPV(1,A1:B2)` panics (index out of bounds). Untrusted input must never panic (directive 5). Probe Excel for the two-argument form (likely rejected as an argument-count error) before choosing the result.

## Acceptance criteria
- [x] A probe case records Excel's result for `XNPV(1,A1:B2)` and `XNPV(1,A1:B2,A1:B2)`; the finding goes in docs/excel-semantics/.
- [x] No argument count panics in XNPV; unit test covers it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_xnpv` destructures exactly three arguments, so a two-argument call is `#ERROR!` instead of an
index-out-of-bounds panic. Excel for the web rejects a workbook containing `=XNPV(1,A1:B2)` on load
(501 `FileCorruptTryRepair`, recorded in docs/excel-semantics/syntax.md "Minimum argument counts");
the three-argument forms are in the new probe probes/xnpv-argument-count (3 cells, excel-graph
2026-10-07, all passing). Unit test `fn_xnpv_argument_count`. Eval delta: probes +3 passing cells,
0 regressions.
