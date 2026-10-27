---
id: T-2509
title: "Student t tails through the ported kernel: K(n/2,1/2,y,z) and Excel's bgrat at b = 1/2 (T-2464 r2-tdist)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2645, T-2505, T-2506]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
T.DIST(TRUE), T.DIST.RT, T.DIST.2T, TDIST and T.TEST's tails make one call into the ported incomplete beta, K(n/2, 1/2,
y, z). bgrat at b = 1/2 is Excel's extended-precision form with three stored pieces. The T tails' visibly-off cells
fall from about 1,286 (the CR level) to about 7.

## Context
- T-2464 research, docs/research/t2464/r2-tdist/RESULTS.md. r2-impact's plan did not have this result: its step E7
  kept T on `student_t.rs` because plain TOMS bgrat breaks T (+60 visibly-off cells).
- The rule is docs/excel-semantics/statistics.md "Student t tails are one kernel call, K(n/2, 1/2, y, z), with bgrat at
  b = 1/2 in extended precision (T-2464)":
  - y = fl(n/fl(n + fl(t·t))), and the complement z = fl(fl(t·t)/fl(n + fl(t·t))). At df 2 the complement is
    fl(1 - y).
  - L = I/2; 2T = I; the upper side is fl(1 - L); y == 1 gives L = 0.5.
  - bgrat at b = 1/2: ν = a - 1/4, and lnx = LN(y) if z > 0.01, else Excel's log1p(-z) (T-2645).
  - TOMS bgrat runs in x87 extended, except three stored doubles: lnx, z = fl(-ν·lnx), and
    ERFC.PRECISE(fl(sqrt(z))). ERFC.PRECISE is the engine's bit-exact `functions/statistical/erf.rs` model (T-1778).
- Model: `r2-tdist/xlk.py` (`bgrat_half_ld`, `XCfg`), scored by `tscore.py` (`args=t2,alnrel=xl,erfc=xl,
  bg_lnx=cut0.01,bg=ld`).
- Scores:
  - T bgrat cells: `bgrat_half_ld` gets 1,545/1,562/1,563 of 1,564; the ideal form gets 1,560. CR gets 48 and r2-bratio's
    kernel 35.
  - Probe t2464-r2-tdist-p1 part C: 4,895 of 4,915 with the implementable `bgrat_half_ld` (4,906 for the ideal
    form `bgrat_half_ideal` in statistics.md).
  - All T tails (FINAL): 3,383/5,126/5,145 of 5,152 lower-side cells; visibly off 7, against CR 1,286.
  - Non-T bgrat cells at b = 1/2: 365 of 387.
- The other T branches come from T-2505 and T-2506:
  - bpser, including the centre bpser with z as x0;
  - w1 = bpser;
  - bup+bgrat for n <= 30, where bup's x0 factor is z: pdf·y·z gets 494 of 1,344, against 224 with fl(1 - y).
- The remaining 1% of `bgrat_half_ld` misses are final-rounding near-ties of the extended exp and log. A double-double
  evaluation may close them (r2-tdist section 3).
- Overlap:
  - T-0969 (blocked, time-boxed) names "a reconstruction of Excel's BETA.DIST cdf kernel that reproduces column E of
    `t0969-t-beta-link`" as its reopen hypothesis. This task provides it for the tails.
  - T-0969's density part stays blocked (T-2513).
  - T-2126 (centre above df 1000) and T-2213 (the rounded-y cut) are likely settled by this rule; re-triage them in
    the Outcome.
- Code: `student_t.rs` (`unflushed_upper_tail`, `closed_form_tail`, `log_gamma_ratio`), `t_dist.rs`.

## Acceptance criteria
- [ ] The T bgrat cells of docs/research/t2464/cells.tsv score at least 1,545 bit-exact of 1,564, and t2464-r2-tdist-p1
      part C at least 4,895 of 4,915.
- [ ] The T tail family on cells.tsv's 5,152 lower-side cells is visibly off on at most 7 (the y == 1 guard fixes two of
      r2-tdist's seven).
- [ ] T.DIST.2T(t) = 2·T.DIST.RT(t), and the other wrapper identities of w5's table, hold bit for bit.
- [ ] bgrat at b = 1/2 is shared with the non-T callers (T-2510 uses it).
- [ ] No gated known-failure line is added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The T density, T.DIST(.,FALSE) (T-2513).
- bgrat for b ≠ 1/2 (T-2510, T-2512).
- T.INV (T-2511).

## Visible-parity stage
Decision 0083: this task is done when every T tail cell it owns (T.DIST TRUE, T.DIST.RT, T.DIST.2T, TDIST, and T.TEST's
tail) is within half a unit of the 15th significant digit of Excel's value, or the cells still off are named in the
Outcome. Bit-level criteria still unmet then move to a follow-up task without the campaign key.
