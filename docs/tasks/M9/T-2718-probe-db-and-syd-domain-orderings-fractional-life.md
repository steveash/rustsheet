---
id: T-2718
title: "Probe DB and SYD domain orderings: fractional life near life+1, cost 0 with a huge life, SYD negative salvage vs life overflow"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence fixes three DB and SYD domain orderings that T-2491 implemented without a probe cell, and the engine matches it.

## Context
T-2491 Excel-fidelity review (PR #1789) found three rules in `fn_db` and `fn_syd`
(crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs) that rest on inference, not on oracle cells:
- DB's partial-last-year bound compares `period.trunc()` with `life + 1`. The only evidence is probes/fin-syd-db-domain C2,
  where the life is an integer. With a fractional life, `DB(100,10,4.5,5.7,6)` changed from #NUM! to a value.
- DB's huge-life #NUM! runs before the cost-0 shortcut, so `DB(0,10,1E10,1)` is #NUM!. Every huge-life probe cell has
  a nonzero cost.
- In SYD a negative salvage beats the `life*(life+1)` overflow, so `SYD(100,-1,1E200,1)` is #NUM!, not #DIV/0!.
docs/excel-semantics/financial.md "DB and SYD argument domains".

## Acceptance criteria
- [ ] A probe case (excel-graph) covers: fractional lives (4.5, 4.25) with periods between life and life+2 at months 6 and 12;
      `DB(0,10,1E10,1)`, `DB(0,10,1E10,0.5,8)`, `DB(0,10,2147483648,1)`; `SYD(100,-1,1E200,1)`, `SYD(1E300,-1E300,1E200,1)`.
- [ ] Any mismatch is fixed in `fn_db` or `fn_syd`, and financial.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DB's one-ulp partial last year (T-2697).
