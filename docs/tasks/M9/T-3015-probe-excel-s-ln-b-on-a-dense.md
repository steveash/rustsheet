---
id: T-3015
title: "Probe Excel's ln B on a dense shape grid (b = 8.5 to 9.5 by 1/16, a = 2 to 8) through BETA.DIST's bpser, to reopen T-2924 with a smooth-or-jump answer"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
New Excel evidence that says how Excel's stored ln B(a, b) error k (integer ulps from the accurate value) varies
when b moves in small steps: smoothly (a formula in a/b or in a + b) or in jumps (the rounding of one large term).
That is the reopen condition T-2924 names, and T-2924's fit is what every visibly-off cell below waits on.

## Context
Gap triage 2026-10-28 (eval on origin/main abefb50). The visibly-off cells that no fix can reach until Excel's ln B for
both shapes above 1 and one above 8 is known:
- T-3007's seven `2*F.DIST.RT(1E+12,df1,df2)` tail cells (listed there, added by this triage) and T-2924's
  gen-m9 F.TEST cell A306 (25 ulp low). statistics.md (F.TEST, T-2829) says the F.DIST.RT cells are off by T-2924's k
  plus steps of the fl(a·LN x) grid.
- BETA.DIST's bpser cells with a shape above 8 were never scored (T-2924 "Blocked because"), so how many BETA.DIST
  cells wait on the same L is unknown.
T-2924 (blocked, time-boxed after two runs) read k off 99 F.TEST cells deep in bpser on a coarse grid (a = 2.5 to 8 by
b = 8.5 to 19.5, `docs/research/t2829/ln_beta_map.py`, `ln_beta_map.out`) and ruled out whole families
(`docs/research/t2924/structure.py`: no sum of double lnGamma values, no TOMS a-reduction for integer a, no separable
per-argument error). Its reopen clause asks for exactly this probe: a fixed a over 2, 3, ..., 8 at b = 8.5 to 9.5 by
steps of 1/16.
F.TEST cannot give fractional shapes (its shapes are (n - 1)/2), so the probe goes through BETA.DIST(x, a, b, TRUE)
with x tiny enough that bpser's series is 1 + O(x) and the cell is EXP of (a·LN x - L - LN a) up to known roundings.
Choose x so that the other roundings are pinned: a power of 2 makes LN x Excel's FLDLN2/FYL2X value
(numbers.md "LN is FLDLN2 then FYL2X"; the engine's `x87_log10.rs` reproduces it), and the product a·LN x is
then one rounding; EXP is the engine's `x87_exp.rs` model. docs/research/t2464/excelmath.py evaluates Excel's EXP, LN,
GAMMALN and GAMMA through the python bindings (`make py-bindings`). Check first, on a few F.TEST twins at half-integer
shapes, that BETA.DIST's bpser carries the same L as F.TEST's (T-2924 says "the same L is open for BETA.DIST").

## Acceptance criteria
- [ ] A probes-numeric case (excel-graph, filled with `cd tools && uv run rustsheet-oracle fill`) with BETA.DIST cells
      at a = 2, 2.5, 3, ..., 8 and b = 8.5 to 9.5 by 1/16 (about 230 cells), plus F.TEST twins at a few half-integer
      shapes as the control.
- [ ] A script under docs/research/t3015/ that reads k for every cell (the method of ln_beta_map.py), with its output
      committed; the control shows BETA.DIST and F.TEST agree on L, or the probe records that they do not.
- [ ] statistics.md (the T-2924 entry) states the answer: k smooth in b at fixed a (and the form it suggests), or
      jumping (where, and which large term's rounding fits the jumps), citing the new cells.
- [ ] T-2924's "Blocked because" gets the new hypothesis, so the steward or the next opus run can reopen it; if the
      answer rules the reopen out, say so there instead.
- [ ] The new case passes the eval gates (probes-numeric is ungated; `make check` green); reviewers approve.

## Out of scope
- Fitting or porting Excel's ln B (T-2924, then T-3007).
- The fl(a·LN x) grid steps that separate F.DIST.RT from F.TEST (T-3007 scores those once L is known).
