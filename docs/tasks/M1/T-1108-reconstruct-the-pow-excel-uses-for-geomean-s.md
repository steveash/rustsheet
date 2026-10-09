---
id: T-1108
title: "Reconstruct the pow Excel uses for GEOMEAN's root at near-halfway roundings"
milestone: M1
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A documented, evidenced model of how Excel rounds the n-th root in GEOMEAN's over/underflow path,
precise enough that the engine can match the 8 near-halfway cells it misses today, or a recorded
finding that no reproducible model exists.

## Context
T-1099 found that the engine's `(m * 2^r)^(1/n) * 2^q` with glibc `powf` matches Excel on 21 of
the 29 probed over/underflow cells, and every miss is one ulp where the root `X^fl(1/n)` sits
0.39 to 0.50 ulp from a rounding boundary (Excel lands on the other side). Probes:
probes/geomean-product-order (Z6, Z7, Z8, Z10, Z11, Z13, Z14 fail) and
probes/geomean-overflow-many-values F4. Rejected models (simulated in Python against all 29
cells; see numbers.md, "GEOMEAN root rounding"): pairwise or multi-accumulator products,
decimal scaling, frexp-[0.5,1) remainders, truncated division, log-sum or exp/log2 forms of the
root, the correctly rounded root with exact or rounded 1/n, and x87-style 64-bit log/exp
compositions. Excel's `^` is also not correctly rounded for non-integer exponents
(numbers.md, "`^` / POWER are not correctly rounded", T-0221), so one model may settle both. Next ideas: probe single-pow behavior directly (`=(X)^(1/n)` and `POWER` on the same
X and n, to see whether Excel's `^` shares the rounding), Newton-iteration roots, and
Excel's own `exp`/`log` on the root's arguments.

## Acceptance criteria
- [x] A probe that separates the candidate root models, filled from excel-graph, and the finding in
  `docs/excel-semantics/numbers.md`.
- [x] Either a follow-up engine task (or the fix itself, size permitting) that makes the 8 cells
  pass with no regression, or a recorded "no reproducible model" finding that closes T-1099.
- [x] `make check` green; reviewers approve.

- [x] The probe also pins (a) the "about 0.38 ulp" agreement bound with cells whose root is 0.30 to
  0.40 ulp from a halfway (the T-1099 probe chose only near-halfway sets), and (b) the engine's
  unprobed remainder-over-1000 fallback (over 1001 values, e.g. 1100 x 1E+300 and one half-size
  value).

## Out of scope
- The in-range log-sum path of GEOMEAN.

## Outcome
New probe probes/geomean-root-models (excel-graph, 2026-10-07). Root!E reproduces the engine's root
argument X bit for bit. Excel's `^` and POWER agree with each other, and they give GEOMEAN's root on
14 of 17 sets (glibc: 10), including five of the seven engine misses. They are still one ulp off on
Z5, Z11 and Z13, so they are not GEOMEAN's root. `EXP(LN)` also gets 14, on a different set, and
`10^LOG10` gets 11. No tried model reproduces all 17. This is recorded as "no reproducible root
model" in numbers.md, "GEOMEAN root rounding", which closes the T-1099 question.
(a) The 0.38-ulp agreement bound is wrong: Pin!C60 (0.357 ulp) and Pin!F60 (0.397 ulp) miss.
(b) The remainder-over-1000 fallback is never right: Excel gives `#NUM!` for 1100 large values plus
a half-size one. The follow-up probe probes/geomean-overflow-count brackets this:
- 1100 x 5E+204 is `#NUM!`; 1025 copies and 1000 x 1E+300 are finite.
- Equal values are not always returned exactly.
- In range, 1101 x 1.5 is exactly 1.5 in Excel, where the engine is 5E-15 low.
Recorded as a new numbers.md entry. Filed T-1112 (the `#NUM!` cut) and T-1113 (in-range precision).
Eval delta: probes +2 cases, 0 fixed, 0 regressions. The new-coverage failures are baselined: the
F/G/H/I root-form cells where Excel's own `^` differs from glibc's, the two pins, the fallbacks and
geomean-overflow-count A1-D1 and H1. `make eval-baseline` also adds the missing coverage line for
probes/xnpv-argument-count (merged in #322 without one).
