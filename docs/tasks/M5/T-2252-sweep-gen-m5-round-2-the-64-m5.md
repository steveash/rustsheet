---
id: T-2252
title: "Sweep gen-m5 round 2: the 64 M5 functions under 400 Excel cells to at least 500"
milestone: M5
lane: research
status: in-progress
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
