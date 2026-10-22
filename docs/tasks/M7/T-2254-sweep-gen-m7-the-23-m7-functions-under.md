---
id: T-2254
title: "Sweep gen-m7: the 23 M7 functions under 400 Excel cells to at least 500, with edge pools"
milestone: M7
lane: research
status: done
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
- [x] In `make eval`'s "Excel evidence per function", every function listed above has at least
      400 Excel-backed cells, and at least 200 of them are new generated formulas with the
      function as the outer call. Aim for 500. Count from T-0062's checkpoint section once it has
      landed, and from the suite's cases until then.
- [x] Every new cell is Excel-backed (excel-graph sidecars; the fill reports 0 failures, or the
      Outcome explains each refused case), and the suite is gated.
- [x] `make eval-baseline` records the new failures as new coverage only: 0 regressions, and the
      decision 0088 lock moves only by new coverage.
- [x] The gap-triager traces the new failures (as T-2114 did). Every visibly-off family gets a
      `campaign: visible` task, new or extended, listed in the Outcome with its cell count.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools, which go to the pool tasks. If a parameter has no pool, note it in the Outcome
  rather than inventing values ad hoc.

## Visible-parity stage
Coverage half of decision 0094's checkpoint: this task is done when its functions meet the
400-cell bar and their visibly-off cells all have tasks.

## Outcome
gen-m7 grew from 38 to 52 cases (14 new, all `--spill`), every new cell excel-graph-backed (fills: 13 + 1 sidecars,
0 failures, no case refused; 119 cells holding an undecodable rich-value error were left pending by the oracle, the
T-2397 family). Generator commands:
- `cd tools && uv run rustsheet-gen --milestone M7 --focus --spill --functions <the 23> --count 13 --formulas 400 --seed 31`:
  each function is the outer call of about 226 formulas.
- Top-up: `--functions BYCOL,MAKEARRAY,REDUCE,SCAN,TOCOL --count 1 --formulas 400 --seed 32` (80 formulas each).

Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7` and `make eval-baseline`. Excel cells per function
after the sweep (report "Excel evidence per function"): BYCOL 460, BYROW 407, DROP 479, EXPAND 457, FILTER 512,
ISOMITTED 405, MAKEARRAY 452, MAP 476, REDUCE 456, REGEXEXTRACT 532, SCAN 480, SORT 552, SORTBY 923, TAKE 770,
TEXTAFTER 607, TEXTBEFORE 743, TEXTSPLIT 510, TOCOL 474, TOROW 412, UNIQUE 834, VSTACK 579, WRAPCOLS 467,
WRAPROWS 467: all at least 400, ten short of 500 (the aim). gen-m7 known failures went from 56 to 706: 650
new-coverage, 0 regressions; all 650 are visibly off (in-scope visibly-off cells 1,950 → 2,600,
decision 0088 lock 2947 unchanged).

Mismatch families (gap-triager, traced to the innermost diverging sub-expression; cells):
- New, visible: T-2415 EXPAND with a multi-cell range pad_with is one `#VALUE!` (312), T-2406 MAKEARRAY array step
  `#CALC!` and BYROW/BYCOL/MAP LAMBDA step `#CALC!` (164), T-2407 MAP/SCAN 1x1 array step (19), T-2408 EXPAND and
  MAKEARRAY sizes past the grid (5), T-2409 TOROW/UNIQUE of a lone blank cell (2), T-2410 ROMAN lifts over an array
  form (5), T-2411 VLOOKUP scalar text range_lookup beside a lifted lookup_value (8), T-2412 TEXTAFTER match_end cut
  toward zero (1), T-2413 blank HSTACK element in a REDUCE LAMBDA (1), T-2414 probe of four unexplained results (31).
- Extended: T-2397 spill holes at undecoded errors (28), T-2360 GROUPBY array LAMBDA results (26), T-2315 TEXTSPLIT
  error order (24), T-2288 DATEDIF unknown unit (12), T-2348 UNIQUE uniform 2-D exactly_once (6), T-1946 MAP error
  later array (4), T-2296 SUMPRODUCT error element (1), T-2365 WRAPCOLS error wrap_count (1).

Total 650. Quirks recorded in docs/excel-semantics (arrays, lookup, text, numbers, information, dates, coercion).
No pool gaps were noted.
