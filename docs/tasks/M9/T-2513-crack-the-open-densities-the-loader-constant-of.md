---
id: T-2513
title: "Crack the open densities: the Loader constant of BETA.DIST(.,FALSE) for a, b > 1 and the Student t density (T-2464 follow-up)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Bit-level models, scored on held-out probes, for two pieces:
- the per-shape constant and small-ratio bd0 branch of Excel's Loader density (BETA.DIST(.,FALSE) with a, b > 1);
- T.DIST(t,n,FALSE).

## Context
- docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point (T-2464)" (PARTIAL) and
  "Student t tails are one kernel call" (its T density paragraph). Work in docs/research/t2464/ (a new `r3-*`
  directory).
- Loader constant:
  - The needed exponent offset runs from +8.5 units at (1.03, 40) to -24.4 at (40, 1.03), and +2.9 to +3.4 at
    (2.5, 3.5). It is not minus TOMS betaln, the GAMMALN B form, or a GAMMALN-based stirlerr (r2-density-large
    `c2_const.out`, `c5_stirvar.out`).
  - Next step (r2-density-large section 7): ulp sweeps of a and b at shapes with non-half-integer k and n - k, such as
    (5.3, 9.7), (3.7, 2.9) and (20.3, 30.1), at a log-form x. The 512-cell fingerprint then decides among stirlerr
    implementations and term orders.
  - Also open: the small-ratio bd0 branch (Excel's sd 0.8-1.3 units against about 0.4 for the exact series), (4, 4)'s
    zero q-tie response (try nq = n - fl(n·x)), and large shapes.
  - The verifier found that probe t2464-r2-density-large-p1's blocks do not test bd0, since they sit near the mode.
    New probes must put x where bd0 takes its log form.
- T density, T.DIST(.,FALSE) on 19,776 cells: CR gets 4,258 bit-exact, and no composition tried does better (best
  4,132). The rejected forms are in r2-tdist `pdf_*.out` and statistics.md "Student t density noise is not input
  rounding (T-0969)".
  - The evidence points at an extended-precision computation with a few stored values, like bgrat. Probe at exact-input
    runs with inner columns (EXP(-h LN(1+q)), GAMMALN(h), GAMMALN(n/2), POWER(1+q,-h)), as r2-tdist section 7 suggests.
- T-0969 (blocked) waits on the T density part.

## Acceptance criteria
- [ ] Both items have a RESULTS.md with a model scored on a held-out probe, or the negatives tried with their counts
      (decision 0056).
- [ ] New probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's entries are updated.
- [ ] If the T density is solved, T-0969's "Blocked because" section is updated with the new hypothesis.
- [ ] Engine follow-ups are filed for any solved piece (T-2507 owns the Loader structure).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
- The cdf pieces (T-2512).
