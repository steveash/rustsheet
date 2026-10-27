---
id: T-2813
title: "SPEC-0024 amendment: how per-call capped work is charged to the formula work budget"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SPEC-0024 says how the per-call caps' work (lifted kernel units and cell visits, `limits::MAX_LIFTED_CELL_VISITS`) is
charged to the per-formula work budget so that both calibration constraints of its section 4 can hold, and T-2803 can
resume from its pushed branch.

## Context
T-2803 (branch `agent/engine/T-2803-formula-work-budget`) implemented SPEC-0024 step 1 and blocked at its calibration
rule (section 4: stop and report if the heaviest eval formula spends more than 2^26 units, or no weights meet both
constraints). Measurements (release build, local instrumentation, 2026-10-26):
- Eval: with per-call visits charged 1 unit each, `make eval-check` has 0 regressions, but the heaviest eval formulas
  spend 124,806,376 units (gen-m9/0a614cfe974b59c3 Calc!A117) and 124,783,848 (gen-m9/984cba67bbe6972b Calc!A14), about
  2^26.9: lifted financial and statistics kernels (CHISQ.DIST, GAMMAINV, POISSON.DIST, ODDLPRICE, YIELD) over PIVOTBY
  results, each one call near the 2^27 per-call cap. With 6 units per visit, 13 gen-m9 cells regress (`#VALUE!`).
- Time: `=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(RATE(SEQUENCE(1000),-1,100)))))` needs at least 5 units per visit to stop
  within 10 s (8.5 s at 6; about 50 s at 1), because a lifted RATE element costs about 190 ns per kernel unit (T-2814).
- Without the per-call caps' charges (nodes 2, built cells 2, 4 text bytes a unit, regex compile and search work), eval
  has 0 regressions and the heaviest eval formula spends 8,388,620 units (about 2^23), within the 2^26 rule; every other
  kind's worst case stops within 10 s (arrays.md "One formula evaluation does at most 268,435,456 work units").
So no single visit weight meets both constraints: an eval cell may legitimately use most of one per-call cap (2^27,
half the 2^28 formula budget), so the 2^26 rule cannot hold for any weight above about 0.5 while the per-call cap stays
2^27. Options to weigh: charge per-call work only inside a LAMBDA call (multiplication needs a helper); lower the
per-call cap or raise the formula limit for this kind; charge it at weight 1 and fix the kernel weights that make a
unit slow (T-2814) so the 10 s target holds; or relax the 2^26 margin for this kind only. Read the T-2803 branch's
`area_budget.rs` (`VISIT_UNITS`, `spend_visits`) and the task's "Blocked because" section.

## Acceptance criteria
- [ ] SPEC-0024 sections 1 and 4 state the rule for per-call capped work, with the measurements above, and the chosen
      rule meets: no eval regression, and each kind's worst case stops within 10 s (or the spec records why not).
- [ ] T-2803's "Blocked because" section points at the amendment, so the next opus run can resume it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the rule (T-2803 resumes with it).
- Kernel weight calibration (T-2814).
