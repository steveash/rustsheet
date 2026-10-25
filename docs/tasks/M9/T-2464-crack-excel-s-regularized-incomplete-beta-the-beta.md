---
id: T-2464
title: "Crack Excel's regularized incomplete beta (the BETA.DIST family's kernel) from the gen-m9 sweeps, with Excel's own EXP, LN and GAMMALN"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Identify Excel's regularized incomplete beta I_x(a, b), the kernel under BETA.DIST, BETADIST, F.DIST, F.DIST.RT,
F.TEST, T.DIST.RT, T.DIST.2T, T.TEST, BETA.INV and the F/T inverses (about 5,600 visibly-off cells, 31% of all on
2026-10-23), well enough that a model is bit-exact or within 15 digits on the sweeps.

## Context
- Prior work: statistics.md "Incomplete beta: Excel's closed forms and the best candidate (T-1744)" (direct branch
  unexplained, best 15 of 107 points), the T-0969 entries (Student t is this kernel; TOMS 708 betaln structure for
  B(a,b)), T-1745 (BETA.INV inverts Excel's own CDF), T-1759 (LibreOffice and published algorithms rejected).
- New since then: Excel's EXP (model V6), LN (FLDLN2/FYL2X) and GAMMALN/GAMMA (T-1941, T-2128, T-2129) are
  modelled to the bit, and the gen-m9 sweeps (T-2255) hold thousands of oracle-filled BETA.DIST, F.DIST and
  BINOM.DIST cells. The earlier candidates were scored with libm elementary functions.
- Method: docs/research/excel-kernel-playbook.md. Work and data live in docs/research/t2464/.

## Acceptance criteria
- [x] A consolidated dataset of every oracle-filled incomplete-beta cell (BETA.DIST cdf and pdf, F.DIST family,
      T.DIST.RT) with its numeric arguments, in docs/research/t2464/.
- [x] Every hypothesis tried is recorded with its bit-exact, within-4-ulp and within-15-digit counts, rejected ones
      included (decision 0056), and the findings go into statistics.md.
- [x] If a model reaches the visible-parity stage on the sweeps, file the engine task (or land it here if small).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Research finished on 2026-10-23: two rounds of six workers, each checked by an independent verifier. The data, scripts
and RESULTS.md files are in docs/research/t2464/ (see its README, "Results"). The findings are nine "(T-2464)" entries in
docs/excel-semantics/statistics.md, after the T-1744 entry, which now points at them.
- Excel's kernel is TOMS 708 `bratio` (1992 Fortran, x0 >= 0.3): one call K(a, b, x, y) behind BETA.DIST, F, T, BINOM
  and NEGBINOM, built on Excel's own EXP, LN and GAMMALN.
- Solved:
  - the oriented closed forms, 1,286 of 1,286 cells;
  - Excel's log1p, 15,764 of 15,791 pinned rows (every miss 1 ulp), and 345 of 345 BINOM.DIST(0,n,p) cells;
  - the M1 density for min(a,b) <= 1, within 15 digits on 12,799 of 12,800 cells;
  - bpser and L140 to visible parity (L140 bup+bpser 11,724 of 11,753);
  - the T tails as K(n/2, 1/2, y, z) with an extended bgrat at b = 1/2: 1,560 of 1,564 bit-exact, and the visibly-off
    tails fall from 1,286 (CR) to 7.
- The integrated model gets 22,808/36,993/38,059 of 40,046 cdf cells (verifier), against CR's 11,473/33,974/37,069.
- Open:
  - for min(a,b) > 1 the density is Loader's (PARTIAL: the shape constant is unexplained);
  - bgrat for b ≠ 1/2, algdiv for b0 >= 8, lnΓ(a0 < 1), the b < 1 bpser ceiling, bfrac's fraction, the T density.
- Verifiers refuted the "(7.3, 0.6) lnB +24 units" and "(30.7, 1.7) sd 47" claims: the workers rounded the formula
  literals where Excel truncates them.
Filed:
- T-2645 (E0, log1p, extended EXP, BINOM k = 0); T-2502 (E1, closed forms); T-2503 (E2, M1 density);
- T-2504 (E3a, spec); T-2505 (E3, dispatch and bpser; subsumes T-2119's BETA.DIST switch); T-2506 (E4, L140);
- T-2507 (E5, Loader density); T-2508 (E6, bfrac); T-2509 (T tails, from r2-tdist); T-2510 (E7, bgrat family);
- T-2511 (E8, inverses);
- research follow-ups T-2512 (cdf pieces) and T-2513 (densities).
r2-impact estimates about 1,800 of the 5,533 pre-T2464 visibly-off family cells fixed.
