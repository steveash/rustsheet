---
id: T-1827
title: "Probe and match IM* results that overflow to infinity or NaN"
milestone: M9
lane: functions-engineering
status: done
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

## Outcome
Probe `eng-im-overflow` pins the overflow results. IMSUM/IMPRODUCT now give `#VALUE!` and IMEXP
`#NUM!` on a non-finite result (IMPOWER and IMDIV already gave `#NUM!`). Eval: 48 new probe cells,
8 new known failures (IMSQRT large inputs T-1996; IM* text formatting T-2001; IMEXP with a nonzero imaginary part T-2002), no
regressions. The finiteness checks are per function (not once in `complex_result`) because Excel's codes differ: `#VALUE!` for IMSUM/IMPRODUCT, `#NUM!` for IMEXP. Follow-ups: T-1996, T-2001, T-2002, T-2003 (trig overflow).
