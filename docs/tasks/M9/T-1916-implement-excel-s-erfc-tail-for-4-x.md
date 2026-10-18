---
id: T-1916
title: "Implement Excel's ERF, ERF.PRECISE, ERFC and ERFC.PRECISE for all x from the T-1778 kernels"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
ERF, ERF.PRECISE, ERFC and ERFC.PRECISE reproduce Excel's bits for every x, from the kernels T-1778 found.
(Widened from the 4 < x <= 26.55 tail once T-1778 research update 10 made the whole family bit-exact.)

## Context
- The model is in docs/research/t1778-erf-kernel-research-plan.md, research updates 2-4, 7, 8 and 10; lessons in
  docs/research/excel-kernel-playbook.md. Notation: fl53(fl64(v)) rounds to a 64-bit significand (x87), then to
  a double; every Horner step is s = fl53(fl64(fl53(fl64(s*x)) + c)) from the top coefficient. For x >= 0:
  1. 0 <= x <= 0.5: ERF = fl(fl(x*N)/W), N = P(z) (quartic) and W = Q(z) (quintic) in z = fl(x*x), both stored
     (round3/sol100.txt); ERFC = fl(1 - ERF).
  2. 0.5 < x <= 4 (the split is exactly at 0.5 + 2^-53; x = 4 goes here): N degree 9, D degree 10 in x, D0 = 1;
     R = fl(N/D), ERFC = fl(E*R) with E = Excel's EXP(-fl(x*x)); ERF = fl(1 - ERFC) (round9/erfc_mid_coef.txt).
  3. 4 < x <= 26.55: the same form with N degree 8, D degree 9, D0 = 128 (round7/erfc_tail_coef.txt). Results
     below DBL_MIN flush to 0; above the cutoff ERFC = 0.
  4. E is Excel's EXP, model V6 (numbers.md, "EXP is an x87 2^(a*log2 e) evaluation"; round8/exp_v6.py), as an
     internal helper. The worksheet EXP switching to it is a separate task.
  5. ERFC(-x) = fl(2 - ERFC(x)), ERF(-x) = -ERF(x); ERF(a,b) = ERF.PRECISE(b) - ERF.PRECISE(a) with no snap;
     legacy ERF/ERFC equal the .PRECISE forms.
- The coefficients and verifiers are on branch `agent/research/T-1778-kernel-map` only, so the small text
  files land on main with provenance; the about 1.8 GB of `kmyst-*` probes do not (a curated subset does).
- The engine today is `erfc_cf`, a continued fraction that statistics.md "ERFC core factor ... (T-1187)" rules out.
- Main-branch evidence: probes/subnormal-function-boundaries A5 (`ERFC(26.4)`, T-1186), probes-numeric
  t1040-erfc and t1040-erfc-precise, and ironcalc-calc ERFs.xlsx, NORM_DIST.xlsx, GAUSS.xlsx and LOGNORM.xlsx.
- NORM.S.DIST and the other callers benefit through the shared erfc; routing them further is a follow-up (T-0968).
- Note: research update 8 called this engine work "T-1780", which is a different task on main.
- Kernel map rank 2 (docs/research/numeric-kernel-map.md, decision 0056).

## Acceptance criteria
- [ ] The coefficient files (round3/sol100.txt, round9/erfc_mid_coef.txt, round7/erfc_tail_coef.txt),
  round8/exp_v6.py, the mid_check.c and erfc_tail_check.c verifiers and the round READMEs are on main under
  docs/research/t1778/, each with a provenance line.
- [ ] The T-1778 entries of docs/excel-semantics/statistics.md and numbers.md (bit-exact kernels, wrapper
  quirks, EXP V6, xlsx misparse) are on main, merged into main's current versions.
- [ ] 0 <= x <= 0.5: ERF is the small-x rational and ERFC = fl(1 - ERF), bit-exact on the probes.
- [ ] 0.5 < x <= 4: ERFC is the (9,10) rational times Excel's EXP and ERF = fl(1 - ERFC), bit-exact, with the
  split exactly at 0.5 + 2^-53 and x = 4 on this side.
- [ ] 4 < x <= 26.55: ERFC is the (8,9) rational times Excel's EXP, bit-exact; below DBL_MIN and beyond 26.55 it
  is 0.
- [ ] Excel's EXP model V6 is an internal helper reusing the engine's x87 rounding; every constant cites T-1778.
- [ ] Negative x, ERF(a,b) and the legacy names follow the wrapper identities.
- [ ] A unit test per piece and per boundary (0.5 / 0.5 + 2^-53, 4 / 4+, the cutoff, negative x).
- [ ] A curated probe subset of the T-1778 probes (about 20-50k cells, with provenance; no misparsed inputs) is
  bit-exact except listed residuals; probes/subnormal-function-boundaries A5 and t1040-erfc(-precise) pass.
- [ ] `make eval-baseline` shows only fixes, with no regression in ERFs/NORM/GAUSS/LOGNORM/CHISQ/POISSON or the
  probes suites. T-1186 is then done.
- [ ] Cells that do not reproduce are listed with their ulp errors in the PR and statistics.md.
- [ ] Follow-ups filed: the worksheet EXP on model V6; routing NORM/CHISQ/POISSON through the T-1778 identities.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The worksheet EXP (follow-up) and EXP V6's 93 F2XM1 near-ties.
- Routing CHISQ, POISSON, GAMMA.DIST and Z.TEST through the identities (follow-up).
