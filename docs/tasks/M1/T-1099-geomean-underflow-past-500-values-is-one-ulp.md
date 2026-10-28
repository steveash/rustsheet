---
id: T-1099
title: "GEOMEAN of 600 mixed-magnitude underflowing values is one ulp off"
milestone: M1
lane: functions-stat
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-1108]
defer: phase-1b
---

## Goal
GEOMEAN over 600 cells alternating 1E-300 and 3E-250 matches Excel bit for bit.

## Context
probes/geomean-overflow-many-values F4 (`GEOMEAN(D1:D600)`, 600 cells alternating 1E-300 and 3E-250): Excel 1.7320508075688772E-275, engine 1.7320508075688775E-275 (T-1095). The other six cells (F1 to F3, F5 to F7) pass. F4 has remainder r = 0 and q = -913, so it takes the ordinary `(product * 2^r)^(1/n) * 2^q` path in `fn_geomean` (functions/statistical/geomean.rs), not the r > 500 fallback; the miss is in the root or the scaling by 2^q for a large negative q. Fix it without breaking F1 to F3, F5 to F7 or probes/geomean-mixed-magnitude.

## Acceptance criteria
- [ ] probes/geomean-overflow-many-values F4 passes; baseline shrinks; no other cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Sonnet run, no fix. Findings (Python simulation of F4, 300 x 1E-300 and 300 x 3E-250): the engine's
path gives q=-913, r=0, mantissa product 2.333464213269655e+47 and powf(product, 1/600)*2^q =
1.7320508075688775e-275. The correctly rounded geometric mean
of the two input doubles (60-digit Decimal) is 1.73205080756887736e-275, which also rounds to ...775,
so Excel's ...772 is an artifact of its own algorithm, not a precision issue. Tried and rejected:
exp(ln(product)/n)*2^q (=...775), plain log-sum (1.732050807566386e-275), log-sum with fsum, log10 and
log2 forms (all far off). Not yet tried: other product associations or chunked rescaling (decimal
or 2^k thresholds), a root taken in pieces (sqrt/cbrt composition), or probing more underflow cases
with the oracle to constrain the algorithm. Keep F1-F3, F5-F7 and probes/geomean-mixed-magnitude green.

## Blocked because
F4 is one ulp off because of how Excel rounds the n-th root, not because of the product or the
exponent split, and no reproducible model of that rounding was found (opus run, 2026-10-07).
A new probe, probes/geomean-product-order (17 random over/underflowing sets), shows the same
one-ulp misses on 7 more cells, all at near-halfway roots (numbers.md, "GEOMEAN root rounding").
This PR lands the probe, the finding and a significand change that passes 2 more of those cells.
It blocks on T-1108, which reconstructs Excel's root rounding.

## Outcome (partial)
- New probe probes/geomean-product-order (17 cells, excel-graph 2026-10-07): 10 pass, 7 are new
  known failures, all near-halfway one-ulp roots.
- `fn_geomean` now keeps the product as a [1, 2) significand and puts the whole remainder under the
  root (fallback only past r = 1000); on the new probe it passes 2 cells (Z2, Z15) the previous
  split missed. No existing cell changed.
- Finding in `docs/excel-semantics/numbers.md`, "GEOMEAN root rounding"; follow-up T-1108.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
