---
id: T-1217
title: "MULTINOMIAL loops once per unit of its argument: bound the work"
milestone: M5
lane: functions-math
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
MULTINOMIAL takes bounded time and memory for any argument, and gives Excel's value or error for large arguments.

## Context
Found by the T-1088 gen-m5 sample (seed 11, case 9f174b69572e2512, not committed): `=MULTINOMIAL(IFERROR((Data!D2+Data!D12),SUMSQ(0.2,Data!B1:B12)))`
timed out after 60 s in the harness, with SUMSQ about 1e30. `log_factorial` (functions/math_and_trigonometry/multinomial.rs) sums `ln i` for
`i` in `1..=n`, so the work is linear in the argument (`MULTINOMIAL(1e15)` loops 10^15 times), and `values.iter().sum::<u64>()` can overflow
(a panic in debug builds). Both break CLAUDE.md directive 5. The excel-graph oracle accepted the workbook and gave a value for that cell.
Probe first (excel-probe skill): `MULTINOMIAL(170)`, `(171)`, `(1e15)`, `(1e15,1)`, `(2^53)`, `(1e300)`, `(85,85)`, `(86,86)`, `(3,1e15)`;
Excel's result is probably 1 or `#NUM!` from an overflow check, but cite the evidence. Then compute in bounded time (an early overflow
exit, or `ln Γ`, which the engine has for GAMMALN), keeping the current results for small arguments.

## Acceptance criteria
- [x] A unit test evaluates `MULTINOMIAL(1e15)`, `MULTINOMIAL(1e300)` and `MULTINOMIAL(2^63,2^63)` within a time bound and with no panic (debug build).
- [x] The probe values are recorded in docs/excel-semantics/numbers.md (or math) and match the engine.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/multinomial-huge (30 cells, excel-graph, 2026-10-09): MULTINOMIAL is `#NUM!` once the truncated
arguments sum past 170 (`(170)` is a number; `(171)`, `(170,1)`, `(100,71)`, `(1E+15)`, `(2^63,2^63)` are `#NUM!`), and a
text argument still wins with `#VALUE!`. `fn_multinomial` now checks that sum (with `checked_add`) before the
factorial loops, so the work is at most 170 steps. Excel refused the workbook with a `1E+300` literal, so that form
is unprobed (the engine gives `#NUM!` by the same rule). Eval: probes +1 case, 21/30 cells pass; the 9 known failures
are the inexact values of T-1190 (baseline from `make eval-baseline`), no regressions. gen-m5/21b9ae9affd6b38c now
completes (T-1197, superseded). Follow-up: T-1233 (FACT/FACTDOUBLE have the same unbounded loop).
