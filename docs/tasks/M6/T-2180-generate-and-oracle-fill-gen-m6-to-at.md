---
id: T-2180
title: "Generate and oracle-fill gen-m6 to at least 20,000 cells with --focus and edge pools, gate it, and file its gaps"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2179]
---

## Goal
eval/suites/gen-m6 has at least 20,000 scored, Excel-backed cells (excel-graph) spread over every
generator-ready M6 function, with the T-2179 date pools. It gates `make check` against a recorded baseline,
and every mismatch family is a filed task or cites an existing one. That is the measurement M6's exit
criterion needs: "same as M5" (docs/milestones.md), so bit-exact >= 99.9% on the gen-m6 row alone with
>= 20,000 scored cells, deviations excluded (decision 0085, which says M6 inherits the M5 definitions).

## Context
- Gap triage 2026-10-20 (`make eval`): gen-m6 has 25 cases and 1,000 scored cells at 99.90%, `gate = false`
  (eval/suites/gen-m6/suite.toml). Its one failure, gen-m6/ed0f4f783473cef0 Calc!A23
  (`TANH(SUM(..))`), belongs to T-2134. No open task grows the suite; M5's growth was T-2114 (done), which
  this task copies step for step. The M6 probes (72 known failures) and the cumulative M6 ironcalc-calc row
  (14 failing, the same cells as M5) are covered by open tasks; this suite is the M6 exit gap with no task.
- Catalog coverage (report "Catalog coverage per milestone"): M6 has 39 functions, 38 with cells in the
  generated suite; GETPIVOTDATA is missing (needs a pivot table, SPEC-0015: T-2083, T-2084), so leave it out.
- Command (after T-2179): `cd tools && uv run rustsheet-gen --milestone M6 --focus --count 55 --formulas 400 --seed 2`,
  then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m6`. Keep the 25 existing cases (a different seed
  cannot collide with their content-hash names). If the scored count ends below 20,000, add cases with `--seed 3`.
- Excel refuses a whole workbook for some inputs (docs/excel-semantics/numbers.md, synthetic-generator.md §3.2).
  A refused case is removed and its cause filed against the generator or pool (harness lane), never patched by hand.
  INDIRECT, OFFSET and the volatile functions make cells volatile; the harness skips cells that depend on them
  transitively, so check the scored count, not the formula count.
- Triage: group the mismatches by function and kind (the gap-triager agent). Extend an open task when its family
  matches (e.g. T-1484 WEEKNUM stale return type, T-2064/T-2065 reference lifting, T-2134 hyperbolics, the M5
  tasks for nested M5 calls) and add the new cell ids to its Context. Otherwise file a new task in the lane of
  the module the fix touches (functions-date, functions-lookup, ...) with milestone M6. Each Excel quirk goes into
  docs/excel-semantics/ in this PR (CLAUDE.md directive 6).
- Why opus: the oracle run's refusals and conflicts need judgement, and the triage decides groupings, tiers
  and new semantics entries.

## Acceptance criteria
- [ ] `make eval` shows gen-m6 with >= 20,000 scored cells, all with excel-graph oracle sidecars committed.
- [ ] Every generator-ready, non-volatile M6 function is the outer call of at least 100 scored cells (state the
      minimum in the Outcome).
- [ ] eval/suites/gen-m6/suite.toml has `gate = true`, and `make eval-baseline` has recorded eval/baseline/gen-m6.txt.
- [ ] Every known failure in eval/baseline/gen-m6.txt is cited by an open task (new or extended), listed by case and
      cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the gaps found (the filed tasks do).
- GETPIVOTDATA cases (SPEC-0015).
