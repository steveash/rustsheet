---
id: T-2654
title: "MOD of a small number by a huge opposite-sign divisor is 0 (MOD(1.005,-1E+30))"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
MOD matches Excel when the exact remainder plus the divisor rounds to the divisor itself: `MOD(1.005,-1E+30)` is 0 in
Excel, not -1E+30.

## Context
- gen-m5 round-3 triage (T-2648, excel-graph oracle, 2026-10-25), 1 cell in eval/baseline/gen-m5.txt, visibly off:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/abef960804539149 Calc!A400 | `=MOD(1.005,((1.005^Data!D12)-SUMSQ((-1),1000000000000000.0,Data!A6:A11)))` (D12 = -0.0001; SUMSQ is about 1E+30) | 0.0 | -1e30 |

- The same rule shows in the real-world calipers goldens (desktop, Phase 1 assessment 2026-10-25): in
  `calipers/roundtrip/formulas_math_basic` (`Math Tests`) and `formulas_datetime` (`Sheet2`), `MOD($C$6,$C$13)` with
  C6 = 1 and C13 = -9.99999999999999E+307 is 0 in Excel, -9.99999999999999E+307 ours (K84), and `MOD($C$7,$C$12)` with
  C7 = -1 is 0 in Excel, 9.99999999999999E+307 ours (J85). Check them when the rule lands.
- The divisor is about -1E+30, and both the power and SUMSQ are plain arithmetic, so MOD is the innermost divergence (its
  divisor only has to be within an ulp, and every such divisor gives the same results below). docs/excel-semantics/numbers.md
  "MOD is an exact remainder": `fmod(n,d)`, plus `d` when the result is non-zero and its sign differs from `d`. Here
  fmod is 1.005 and 1.005 + d rounds to d. Excel's 0 fits "a result equal to d in magnitude is 0" (the remainder must be
  below |d|), but other rules fit too (a quotient-based `n - d*INT(n/d)` path for tiny |n/d|, or a relative snap of
  `n/d`), so this needs a probe; opus because it is a fit.
- Recorded in numbers.md "MOD of a small number by a huge opposite-sign divisor is 0" (UNVERIFIED).

## Acceptance criteria
- [ ] An excel-graph probe (`excel-probe` skill) of `MOD(n,d)` with opposite signs over |n/d| from 2^-40 to 2^-60 around
      the point where `n + d` rounds to `d` (e.g. n = 1, 1.005, 3, 0.5 and d = -2^53, -1E+16, -1E+17, -1E+30, and the
      mirrored signs), plus same-sign controls, separates the hypotheses.
- [ ] The engine matches the probe; the gen-m5 line above disappears (`make eval-baseline`), with no new known failure.
- [ ] numbers.md entry updated with the fitted rule and `Status: VERIFIED`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MOD's `#NUM!` quotient limit (numbers.md, 1.1259E+12), which this case is far from.
