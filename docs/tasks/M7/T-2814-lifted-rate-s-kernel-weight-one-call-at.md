---
id: T-2814
title: "Lifted RATE's kernel weight: one call at the 2^27 visit cap runs about 25 s"
milestone: M7
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
One lifted RATE call at the per-call visit cap (`limits::MAX_LIFTED_CELL_VISITS`, 2^27) runs in a few seconds of a
release build, not about 25 s, so the per-call cap bounds its time as SPEC-0011 "Security rule for kernel work" intends.

## Context
Found by T-2803 (SPEC-0024 calibration, 2026-10-26, release build): `=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(RATE(
SEQUENCE(1000),-1,100)))))` charges 202,009,005 units (about 200 kernel units per RATE element) and takes 38.6 s, so a
RATE element costs about 38 us, about 190 ns per kernel unit, where the other kernels measured cost 30-50 ns. One call
of RATE over about 670,000 elements stays within the 2^27 cap and takes about 25 s. Check RATE's `KernelWork` entry
(functions/lift/kinds) and the other iterative financial kernels (IRR, XIRR, YIELD, ODDFYIELD, ODDLYIELD, PRICE family)
the same way: time one call at the cap. Raising a weight can turn eval cells whose lifted calls are near the cap into
`#ERROR!`: `make eval-check` must stay at 0 regressions.

## Acceptance criteria
- [x] RATE's kernel weight (and any other kernel found more than 2x slower per unit than the table assumes) is fitted
      so one call at the cap takes at most about 5 s in a release build; the PR records the timings. (YIELD and
      ODDFYIELD were found slower too; their refit needs charge refinements first and moved to T-2879.)
- [x] No baseline gains a line (`make eval-check`: 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- The per-formula work budget (SPEC-0024, T-2803, T-2813).

## Outcome
Fitted from release-build timings of one lifted call at the 2^27 visit cap:
- RATE: a run to the 200-step cap costs about 150 us an element (10,000 elements of `RATE(3000,-1,100)`, which runs all
  200 steps to `#NUM!`, took 1.52 s), so the old 200-unit weight let one call run 671,088 elements, about 100 s.
  `RATE_WEIGHT` is now 2^13: 16,384 elements, 2.41 s measured; 16,385 is `#ERROR!`.
- Within the table: IRR (no-root flows `{1;-3;3}`, 87,000 guesses: 0.38 s), XIRR (0.84 s, the slowest found),
  ODDLYIELD and PRICE (closed form over their walks).
- YIELD and ODDFYIELD: about 17 s at the cap (the discount's `pow` slow path near overflow); a 2^26 weight brings them
  to about 4.3 s but costs two gen-m9 cells, so the refit and the two charge refinements it needs are T-2879.
Eval delta: none (`make eval-check` 0 regressions). Docs: arrays.md (RATE's lift limit).
