---
id: T-2258
title: "Sweep gen-m9 engineering and database: the 59 functions under 400 Excel cells to at least 500"
milestone: M9
lane: research
status: in-progress
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
