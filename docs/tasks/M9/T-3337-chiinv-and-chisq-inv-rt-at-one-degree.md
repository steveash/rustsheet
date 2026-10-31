---
id: T-3337
title: "CHIINV and CHISQ.INV.RT at one degree of freedom land where Excel's search on its double cdf lands (CHIINV(1E-9,1) = 37.324892889999234, ours 37.324893106518715)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3336]
campaign: visible
---

## Goal
CHIINV and CHISQ.INV.RT at one degree of freedom (df 1 to 1.99, truncated) give Excel's value in the right tail,
where Excel's search on its own double-valued lower cdf of 1 - p lands a cdf step away from the crossing, so the
visibly-off pair below is within 15 digits of Excel (or the time-box outcome lists the forms tried, decision 0056).

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main, excel-graph; `scripts/exit_owners.py --all-off`: no open task
  named these; T-2561, which cited them as a lead, is done, and T-3311, T-3146 and T-3232 own other shapes and p):
  - probes-numeric/m9-legacy-chiinv Calc!A1 `=CHIINV(1e-09,1)` and Calc!A2 `=CHISQ.INV.RT(1e-09,1)`: Excel
    37.324892889999234, ours 37.324893106518715 (`NumericMismatch`, visibly off in the 8th digit).
- What is known (docs/excel-semantics/statistics.md "CHISQ.INV.RT and CHIINV quirks"): at df 1 the right-tail quantile
  is the lower quantile of 1 - p, `#NUM!` from p = 2^-54 down. Ours does exactly that (`chisq_inv_of` in
  crates/rustsheet-engine/src/functions/statistical/chisq.rs, the `is_one_degree` branch, `gamma_quantile(.., 1.0 - p,
  Tail::Lower)`) and returns the complement crossing. The true quantile is 37.32489305136234 (T-2561), so Excel is the
  inaccurate side: about 2.2E-7 below the crossing, one step of the double cdf's resolution near 1 (an ulp of 1 - p is
  about 1.1E-16, the density about 5E-10). At p = 1E-10 Excel is 2 ulp from ours (m9-chisq-inv-rt A1, 41.82145620298276
  against 41.82145620298279) and at 1E-4 1 ulp (m9-legacy-chiinv A12), so the step is not taken everywhere.
- Needs an oracle sweep: three points cannot say at which p Excel takes the step or what its search stops on. The fit
  waits for T-3336 (CHIINV and CHISQ.INV.RT at df 1 for p from 1E-4 to 2^-54, CHISQ.INV at 1 - p, and CHISQ.DIST at
  Excel's landing points). Read docs/research/excel-kernel-playbook.md first.
- Leads: Excel may stop when its cdf at the iterate first rounds to 1 - p (the lowest x of the flat band of the double
  cdf, as T-3146 suspects at p = 0.999 for other shapes), or bisect to a bracket width in x. Excel's lower cdf near 1 is
  1 - Q with Q from the complement (ERFC at df 1), so the band edges follow ERFC's last digits: check
  `CHISQ.DIST(x,1,TRUE)` bit-exactness at the landing points first (T-3336's CHISQ.DIST cells).
- Gap triage 2026-10-31 (100-digit mpmath): Excel's A1 value is, to the bit, the exact quantile of the lower cdf
  value one double below q = fl(1 - 1E-9) (tail 2·(1 - fl((1 + q)/2)) = 1.0000000827E-9 against q's 9.999999717E-10);
  ours is the exact quantile of q. So the lead "stops where the cdf first rounds to 1 - p" fits A1 with the stop one
  cdf ulp outside q's band, on the smaller-x side. `NORM.S.INV((1+q)/2)^2` also matches A1 but is 114 to 134,935 ulp
  off Excel at p = 0.001, 0.995, 0.999 and 0.99999, so a df-1 normal closed form is ruled out.
- The 1-to-4-ulp df-1 cells (m9-chisq-inv-rt A1-A7, m9-legacy-chiinv A11-A62, t1049-chisqinvrt, t3148-gamma-quantile-flat-band
  A64-A112) are owned by T-3340 (m9-legacy-chiinv) and T-3146 (T-3232 is not a task id); if the landing rule moves them, record which (they may pass with it).

Cells:
- probes-numeric/m9-legacy-chiinv.case.json (2): Calc!A1, A2

## Acceptance criteria
- [ ] Visible parity: the 2 cells above are within 15 digits of Excel (`make eval-show
      CASE=probes-numeric/m9-legacy-chiinv.case.json`; probes-numeric is ungated, so they have no baseline line), no new
      known-failure line appears in any eval/baseline file (`make eval-baseline`), and every T-3336 cell is within 15
      digits of Excel or named in a follow-up task.
- [ ] Or, time-boxed (decision 0056, two attempts): the Outcome and statistics.md list the candidates with their
      bit-exact and visible counts on the T-3336 cells.
- [ ] No df-1 cell that passes today regresses (t1543-inverse-cdf-tiny-p A37-A40, m9-legacy-chiinv A71, A72).
- [ ] The search stays bounded (a fixed number of cdf evaluations, directive 5).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every
visibly-off cell it owns is within half a unit of the 15th significant digit of Excel's value. Excel is the side off
the true quantile here (ours is the correctly rounded crossing), so this is the `visible` stage: reproduce Excel's
landing, never a more accurate kernel. Bit-level misses left then go to a `defer: phase-1b` follow-up.

## Out of scope
- df 2 and above (T-3311 deep lower tail, T-3146 p = 0.999, T-3310 far right tail, T-3232 last digits).
- The `#NUM!` edge between 2^-54 and 1E-16 beyond recording what T-3336 finds.
