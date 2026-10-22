---
id: T-2254
title: "Sweep gen-m7: the 23 M7 functions under 400 Excel cells to at least 500, with edge pools"
milestone: M7
lane: research
status: in-progress
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063, T-2286]
---

## Goal
The 23 M7 functions under 400 Excel cells (dynamic arrays, LAMBDA family, regex) reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 23 in all:
  BYCOL (37), BYROW (62), DROP (102), EXPAND (27), FILTER (83), ISOMITTED (54), MAKEARRAY (22), MAP (83), REDUCE (23), REGEXEXTRACT (135), SCAN (42), SORT (113), SORTBY (39), TAKE (292), TEXTAFTER (247), TEXTBEFORE (380), TEXTSPLIT (141), TOCOL (26), TOROW (59), UNIQUE (37), VSTACK (151), WRAPCOLS (91), WRAPROWS (73).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M7 --focus --functions <the list> --spill --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7` and `make eval-baseline`.
- Use `--spill` so that spilled results are checked cell by cell.
## Acceptance criteria
- [ ] In `make eval`'s "Excel evidence per function", every function listed above has at least
      400 Excel-backed cells, and at least 200 of them are new generated formulas with the
      function as the outer call. Aim for 500. Count from T-0062's checkpoint section once it has
      landed, and from the suite's cases until then.
- [ ] Every new cell is Excel-backed (excel-graph sidecars; the fill reports 0 failures, or the
      Outcome explains each refused case), and the suite is gated.
- [ ] `make eval-baseline` records the new failures as new coverage only: 0 regressions, and the
      decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces the new failures (as T-2114 did). Every visibly-off family gets a
      `campaign: visible` task, new or extended, listed in the Outcome with its cell count.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools, which go to the pool tasks. If a parameter has no pool, note it in the Outcome
  rather than inventing values ad hoc.

## Visible-parity stage
Coverage half of decision 0094's checkpoint: this task is done when its functions meet the
400-cell bar and their visibly-off cells all have tasks.
