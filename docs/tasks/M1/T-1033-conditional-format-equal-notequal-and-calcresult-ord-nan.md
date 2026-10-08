---
id: T-1033
title: "Conditional-format Equal/NotEqual and CalcResult Ord NaN: drop absolute epsilon, total order"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Conditional-format value rules and `Ord for CalcResult` have no absolute `f64::EPSILON` tolerance and a NaN-safe total order.

## Context
Follow-ups of T-1028 review. `conditional_formatting.rs` ~612/617 (ValueOperator Equal/NotEqual) still use an absolute epsilon; use `cmp_excel_precision` (number_format.rs). `Ord for CalcResult` maps NaN to Equal, which can make `sort` inconsistent; decide a total order. Needs an oracle probe for conditional-format Equal with tiny values (excel-probe skill).

## Acceptance criteria
- [ ] A probe records Excel's conditional-format Equal/NotEqual behavior for tiny values; the engine matches or the gap is baselined.
- [ ] `Ord for CalcResult` cannot panic or misorder on NaN.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
CF Equal/NotEqual use `cmp_excel_precision`; it is now a NaN-safe total order (NaN above numbers, -0 == 0), unit-tested. The Excel probe could not be run (Graph oracle cannot evaluate conditional formats), so the first acceptance criterion is deferred to T-1051 (probe, plus the other CF operators) and the behavior is recorded as UNVERIFIED (probe pending) in `docs/excel-semantics/criteria.md`. Eval delta: none.
