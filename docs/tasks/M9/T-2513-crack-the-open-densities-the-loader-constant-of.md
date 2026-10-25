---
id: T-2513
title: "Crack the open densities: the Loader constant of BETA.DIST(.,FALSE) for a, b > 1 and the Student t density (T-2464 follow-up)"
milestone: M9
lane: research
status: in-progress
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
- docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point (T-2464)" (PARTIAL; now
  "(T-2464, T-2513)") and
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
- [x] Both items have a RESULTS.md with a model scored on a held-out probe, or the negatives tried with their counts
      (decision 0056).
- [x] New probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [x] statistics.md's entries are updated.
- [x] If the T density is solved, T-0969's "Blocked because" section is updated with the new hypothesis.
- [x] Engine follow-ups are filed for any solved piece (T-2507 owns the Loader structure).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
- The cdf pieces (T-2512).

## Outcome
Round 3 (2026-10-24): six workers in docs/research/t2464/r3-*, each re-scored by a verifier in `verify-r3-*` (verifier
counts below; worker claims the verifier refuted are dropped). Probes: 21 cases t2513-r3-* and 6 t2513-verify-r3-*
(probes-numeric, excel-graph); every hold-out was filled before tuning and read once.
- Beta density min(a,b) <= 1, model M2 (CONFIRMED): exponent split hi = fl(fl(tx + ty) - lnB), exact remainder lo,
  density fl(EXP(hi)·fl(1 + lo)); lnB = -LN(the other shape) at a = 1 or b = 1, the GL form to max = 8 with Excel's
  measured GAMMALN. probe1 12,796/12,799/12,799 of 12,800 (M1 10,375); old min <= 1 cells 815/1,101/1,103 of 1,107
  (M1 566); hold-outs 3,106/3,991/4,062 of 4,096 and 649/816/846 of 864. algdiv (max > 8) still misses (66 of 270).
- Loader density a, b > 1 (structure CONFIRMED, PARTIAL): E extended into EXP, k, n, nk stored in R's form, bd0's x87
  log form with M - k stored, bd0 probably a double on return, cut 0.2, accurate stirlerr, (4, 4) no exception. M3 on
  the hold-out 952/2,990/3,176 of 3,296 (LDe 685, CR 251 exact); verifier hold-out 133/1,035/1,351 of 1,600 (CR
  66/533/909).
  A 2-4 unit constant residual and the series branch (+δ(2kv) term, mechanism unknown) remain.
- T density (PARTLY REFUTED): Loader's dt for df >= 3 (log branch at x² > n, two-step denominator, E stored); df 1 and 2
  closed forms (27/27, 1,050/1,050). Exact constant: hold2 573/960/960 of 960, verifier hold-out 505/918/977 of 1,000
  (CR 93/555/787: 213 visibly off against 23). Refuted: that the per-df constant is r3-loader-const's stirlerr table.
- BINOM pmf: k = n EXP(fl(n·LN p)) 172/172 (new), k = 0 173/173; BINOM and BETA(p,k+1,n-k+1) share one kernel.
  NEGBINOM sits on dbinom_raw(r-1, f+r-1, p, q); the fl(p·B) final step is refuted. Interior pmf unsolved.
- F density: not beta × Jacobian, not R's df, not GAMMALN; swap-symmetric constant (585/585), Excel's log1p switch
  on r = min(mx/n, n/mx). Unsolved (ratio model 161 against CR 114 of 1,440 exact on the hold-out).
- GAMMALN: the engine model is off on 14 of 47 arguments below 1 (verifier 24 of 66); evidence added to T-2122.
Filed: T-2585 (engine, T density, campaign visible, depends on T-2633 and T-2507), T-2586 (research: Loader constant,
bd0 series, T constant, BINOM interior, NEGBINOM final step, F density). Amended: T-2503 (M2), T-2507 (round-3
structure, still PARTIAL), T-2633 (BINOM k = n), T-2512 (algdiv pins), T-2122 (GAMMALN points), T-0969 (Blocked
because). statistics.md: four T-2464 entries and the T-0969 density entry updated, four new T-2513 entries, a
GAMMALN note.
