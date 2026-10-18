---
id: T-1896
title: "HYPGEOM.DIST's cdf gives Excel's value at a large population past T-1873's term cap"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1873]
---

## Goal
HYPGEOM.DIST's cdf gives Excel's value where T-1873's term cap makes it `#NUM!` (a wide distribution at a population
above about 1.84E8, or a sum of more than 2^18 terms at a population above 2^52), within the lift weight T-1873 set.

## Context
T-1873 ported statrs' `Hypergeometric::cdf` (`statistical/hypegeom.rs`) with a per-call cap: it skips the terms that
are exactly 0 and sums at most 2^18 of the rest, statrs' double bit for bit within the cap, `#NUM!` past it
(statistics.md "HYPGEOM.DIST: per-call cap and lifting", a DEVIATION). No suite case is near the cap (the largest
sums 501 terms), so Excel's values there are unprobed. Main's values past the cap:
`HYPGEOM.DIST(1074741824,2147483648,2147483648,4294967296,TRUE)` 1.0000065293398746 (191 s in release; a cdf above 1,
so statrs' terms lose digits at that size), and `HYPGEOM.DIST(524288,1048576,2^59,2^60,TRUE)` `#NUM!` (its sum
overflows). Probe first (excel-probe skill, one milestone tag): a few wide distributions at populations of 1E9 to
1E15 and x around the mode, then pick a bounded formulation that matches (a normal or Edgeworth approximation for a
large spread, or a cdf ratio recurrence from the mode, kept only where it is bit-identical inside the cap).

## Acceptance criteria
- [ ] A probe case holds Excel's values for HYPGEOM.DIST's cdf at large populations past the cap.
- [ ] Those cells pass, or are known failures with a documented residue; the values inside the cap, probes-numeric and
  ironcalc-calc are unchanged; the call stays bounded at the weight in kinds.rs (`HYPGEOM_DIST_WEIGHT`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit residues of small arguments (T-0977).
