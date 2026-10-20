---
id: T-2114
title: "Generate and oracle-fill gen-m5 to at least 20,000 cells with --focus and edge pools, gate it, and file its gaps"
milestone: M5
lane: research
status: in-progress
priority: P1
size: S
tier: opus
depends_on: [T-2108, T-2109, T-2110]
---

## Goal
eval/suites/gen-m5 has at least 20,000 scored, Excel-backed cells (excel-graph) spread over every
generator-ready M5 function, with edge pools. It gates `make check` against a recorded baseline,
and every mismatch family is a filed task or cites an existing one. That is the measurement M5's
exit criterion "bit-exact ≥ 99.9% on generated suites (≥ 20k cells)" needs (docs/milestones.md).

## Context
- Today gen-m5 has 10 cases (seed 1, 40 formulas each) and 400 scored cells at 99.50%. Its two
  failures, `2e604cf8d308920e` Calc!A36 `ACOTH(FACT(14))` and `d436f7d80724b076` Calc!A28
  `COTH(0.5)`, are in T-0917. The suite is `gate = false`. Keep these 10 cases. The new cases come
  from a different seed, so their content-hash names cannot collide.
- Command (after T-2108, T-2109 and T-2110):
  `cd tools && uv run rustsheet-gen --milestone M5 --focus --count 55 --formulas 400 --seed 2`, then
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5`. `write_suite` does not overwrite an
  existing suite.toml. 55 x 400 leaves headroom for cells the fill leaves pending (CONFLICT) or the
  harness skips as volatile. If the scored count still ends below 20,000, add cases with `--seed 3`.
- gen-m2's growth (T-0203) is the model: 30 x 400 formulas filled from Excel for the web in one
  session. Then `gate = true`, a baseline of the known failures, and gap triage.
- Excel refuses a whole workbook for some inputs (a formula literal ≥ 1E+308, a `hard` domain
  violation; docs/excel-semantics/numbers.md, synthetic-generator.md §3.2). A refused case is
  removed, and its cause is filed against the generator or pool (harness lane), never patched by hand.
- Triage: group the mismatches by function and kind (the gap-triager agent, or `make eval` →
  "Functions with failing cells"). Extend an open task when its family matches (T-0915 trig
  reduction, T-0917 hyperbolic ulp, T-0929 VALUE percent, T-1944 EXP edges, T-2004 LOG10 and
  others under docs/tasks/M5 and docs/tasks/M9), and add the new cell ids to its Context. Otherwise
  file a new functions-lane task with milestone M5. Each Excel quirk found goes into docs/excel-semantics/
  in this PR (CLAUDE.md directive 6).

## Acceptance criteria
- [ ] `make eval` shows gen-m5 with ≥ 20,000 scored cells, all with excel-graph oracle sidecars committed.
- [ ] Every generator-ready, non-volatile M5 function is the outer call of at least 100 scored cells
      (state the minimum in the Outcome; the T-2112 section shows it if that task has landed).
- [ ] eval/suites/gen-m5/suite.toml has `gate = true`, and `make eval-baseline` has recorded eval/baseline/gen-m5.txt.
- [ ] Each mismatch family has a task id (new or extended) listed in the Outcome with its cell count.
      New tasks are created with `make new-task`, and no family is left unfiled.
- [ ] The Outcome records the suite's bit-exact percentage and the exact commands used, including seeds.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the mismatches (functions lanes).
- Generator changes beyond removing refused cases (file a harness task instead).
