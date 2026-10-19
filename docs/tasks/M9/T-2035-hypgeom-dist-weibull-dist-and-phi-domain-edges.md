---
id: T-2035
title: "HYPGEOM.DIST, WEIBULL.DIST and PHI: domain edges and boolean arguments as Excel (T-1048, T-1049 sweeps)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
HYPGEOM.DIST/HYPGEOMDIST, WEIBULL.DIST/WEIBULL and PHI give Excel's value or error on the domain edges
and boolean arguments the T-1048 and T-1049 oracle sweeps recorded, instead of `#NUM!`, `#VALUE!` or a
mathematically "right" density.

## Context
docs/excel-semantics/statistics.md "Thin distribution functions: argument coercion, truncation and error
edges (T-1049)" and "Legacy distribution names" (T-1048) record these rules; T-1049's Outcome says it
implemented none of them and no open task did (gap triage 2026-10-19). probes-numeric is not gated
(no baseline file), so progress is read from `make eval` (`make eval-show CASE=probes-numeric/<case>.case.json`).
Cells (excel-graph, Excel vs ours):
- HYPGEOM.DIST / HYPGEOMDIST, sample_s above min(number_sample, population_s) is 0 (density) or 1
  (cumulative), not `#NUM!`: t1048-hypgeomdist A13, B13 `(5,4,8,20)` 0; A16, B16 `(1,0,8,20)` 0;
  t1049-hypgeomdist A35 `(5,4,8,20,TRUE)` 1, A36 `(...,FALSE)` 0, A43 `(1,0,8,20,FALSE)` 0, A45
  `(1,4,0,20,FALSE)` 0. A zero number_sample or population_s gives density 1 at s = 0: A44
  `(0,0,8,20,FALSE)` 1, A46 `(0,4,0,20,FALSE)` 1, A49 `(0,0,0,1,FALSE)` 1, and a blank one reads as 0:
  A55 `(1,Z99,8,20,TRUE)` 1, A56 `(1,4,Z99,20,TRUE)` 1. A negative fraction is `#NUM!` before
  truncation: A34 `HYPGEOM.DIST(-0.5,4,8,20,TRUE)` `#NUM!` (ours 0.10216718266253876). A TRUE sample_s
  is 1: t1048 A19, B19 and t1049 A53 (0.3632610939112486 and 0.46542827657378727; ours `#VALUE!`).
- WEIBULL.DIST / WEIBULL, the density at x = 0 is 0 for every shape: t1048-weibull A16, B16
  `(0,1,1,FALSE)` 0 (ours 1), A17, B17 `(0,0.5,1,FALSE)` 0 (ours `#NUM!`); t1049-weibulldist A27
  `(0,1,3,FALSE)` 0 (ours 0.3333333333333333), A28 `(0,0.5,3,FALSE)` 0. A TRUE x is 1: t1048 A25, B25
  `WEIBULL(TRUE,1,1,TRUE)` 0.6321205588285577, t1049 A33 `WEIBULL.DIST(TRUE,2,3,TRUE)` 0.1051606831856302.
- PHI: a boolean is a number, t1049-phi A32 `PHI(TRUE)` 0.24197072451914337, A33 `PHI(FALSE)`
  0.3989422804014327 (ours `#VALUE!`).
Code: crates/rustsheet-engine/src/functions/statistical/hypegeom.rs (the four counts are read with
`cast_number_no_bools_ref` and truncated), weibull.rs, phi.rs (`cast_number_no_bools_ref`). Switch only
the arguments listed above to the boolean-accepting read the statistical functions already use (see
T-1849 for the same change elsewhere; reuse the helper, do not add one); cite the probe cell in a comment.

## Acceptance criteria
- [ ] Every cell listed above passes in `make eval` (28 cells: 17 HYPGEOM, 9 WEIBULL, 2 PHI), and no gated baseline gains an F line.
- [ ] The two statistics.md entries say which rules are implemented (T-2035).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Booleans in the other HYPGEOM.DIST, WEIBULL.DIST arguments (unprobed: probe first, as T-1849 says).
- HYPGEOM.DIST at huge populations (t1049 A16-A18, A50, A51: T-1896, T-0977) and its last digits (T-0977).
- `PHI(1E300)` and `PHI(-1E300)` are `#NUM!` while `PHI(38.5)` is 0 (t1049-phi A25, A26): the threshold is
  unprobed, so it waits for a probe.
