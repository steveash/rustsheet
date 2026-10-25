---
id: T-2698
title: "NPER: a rate of 1E-300 is #DIV/0! (ln(1+rate) is 0) and a rate of -1 is #NUM! (gen-m9, 7 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
NPER returns Excel's `#DIV/0!` when its denominator ln(1+rate) is 0 (a rate so small that 1+rate rounds to 1, such as
1E-300), and Excel's `#NUM!` at a rate of exactly -1, instead of today's `#NUM!` and 0.

## Context
Cut from T-2493 (gap triage 2026-10-25, sonnet refill): the NPER half of that task needs no probe, because every cell
below has literal arguments that decide it. T-2493 keeps NPV's error order. NPV at a rate of -1 is T-2618's.

financial.md "Financial functions in the gen-m9 financial sweep (T-2257)" records the 1E-300 rule. Excel takes the
`ln(1+r)` path for any nonzero rate, and 1+1E-300 is 1, so the division is by 0. The rate -1 line is added to the same
bullet in this triage. Excel's LN(0) is `#NUM!`, which fits.

Why ours differs. `nper_of` in crates/rustsheet-engine/src/functions/financial/annuity.rs:
- At 1E-300 the ratio is 1, so `extended_quotient(0.0, 0.0)` is NaN and the `!is_finite` check returns `#NUM!`.
- At -1 the `rate < -1.0` guard lets -1 through. Then `(1.0 + rate).ln()` is -inf and the quotient is a finite 0.

Excel cells (gen-m9, excel-graph, 2026-10-23; known failures in eval/baseline/gen-m9.txt):

| cell | formula | Excel | ours |
|---|---|---|---|
| gen-m9/3c06f9298f65a3ba Calc!A148 | `NPER(1E-300,-236,2766,945)` | #DIV/0! | #NUM! |
| gen-m9/3aa168259335be54 Calc!A363 | `NPER(1E-300,-27,1000,469,((-2.5)*ROUNDUP(0,-3)))` | #DIV/0! | #NUM! |
| gen-m9/67a4c03afd63f988 Calc!A102 | `NPER(1E-300,-233,8775,380,0)` | #DIV/0! | #NUM! |
| gen-m9/a91bad4e84f49739 Calc!A336 | `NPER(1E-300,MAX((Data!E1-(-0.0001)),Data!A5:A11),1491,929)` | #DIV/0! | #NUM! |
| gen-m9/ba4ddb817a8e700e Calc!A43 | `NPER(1E-300,-567,4366,684)` | #DIV/0! | #NUM! |
| gen-m9/e168be7f6d3f84c2 Calc!A41 | `NPER(1E-300,-147,1951,894,0)` | #DIV/0! | #NUM! |
| gen-m9/3c06f9298f65a3ba Calc!A50 | `NPER((-1),-31,(Data!C$1^YEAR(0.1)))` (pv 1) | #NUM! | 0 |

Keep the existing checks in their order: rate 0 first (its own `#DIV/0!` when pmt is 0), then the ratio's `#NUM!`, then
the new zero-denominator `#DIV/0!`. A rate between 1E-300 and the point where 1+rate is no longer 1 (about 1.1E-16) is not
probed. The rule as written (denominator exactly 0 gives `#DIV/0!`) covers it without a threshold of its own.

Cells:
- gen-m9/3aa168259335be54.case.json (1): Calc!A363
- gen-m9/3c06f9298f65a3ba.case.json (2): Calc!A50, A148
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A102
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A336
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A43
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A41
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A390 (from T-2429: `HYPGEOMDIST(4,NPER((-1),-728,6163,(Data!D8/42),0),3,4)`, Excel #NUM!, ours 0; the inner NPER is Excel `#NUM!`, ours 0, probes/t2429/eced4bb6a0234c9e A10)

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2257): these 8 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `NPER(1E-300,-236,2766,945)` is `#DIV/0!`, `NPER(-1,-31,1)` is `#NUM!`, and `NPER(0,0,100)` stays
      `#DIV/0!`.
- [x] The financial.md T-2257 bullet gets an "Ours:" note naming this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- NPV at a rate of -1 or 1E+300 (T-2618) and NPV's error order (T-2493).
- NPER's last digits (T-1524 fit).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong error or a wrong type (none is a last-digit difference), so the task
is in the campaign at the `semantics` stage. Its target is that every cell it owns has Excel's type and error; the
visible-parity criterion under Acceptance criteria states it.

## Outcome
Fixed by T-2620 (same `nper_of` change). NPER is `#DIV/0!` when ln(1+rate) is 0 and `#NUM!` at a rate of -1. All 8 cells
left eval/baseline/gen-m9.txt.
The unit test is T-2620's `annuity_and_db_domain_errors`: `NPER(1E-300,-113,3037,368,1)` and `NPER(-1,-31,1)`. The rate-0
branch (`NPER(0,0,100)`) is unchanged code. The T-2257 bullet names T-2620.
