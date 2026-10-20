---
id: T-2114
title: "Generate and oracle-fill gen-m5 to at least 20,000 cells with --focus and edge pools, gate it, and file its gaps"
milestone: M5
lane: research
status: done
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
- [x] `make eval` shows gen-m5 with ≥ 20,000 scored cells, all with excel-graph oracle sidecars committed.
- [x] Every generator-ready, non-volatile M5 function is the outer call of at least 100 scored cells
      (state the minimum in the Outcome; the T-2112 section shows it if that task has landed).
- [x] eval/suites/gen-m5/suite.toml has `gate = true`, and `make eval-baseline` has recorded eval/baseline/gen-m5.txt.
- [x] Each mismatch family has a task id (new or extended) listed in the Outcome with its cell count.
      New tasks are created with `make new-task`, and no family is left unfiled.
- [x] The Outcome records the suite's bit-exact percentage and the exact commands used, including seeds.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the mismatches (functions lanes).
- Generator changes beyond removing refused cases (file a harness task instead).

## Outcome
gen-m5 now has 65 cases and 22,400 scored cells, every one backed by excel-graph. That is the 10 seed-1 cases plus 55 new seed-2 focus cases of 400 formulas each. The fill wrote 55 sidecars with 0 failures, and Excel refused no case. **98.84% bit-exact** (22,140/22,400). `gate = true`; eval/baseline/gen-m5.txt records 260 known failures, all of them new coverage.
Commands: `cd tools && uv run rustsheet-gen --milestone M5 --focus --count 55 --formulas 400 --seed 2`,
`uv run rustsheet-oracle fill -b excel-graph --suite gen-m5`, then `make eval-baseline`.
Coverage: each of the 132 M5 functions the generator emits is the outer call of at least 165 formulas.

Mismatch families (gap-triager, traced to the innermost diverging sub-expression; cells):
T-2142 SINH/COSH/EXP overflow (37), T-2134 SINH/TANH/COTH/CSCH last digit (19, extended),
T-2143 ASIN/ACOS just below -1 (14), T-2144 INT 15-digit cut (5), T-2146 ODD/EVEN above 2^53 (3),
T-2147 TAN near 1.2E8 (3; both depend on the sweep T-2145), T-2148 FIXED/DOLLAR/BAHTTEXT 15-digit cut (15),
T-2149 FIXED/DOLLAR past 2^64 (5), T-0980 TEXT 15-digit cut (2, extended), T-1730 TEXT exponent sign (1, extended),
T-0961 SUMX2MY2/SUMX2PY2/SUMXMY2 blanks and shapes (28, extended), T-2150 SUMPRODUCT blank/error order (9),
T-2151 NUMBERVALUE currency/dates (24), T-2152 NUMBERVALUE separators/empty (12), T-2153 ARABIC blank/trim (12),
T-2154 ROMAN/BASE sign (6), T-2155 REPT/CODE/UNICODE over 255 chars (6), T-2156 TYPE of a one-cell range (2),
T-2157 TEXT invalid code on text/boolean (11), T-1731 TEXT clock codes past 2958465 (3, extended),
T-2158 TEXT number-like text (4), T-1735 TEXT fill (1, extended), T-2159 LOG/DECIMAL/BASE/ATAN2/ARABIC lifting (7),
T-2160 DOLLAR/FIXED/NUMBERVALUE lifting (5), T-2161 CELL lifting (1), T-2162 COMBIN/COMBINA bound (5),
T-2163 SERIESSUM blanks/power (8), T-2164 MMULT/MINVERSE/MDETERM blank (4), T-2067 MINVERSE pivot (1, extended),
T-2165 ISEVEN/ISODD past 2^63 (3), T-2166 SWITCH FALSE vs blank (1), T-2167 MROUND (1), T-2168 VLOOKUP/HLOOKUP blank (2). Total 260.
Quirks recorded in docs/excel-semantics (numbers, text, number-formats, coercion, arrays, information, logic, lookup).
