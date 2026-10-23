---
id: T-2258
title: "Sweep gen-m9 engineering and database: the 59 functions under 400 Excel cells to at least 500"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063, T-2249]
---

## Goal
The 59 M9 engineering and database functions under 400 Excel cells reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 59 in all:
  BESSELI (130), BESSELJ (311), BESSELK (134), BESSELY (311), BIN2DEC (72), BIN2HEX (81), BIN2OCT (113), BITAND (39), BITLSHIFT (57), BITOR (31), BITRSHIFT (55), BITXOR (32), COMPLEX (43), DAVERAGE (88), DCOUNT (68), DCOUNTA (86), DEC2BIN (81), DEC2HEX (49), DEC2OCT (79), DELTA (52), DGET (53), DMAX (53), DMIN (54), DPRODUCT (55), DSTDEV (60), DSTDEVP (59), DSUM (78), DVAR (61), DVARP (61), GESTEP (49), HEX2BIN (85), HEX2DEC (75), HEX2OCT (89), IMABS (47), IMARGUMENT (32), IMCONJUGATE (31), IMCOS (64), IMCOSH (70), IMCOT (67), IMCSC (67), IMCSCH (68), IMDIV (74), IMEXP (201), IMLN (72), IMLOG10 (68), IMLOG2 (69), IMPOWER (99), IMPRODUCT (63), IMSEC (71), IMSECH (71), IMSIN (70), IMSINH (69), IMSQRT (74), IMSUB (61), IMSUM (115), IMTAN (68), OCT2BIN (76), OCT2DEC (34), OCT2HEX (74).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M9 --focus --category engineering (then --category database) --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`.
- The database functions (D*) need a database range and a criteria range in the case, so the generator draws both.
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
gen-m9 gains 64 focus cases of 400 formulas each (25,600 cells), every one backed by excel-graph: 36
engineering cases (seed 2258) and 9 database cases (seed 22580) give each of the 59 functions about 300
outer calls; a top-up of 19 cases (seed 22581) gives the 49 still under 500 about 155 more. The fills wrote
64 sidecars with 0 failures, 0 refused. The suite was already gated.
Commands: `cd tools && uv run rustsheet-gen --milestone M9 --focus --functions <the 47 engineering names>
--count 36 --formulas 400 --seed 2258`, the same with the 12 D* names `--count 9 --seed 22580`, then
`--functions <the 49 under 500> --count 19 --seed 22581`; `uv run rustsheet-oracle fill -b excel-graph
--suite gen-m9` after each; `make eval-baseline`. I used `--functions` rather than `--category` so the
already-covered CONVERT/ERF family took no cells.
Coverage ("Excel evidence per function"): every listed function now has at least 505 Excel cells (minimum
IMABS 505, IMCONJUGATE 509, DCOUNTA 510), each with at least 298 new formulas where it is the outer call.
Eval: gen-m9 261 cases, 95,262 cells, 90.88% bit-exact; known failures 8,095 -> 8,696: 601 new-coverage
failures, 0 regressions, 0 fixed. Visible-parity lock 8,326 -> 8,760: the new cases add 562 visibly-off
cells, and the rest of the suite went down by 128.

Mismatch families (gap-triager, traced to the innermost diverging sub-expression; cells). New:
T-2532 COMPLEX suffix first (17), T-2533 IM* "1i" (10), T-2534 IM trig of a real via SIN/COS/TAN and 2^27 (62),
T-2535 IMSINH/IMSIN/IMCOS exp-based sinh/cosh (38), T-2536 IMSQRT of a negative real (5),
T-2537 IMSECH/IMCOT/IMCSC/IMLOG2 last printed digit (154), T-2538 IMABS/IMARGUMENT last ulp (27, not visible),
T-2539 IM log modulus at 1E±200 and IMPOWER overflow (10), T-2540 IMCSCH of 0 (11), T-2541 inumber reading (8),
T-2542 large IM parts in plain notation (1), T-2543 IMSUB/IMDIV of IFERROR-empty (1), T-2544 BIN2* of non-binary text (7),
T-2545 places checked before the number (15), T-2546 BESSEL past the order cap (5), T-2547 BESSELJ/Y above 100 (7),
T-2548 BESSEL last-ulp tail (12, not visible), T-2549 BESSELK(700,3) (1; needs the sweep T-2550), T-2551 DB negative salvage (1),
T-2552 F.TEST of a scalar (2), T-2553 D* computed criteria (16).
Extended: T-2420 (63), T-2421 (17), T-2422 (1), T-2389 (24), T-2335 (9), T-2424 (4), T-2425 (17), T-1741 (5),
T-1742 (5), T-1849 (6), T-1634 (1), T-2344 (6), T-2426 (4), T-2427 (1), T-2428 (10), T-2430 (1), T-2431 (1),
T-2436 (6), T-2468 (1), T-2470 (2), T-2474 (2), T-2483 (2), T-2487 (1), T-2516 (2). Total 601.
Every visibly-off family has `campaign: visible`. Quirks are in docs/excel-semantics/engineering.md and
criteria.md (sections citing T-2258). No parameter lacked a pool.
