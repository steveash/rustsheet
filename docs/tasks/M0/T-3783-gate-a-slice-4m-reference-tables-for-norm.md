---
id: T-3783
title: "Gate A slice 4m: reference tables for NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS each get a committed, oracle-filled `s1` Gate A table
with a cited mpmath definition, and `make check` runs them. Every failing point has an owner and every R3 candidate
is listed.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs a table or an `exempt.tsv` line for every in-scope
  numeric function. On main (fafb6c6f) `rustsheet-eval accuracy` still counts these six M9 functions as missing
  tables. No open task and no pending branch names them for a table: decision 0228 classified only M5, M6 and M7,
  and T-3358's distribution slices stopped at Student t, F, the discrete distributions and T-3529's gamma and beta.
- These are the normal kernels that the later slices build on. LOGNORM.* is T-3785, and the ERF family is T-3784
  (statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)":
  `NORM.S.DIST(x,TRUE)` = `ERFC.PRECISE(-x*SQRT(0.5))/2`, `GAUSS(x)` = `NORM.S.DIST(x,TRUE)-0.5` except for tiny x).
- The math: mpmath `ncdf`/`npdf` and the inverse by a high-precision root of the cdf at the double of each written
  literal. NORM.DIST and NORM.INV draw mean and standard deviation too, and the cumulative flag is a drawn argument.
- Semantics the domain and sampler must read (statistics.md):
  - "The normal CDF is not always correctly rounded" and "Normal CDF and ERFC sweep (T-1040)": the lower tail is up to
    about 1.15E-13 relative from Phi, and 0 from z = -37.625 down. That is a relative error far under decision 0188
    R3's 1e-4, so no R3 point is expected. Any point outside the envelope is a known failure with an owner.
  - "Normal density oracle sweep (T-1041)": PHI equals `NORM.S.DIST(z,FALSE)` bit for bit.
  - "Normal densities are #NUM! once the square overflows (T-2435)": the density is `#NUM!` from |z| = 2^512.
    This is part of the error domain, and T-2260 owns PHI's cut, which is still unprobed.
  - "NORM.S.INV is AS 241 PPND16 in the tails; the central region is unfitted (T-1746)" and numbers.md
    "NORM.S.INV / NORMSINV / NORM.INV(p,0,1): oracle sweep (T-1006)": p reaches about 5E-309, and the central region
    is up to 31 ulp from the correctly rounded inverse.
  - "GAUSS of a tiny argument is x times phi(0) (T-2255)": GAUSS keeps its leading term for tiny z, so its table
    stratum must reach |z| down to 1E-300.
- Likely owners for failing points (check that each is still open when filing): T-0925 (inverse normal), T-0968
  (cdf residue, blocked), T-0975 (pdf last digits). File a new task for a point none of them covers.
- Tools: as in T-3549 and T-3586: `rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`. The definitions go in
  a new `defs/normal.py`, registered in `defs/__init__.py`.
- Why opus: the definitions set the sampler's strata against evidenced Excel cuts (the zero tail, the overflow cut,
  the tiny-z GAUSS term), and each failing point must be told apart as a known failure or an R3 candidate.

## Acceptance criteria
- [ ] `tools/src/rustsheet_tools/accuracy/defs/normal.py` has one definition per function, each citing its sources,
      with unit tests against Microsoft's published examples or the semantics entries' values above.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each of the six functions, generated, filled
      (excel-graph) and committed. `rustsheet-eval accuracy` reports `missing tables` lower by 6, with 0 failing points
      and 0 table defects.
- [ ] Every point outside the envelope is a `known-failures.tsv` line with an open owner task (T-3379's rules), and the
      Outcome lists them. Any R3 candidate is listed for an owner ruling and is not marked in this PR.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The legacy names NORMDIST, NORMSDIST, NORMINV and NORMSINV, and STANDARDIZE, CONFIDENCE.NORM and Z.TEST: the M9
  statistical classification task (T-3786) rules on them.
- Changing the engine's normal kernels: each failing point gets its owner task.
