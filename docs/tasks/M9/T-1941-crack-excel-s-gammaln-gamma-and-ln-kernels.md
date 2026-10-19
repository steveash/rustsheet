---
id: T-1941
title: "Crack Excel's GAMMALN, GAMMA and LN kernels to the bit"
milestone: M9
lane: research
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Find Excel's exact algorithms for GAMMALN (and GAMMALN.PRECISE), GAMMA and the worksheet LN, bit for bit,
from oracle evidence, as T-1778 did for ERF/ERFC: structure, order of operations, x87 rounding and exact
coefficient doubles. They are the foundation under the incomplete gamma and beta kernels and their
inverses (about 5,000 near-miss cells in `docs/research/numeric-kernel-map.md`, re-run 2026-10-19).

## Context
- Owner request (2026-10-19): crack the remaining numeric kernels in the kernel-map order, starting with
  gamma. Method: `docs/research/excel-kernel-playbook.md`.
- Engine tasks this feeds: T-0973 (GAMMA/GAMMALN kernel), T-1529 (blocked, time-boxed, decision 0056).
  This task supplies the new hypothesis T-1529 needs to reopen: Excel's numeric kernels are private x87
  stored-Horner rational (or series) approximations with their own coefficients. They are identified from
  dense consecutive-double runs (rounding fingerprints, phase lattices), not from named library routines.
- Known so far (statistics.md, "GAMMA and GAMMALN sweep (T-1042)"; numbers.md, "GAMMALN / GAMMALN.PRECISE
  (T-0926)"):
  - GAMMA(n) for n <= 88 is the downward product in doubles; for n >= 89 it is something else.
  - GAMMALN at integers is not correctly rounded (91 of 171 differ), and no tried model passes 108/171.
  - GAMMA(m+1/2) is unexplained.
  - EXP is model V6 (T-1778; worksheet switch in T-1931). Excel's LN has no bit-exact model yet.
- Evidence on main: probes-numeric `t1042-gamma-*`, ironcalc-calc GAMMA.xlsx, BETA_GAMMA.xlsx and
  GAMMALN_GAMMALNPRECISE.xlsx.

## Acceptance criteria
- [ ] New probes (dense consecutive-double runs across GAMMALN's and LN's pieces, plus composition
  columns) are filled by excel-graph and committed.
- [ ] Each of GAMMALN, GAMMA and LN has either a bit-exact model (with its verifier, scored on all probed
  points) or a documented structure and the remaining gap, in `docs/excel-semantics/` plus coefficient
  files under `docs/research/t1941/`.
- [ ] Lessons are added to `docs/research/excel-kernel-playbook.md`. T-0973 and T-1529 are updated to
  point at the model, and engine follow-ups are filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine implementation (T-0973 / T-1529 and follow-ups).
- Incomplete gamma and beta (T-0971, T-0970), which come next.
