---
id: T-2062
title: "Probe ASINH and ACOSH where x*x overflows (|x| above about 1.3E154)"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's ASINH and ACOSH values are known where `x*x` overflows a double, and they are recorded in
docs/excel-semantics/numbers.md.

## Context
T-2031 made ASINH and ACOSH evaluate Excel's plain log formulas, `ln(x + sqrt(x*x +- 1))` (numbers.md,
"Inverse hyperbolics, COTH/CSCH and the SINH/TANH thresholds"). Where `x*x` overflows (|x| above about
1.3407807929942596E154) no oracle evidence exists. The engine keeps libm `asinh`/`acosh` there, so ASINH(1E200)
is about 461.2. If Excel really evaluates the formula in doubles, it may give `#NUM!` instead (ln of infinity).
Code: `fn_asinh`/`fn_acosh` in crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs.
Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case (probes-numeric) with oracle values for ASINH and ACOSH at 1E154, 1.3E154, 1.35E154, 1.4E154,
      1E200, 1E308 and 1.7976931348623157E308, and ASINH at the negatives of these.
- [ ] numbers.md records the result. If the engine differs, a follow-up implementation task is filed
      (`make new-task`), or the fix goes in this PR when it is a one-branch change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ATANH, ACOTH, COTH, CSCH (T-0917).
