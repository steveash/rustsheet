---
id: T-1051
title: "Probe conditional-format value operators on tiny values; move all CF ValueOperator arms to 15-digit compare"
milestone: M1
lane: harness
status: blocked
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's conditional-format value operators (Equal, NotEqual, >, >=, <, <=, Between) on tiny and 1-ulp-apart values are recorded by a probe, and the engine matches it.

## Context
Follow-up of T-1033, which moved only Equal/NotEqual to `cmp_excel_precision` (inferred from the VERIFIED 15-digit rules in `docs/excel-semantics/numbers.md`; the Graph oracle cannot evaluate conditional formats). `ValueOperator` GreaterThan/GreaterThanOrEqual/LessThan/LessThanOrEqual/Between in `conditional_formatting.rs` still compare raw f64. Probe cases: 1E-17 vs 0, 1+2^-52 vs 1, for each operator. Needs a backend that evaluates conditional formats (desktop Excel or an Excel-authored xlsx); then update the `UNVERIFIED` entry in `docs/excel-semantics/criteria.md`.

## Acceptance criteria
- [ ] A probe records Excel's result for each operator on the cases above.
- [ ] All CF `ValueOperator` arms agree with it; the criteria.md entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
The acceptance criteria need Excel's conditional-format evaluation as evidence. `rustsheet-oracle status` shows `excel-graph` (cannot evaluate conditional formats) and `libreoffice` (approximate only); `excel-desktop` is n/a (needs Windows with Excel). Without an oracle there is no evidence, and CLAUDE.md directive 1 forbids implementing from inference. Unblock when `excel-desktop` is available or an Excel-authored xlsx with conditional-format results is supplied.
