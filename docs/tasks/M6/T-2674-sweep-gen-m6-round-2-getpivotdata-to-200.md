---
id: T-2674
title: "Sweep gen-m6 round 2: GETPIVOTDATA to 200 generated outer-call cells, the last uncovered M6 function"
milestone: M6
lane: research
status: in-progress
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
GETPIVOTDATA has at least 200 Excel-backed gen-m6 cells with GETPIVOTDATA as the outer call, so that M6's catalog
coverage is met (39/39) and M6's exit row in "Milestone exit (decisions 0085, 0114)" loses its only coverage miss.

## Context
- Gap triage 2026-10-25 on origin/main. "Milestone exit": M6 coverage is 38/39, "not met". `report.json`
  `catalog_coverage` M6 lists GETPIVOTDATA under `missing` (281 Excel cells, `misses_generated: true`), and the
  "Fifteen-digit parity checkpoint" row is 281 cells, 0 generated, 14 visibly off. Every other M6 exit measure is met
  (gen-m6: 25,000 scored, 2 known failures against an allowance of 25; ironcalc-calc and probes have no visibly-off M6
  cell).
- T-2396 (done) taught the generator to emit GETPIVOTDATA (`rustsheet-gen --milestone M6 --focus`,
  tools/src/rustsheet_tools/gen/pivots.py, catalog `pivot: true`): a case calling it gets a `Pivot` sheet with a
  seeded source block and one compact-form pivot with its `rendered` cells. Its 4-case scratch sample was not
  committed ("the gen-m6 sweep is its own task"), and T-2253's gen-m6 sweep ran before it. No task has run it since.
- Known on that sample: excel-graph filled every cell (116 numbers, 44 `#REF!`), ours matched 157 of 160. The 3
  misses are an Average pivot's empty intersection (Excel 0, ours `#REF!`), owned by T-2404. Excel for the web
  refuses a workbook with an odd GETPIVOTDATA argument count of 5 or more (syntax.md, T-2403), so the generator's odd
  miss is the 3-argument call only.
- Commands: `cd tools && uv run rustsheet-gen --milestone M6 --focus --functions GETPIVOTDATA --count <n> --formulas
  <m> --seed 2674` for at least 220 GETPIVOTDATA formulas, then
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m6` and `make eval-baseline`.

## Acceptance criteria
- [ ] In `make eval`, "Milestone exit" shows M6 coverage 39/39 met, and the "Fifteen-digit parity checkpoint" row for
      GETPIVOTDATA shows at least 200 generated cells and covered.
- [ ] Every new cell is Excel-backed (excel-graph sidecars, canary-verified). The fill reports 0 failures, or the
      Outcome explains each refused case. The suite stays gated.
- [ ] `make eval-baseline` reports 0 regressions. The gen-m6 exit row stays met: known failures within
      `floor(0.001 * scored)`.
- [ ] Every new gen-m6 known failure is traced and owned: the empty-intersection cells go to T-2404 (extend its cell
      list), and any other family gets a new task (`campaign: visible` if visibly off, milestone M6).
      `make exit-owners MILESTONE=M6` lists no GETPIVOTDATA cell.
- [ ] Quirks found are recorded in docs/excel-semantics/ (lookup.md or web.md) in this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing GETPIVOTDATA (T-2404, T-2403, T-2662).
- New pivot shapes in the generator (several data fields, filters, dates: T-2085 probes them first).
- The two gen-m6 exit cells `make exit-owners MILESTONE=M6` lists today (b62dd1ebd266b71c A39 TAN, owned by T-2147,
  and fe0972380d030e4a A224 XLOOKUP, owned by T-2290; both tasks are M5).

## Visible-parity stage
The coverage half of decision 0094's checkpoint for M6's last function. The task is done when GETPIVOTDATA meets the
200 outer-call-cell bar and every new visibly-off family has a task.
