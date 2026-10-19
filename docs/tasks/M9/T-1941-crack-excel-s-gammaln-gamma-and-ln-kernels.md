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

## Outcome (2026-10-19, research in progress; status stays in-progress)
Full record: `docs/research/t1941/README.md` and `HANDOFF.md` on branch agent/research/T-1941-crack-gammaln-gamma-ln.

**Bit-exact or essentially so** (statistics.md, "GAMMA, GAMMALN and LN kernels (T-1941)"):
- LN, x87 FLDLN2 + FYL2X: 114,687 / 114,688.
- GAMMALN for x >= 8: 10-term Stirling.
- GAMMALN for x < 0.7: the recurrence.
- The [3,8) glue: 100% with Excel's own columns.
- Legacy GAMMALN equals GAMMALN.PRECISE.
- All of GAMMA's reductions, reflection and special paths, given G.

**Open kernels** (structure known, coefficients not bit-exact):

| Kernel | Structure | Best set | Exact |
|---|---|---|---|
| GAMMA G on [1,2) | raw-y (7,7), D0 = -1/16 | AN2 | 78.2% (frozen; `wc/README.md` explains why and gives the future route) |
| GAMMALN near 1 | (x-1)*fl(N/D), (9,9) | `w5/p4/ss_fc.txt` | 46.8%; six constants read from plateaus |
| GAMMALN (2,3] | (x-2)*fl(N/D), (9,8) raw x | grid_a 77% / efC 35% | efC is function- and staircase-correct; Excel's N s4/s5 and D s3/s4 lines contradict the stored-product law |
| GAMMALN (1.5,2] | not a composition of neighbours | none yet | open |

**Follow-ups to file:**
- An engine task to ship the bit-exact pieces now, with the best sets for the open kernels behind a "not bit-exact"
  note.
- Per-line law validation for K2's s4/s5 lines.
- The K3 product-level phase lattice on top of ss_fc.
- (1.5,2] structure.
- The optional K1 route (i).

Playbook lessons are in `docs/research/excel-kernel-playbook.md` section 8.
