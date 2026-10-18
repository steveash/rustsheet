---
id: T-1827
title: "Probe and match IM* results that overflow to infinity or NaN"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions whose result overflows (`IMSUM("1E308","1E308")`, `IMPRODUCT("1E200","1E200")`,
`IMEXP("1000")`, an inf-inf that is NaN) give what Excel gives, presumably `#NUM!`.

## Context
Found by T-1384's security review: `complex_result` (functions/engineering/complex.rs) formats an
infinite or NaN part as text with no finiteness check, in every IM* function, so ours prints a
non-Excel string. This predates T-1384. Probe the overflow forms first (excel-probe skill), then
check finiteness once in `complex_result` and record the rule in docs/excel-semantics/engineering.md.

## Acceptance criteria
- [ ] A probe case pins the overflow results of IMSUM, IMPRODUCT, IMEXP, IMPOWER and IMDIV.
- [ ] Each matches Excel; no scalar result that was finite changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Precision of finite IM* results (T-0907).
