---
id: T-3771
title: "Incomplete beta: pin bpser's lnGamma(a0) just above 0.5 and split the bpser ceiling between LN and EXP"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
bpser's lnΓ(a0) is pinned on 0.5 < a0 < 0.7, and the per-a0 ceiling of the bpser model is attributed to Excel's LN at x
or to EXP's argument, or both are recorded as negatives with counts.

## Context
- Follow-up of T-3570 (docs/research/t2464/r5-bpser-lg/RESULTS.md, section 3).
- T-3570 found that no single lnΓ(a0) form is bpser's g at every a0:
  - Worksheet GAMMALN(a0) is right near 0.15 and 0.3, and from 0.7 up.
  - Just above 0.5, TOMS's fl(gamln1(a0) - LN(a0)) is right more often: 19 a0 against 7.
  - Seven other forms lost, among them correctly rounded, LN(GAMMA), the raw-t kernel and K3 kept past 1.5.
- In the same 1 < b0 < 8 sub-branch, the best g per a0 still makes only 83% of cells exact (66% at a0 = 0.15).
  - The misses are ±1-2 ulp and move with x at a fixed a0, so the cause sits in alx, s or EXP, not in L.
  - None of the series and composition variants T-3570 tried raised this ceiling.
- Work in docs/research/t2464/ in a new `r6-*` directory, importing r5-bpser-lg's `pin.py` (data and model). Method:
  docs/research/excel-kernel-playbook.md. Rules: positive control first, a real hold-out, record negatives (decision
  0056).
- Probe 1: a0 through (0.5, 0.56] in consecutive doubles at one b0, with 16 x per a0, so that each a0's g is pinned
  despite the ceiling. Take Excel's GAMMALN(a0), GAMMALN(fl(a0 + b0)) and GAMMALN(b0) from probe columns, as
  r5-bpser-lg did.
- Probe 2: next to each BETA.DIST cell, Excel's LN(x) and EXP(fl(a0*LN(x)) - L) as side columns, to tell an LN miss
  from an EXP-argument miss.

## Acceptance criteria
- [ ] A RESULTS.md records, with exact counts on a held-out half, either the lnΓ(a0) form on (0.5, 0.7) or the
      negatives tried.
- [ ] The ceiling's cell misses are split between LN, EXP's argument and the rest, with counts.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's "Incomplete beta bpser" entry is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes. The engine's bpser L is ported separately once a form is pinned.
- b0 >= 8 (algdiv, T-3383 line of work).
