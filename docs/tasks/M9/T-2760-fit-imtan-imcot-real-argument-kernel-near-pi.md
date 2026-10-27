---
id: T-2760
title: "Fit IMTAN/IMCOT real-argument kernel near pi/2 (probes/eng-imtan-imcot-overflow A10-A13)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMTAN and IMCOT of a real argument at and near the double nearest pi/2 print Excel's digits.

## Context
T-1966 probed it: Excel gives IMTAN("1.5707963267948966") = 148752068975966, ours 148752623989354 (differs from the 6th or 7th digit), and likewise for the 1.5707963267949 and IMCOT cells. Failing cells: probes/eng-imtan-imcot-overflow.case.json A10-A13 (eval/baseline/probes.txt). Notes: docs/excel-semantics/engineering.md "IMTAN and IMCOT use the double-angle form"; code: `tan_parts` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs. Numeric fitting of the kernel near a pole.

## Acceptance criteria
- [ ] The four cells A10-A13 of probes/eng-imtan-imcot-overflow pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `tan_parts` (complex/trig.rs): where the double-angle denominator rounds to 0, IMTAN now takes TAN's x87 kernel
  (`excel_tan`, reduced by the 64-bit π) instead of libm's `tan`. "1.5707963267948966" reads as 1.57079632679489, and
  1 over the tangent of `(FRAC_PI_2 - x) + π₆₄/2's tail` gives Excel's digits for all four cells (IMCOT already used
  `excel_tan` since T-2534, so A12, A13 passed before this task).
- Eval delta: probes/eng-imtan-imcot-overflow A10, A11 fixed (2 cells), 0 regressions. engineering.md "IMTAN and IMCOT
  use the double-angle form" updated.
