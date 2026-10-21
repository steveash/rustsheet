---
id: T-2225
title: "Generate and oracle-fill gen-m7 to at least 10,000 cells with spill shapes, gate it, and file its gaps"
milestone: M7
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2224]
---

## Goal
eval/suites/gen-m7 has at least 10,000 scored, Excel-backed cells (excel-graph), including spilled
cells from T-2224's footprints, spread over every generator-ready M7 function. It gates `make check`
against a recorded baseline, and every mismatch family is a filed task or cites an existing one. That
is the measurement M7's exit criterion needs: "generated dynamic-array suite (≥ 10k cells, spill shapes
included) at ≥ 99.9%; spill extents identical to Excel" (docs/milestones.md).

## Context
- Gap triage 2026-10-21 (`make eval`, `main` fce6020d): gen-m7 has 50 cases and 994 scored cells at
  99.90%, `gate = false`. Its one failure is owned. No open task grows the suite. M5's growth was
  T-2114 (done) and M6's is T-2180. This task copies them step for step.
- Command (after T-2224): `cd tools && uv run rustsheet-gen --milestone M7 --focus --spill --count <n>
  --formulas <m> --seed 2` (use the option name T-2224 lands), then
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`. Keep the existing cases (a different seed
  cannot collide with their content-hash names). Size `--count` and `--formulas` so the scored count
  (anchors plus footprint cells) passes 10,000. Add cases with `--seed 3` if it ends short.
- Excel refuses a whole workbook for some inputs (docs/excel-semantics/numbers.md, synthetic-generator.md
  §3.2). A refused case is removed and its cause filed against the generator or pool (harness lane),
  never patched by hand. RANDARRAY and the volatile functions make cells volatile, and the harness skips
  cells that depend on them, so check the scored count, not the formula count.
- Triage: group the mismatches by function and kind (the gap-triager agent). Extend an open task when its
  family matches (e.g. T-1835, T-1836, T-1839 lifting, T-1855 PIVOTBY, T-1899, T-1900, T-1973 REGEX,
  T-1965 stacks), adding the new cell ids to its Context. Otherwise file a new task in the lane of the
  module the fix touches, with milestone M7. Each Excel quirk goes into docs/excel-semantics/ in this PR
  (CLAUDE.md directive 6).
- Why opus: the oracle run's refusals and conflicts need judgement, and the triage decides groupings,
  tiers and new semantics entries.

## Acceptance criteria
- [ ] `make eval` shows gen-m7 with >= 10,000 scored cells, with all excel-graph oracle sidecars committed,
      and at least 2,000 of those cells are spilled footprint cells (state the count in the Outcome).
- [ ] Every generator-ready, non-volatile M7 function is the outer call of at least 50 scored formulas
      (state the minimum in the Outcome).
- [ ] eval/suites/gen-m7/suite.toml has `gate = true`, and `make eval-baseline` has recorded
      eval/baseline/gen-m7.txt.
- [ ] Every known failure in eval/baseline/gen-m7.txt is cited by an open task (new or extended), listed by
      case and cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the gaps found (the filed tasks).
