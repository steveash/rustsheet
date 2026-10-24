---
id: T-2231
title: "IM* hyperbolic and log kernel digits near the overflow edge (710-800)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The 10 known-failure cells of probes/eng-im-overflow-edges match Excel bit-for-bit.

## Context
T-2077 baselined 10 cells (A6, A9, A14, A17, A47, A58, A95, A104, A107, A109): IMCOSH/IMSINH near 710.4 differ from Excel by several units in the 15th digit (Excel 1.17873946358278E+307, true ...283, ours ...281), which suggests a scaled exp in Excel; IMLOG2/IMLOG10/IMCOT/IMSECH differ by 1 in the last digit. Also probe a tiny nonzero imaginary part with a large real part, a blank inside an IMPRODUCT range, and |y| between 709.78 and 710.4 for IMSIN/IMCOS. Code: functions/engineering/complex.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 10 cells, 0 numeric past half a unit in the 15th digit and 10 with a wrong type, error or text, blamed on IMLOG2 3, IMSINH 2, IMCOSH 2, IMCOT 1, IMSECH 1, IMLOG10 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: partial (6 of the 9 remaining cells; A47 had already passed; A9, A17, A58 moved to T-2629): the 10 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 10 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] the 10 cells pass (but A9, A17, A58: T-2629) and leave eval/baseline; the unprobed edges above are evidenced
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (IM* text whose 15th printed digit differs near the overflow edge), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).

## Outcome
- IMCOSH and IMSINH past |x| = 709 with a nonzero imaginary part compute `exp(|x| + ln(cos y / 2))` and
  `exp(|x| + ln(sin y / 2))`, so they are `#NUM!` whenever cos y or sin y is negative (`IMCOSH("709.05+2i")`).
  Evidence: new probe probes/eng-im-overflow-edges-2 (114 cells, excel-graph), which also covers the unprobed edges
  this task listed: a tiny imaginary part, IMSIN/IMCOS with 709.78 < |y| < 710.4, and IMPRODUCT over a blank.
- IMLOG2 is `ln(scaled modulus)·K` and `atan2·K` with K the double above log2 e. It fits every IMLOG2 cell in the
  suites, which ends T-2044's time-box and meets T-2530 (COMPLEXs AH19). IMLOG10's argument is `atan2 / LN_10`.
- Eval delta: 14 known failures fixed (probes 6, gen-m9 7 (T-2537's IMLOG2 cells), ironcalc-calc 1), no
  regressions, plus 114 new passing probe cells.
- Follow-up: T-2629, for A9, A17 and A58. Each is one digit off at a rounding midpoint, and no candidate fits.
