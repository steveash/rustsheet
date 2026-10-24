---
id: T-2252
title: "Sweep gen-m5 round 2: the 64 M5 functions under 400 Excel cells to at least 500"
milestone: M5
lane: research
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063, T-2251]
---

## Goal
The 64 M5 functions still under 400 Excel cells after T-2114 reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 64 in all:
  ACOT (274), ASC (246), ATANH (325), BAHTTEXT (299), BASE (285), CLEAN (338), CODE (338), COMBINA (249), COT (399), COTH (321), CSCH (333), DBCS (217), DECIMAL (315), DEGREES (262), DOLLAR (274), ECMA.CEILING (243), ENCODEURL (258), EVEN (284), EXACT (273), FILTERXML (290), FINDB (222), FIXED (270), ISBLANK (356), ISERR (244), ISEVEN (228), ISFORMULA (208), ISLOGICAL (229), ISNA (277), ISNONTEXT (219), ISODD (224), ISTEXT (262), JIS (225), LEFTB (241), LENB (226), MDETERM (243), MIDB (215), MINVERSE (255), MMULT (281), MUNIT (275), NUMBERVALUE (232), ODD (275), PERCENTOF (243), PHONETIC (227), PROPER (343), QUOTIENT (315), RADIANS (321), REPLACEB (221), RIGHTB (233), SEARCHB (217), SEC (387), SECH (301), SERIESSUM (338), SHEET (239), SHEETS (28), SIGN (322), SQRTPI (282), SUMX2MY2 (244), SUMX2PY2 (232), SUMXMY2 (322), T (320), TRUNC (379), UNICODE (269), VALUETOTEXT (280), XOR (367).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M5 --focus --functions <the list> --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5` and `make eval-baseline`.
- Left out as decision 0094 exemptions (T-0062 records them): TRUE, FALSE, RAND, RANDBETWEEN and INFO, plus M2's NOW and TODAY and M7's RANDARRAY. SHEETS stays in: T-2251 makes its reference form generatable.
- M5's catalog-coverage cells for INFO and SHEETS (decision 0085 item 4) come from T-2353's small batch, not
  from this sweep. Its SHEETS formulas count toward the 400-cell bar here.
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
gen-m5 grew from 65 to 107 cases and from 22,400 to 38,780 scored cells, all excel-graph-backed (fills: 40 + 2 sidecars, 0 failures, no case refused). Generator commands:
- `cd tools && uv run rustsheet-gen --milestone M5 --focus --functions <the 64> --count 40 --formulas 400 --seed 3`: each function is the outer call of 242-260 formulas.
- Top-ups: `--functions SHEETS --count 1 --formulas 220 --seed 4`, and `--functions DBCS,FINDB,ISFORMULA,ISODD,JIS,MIDB,PHONETIC,SEARCHB --count 1 --formulas 160 --seed 5`.

Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5` and `make eval-baseline`. After the sweep, every one of the 64 functions has at least 500 Excel cells (minimum ISNONTEXT 500; SHEETS went from 28 to over 500). 99.94% bit-exact. gen-m5 known failures went from 8 to 24: 16 new-coverage, 0 regressions. Visible parity: 13 of them are visibly off.

Mismatch families (gap-triager, traced to the innermost diverging sub-expression; cells):
- Extended: T-2147 TAN near 1.2E8 (1), T-2173 ODD above 2^53 (1), T-2362 MINVERSE 1x1 tiny pivot (1, now `campaign: visible`), T-2289 BASE past 2^53 probe (its Context), T-2172 sweep (a SUMXMY2 item).
- New, visible: T-2373 BASE past 2^53 (2), T-2374 BASE of zero with min_length 0 (1), T-2375 INDEX error column vs a row index below 1 (1), T-2376 ACOT of -0 (1), T-2377 TEXT bare digits beside a date token (1), T-2378 FIXED of 99^100 (1), T-2379 NUMBERVALUE long or case-clashing separator (1), T-2380 SERIESSUM negative x to a fractional power (1), T-2381 AGGREGATE option -0.0001 with a range k (1), T-2382 ISEVEN just below an integer (1), T-2383 re-probe of a CELL("type") cell that conflicts with the verified rule (1).
- New, last digit only: T-2384 SUMXMY2 on a 1.5E+16 sum (1).

Total 16. Quirks recorded in docs/excel-semantics (numbers, information, lookup, text, number-formats, arrays). No pool gaps were noted for these functions.
