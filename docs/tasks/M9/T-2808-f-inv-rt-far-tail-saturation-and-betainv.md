---
id: T-2808
title: "F.INV.RT far-tail saturation and BETAINV's underflow and 0.5 quirk (T-2435 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
F.INV.RT gives Excel's saturated value in the far tail, and BETA.INV/BETAINV Excel's `#NUM!` where the
quantile underflows (and its 0.5 at a tiny alpha with a high probability).

## Context
Split out of T-2435 (2026-10-26), which settled the BINOM, normal-density, BETA.DIST-endpoint and
F-cdf rules of the same gen-m9 sweep and left these two search-internal ones. Both rules and their
probe evidence are in docs/excel-semantics/statistics.md, "F.INV.RT saturates at
3.696066238512854E+150/df1 in the far tail (T-2435)" and "BETAINV is #NUM! where its quantile
underflows, but 0.5 at a tiny alpha and a high p (T-2435)"; the cells are
probes/t2435-dist-extremes A68:A81, A108:A112, A115:A118 (known failures) plus the owned gen-m9
cells below. The F.INV.RT constant 3.696066238512854E+150 is df1-only (half it at df1 = 2), so it
looks like the search's own upper bracket: find what it is before implementing a cap, and read
docs/research/excel-kernel-playbook.md first. BETAINV's 0.5 is Excel's search stopping at its
start, which the crossing refinement in functions/statistical/inverse_beta.rs does not model.
Code: functions/statistical/fisher.rs (`f_quantile`), beta.rs (`beta_quantile`), inverse_beta.rs.

Owned gen-m9 cells (T-2435 split, 2026-10-26): 4 known-failure lines in eval/baseline/gen-m9.txt.
- gen-m9/3e98b26cf6713e77 Calc!A151 `=BETAINV(1E-20,1E-300,30)`: Excel #NUM!, ours 3.333333333333333e-302
- gen-m9/abc2939ebd1e2365 Calc!A11 `=CHIDIST(F.INV.RT(1E-200,1,1.999),10)`: Excel 0.0, ours #NUM!
- gen-m9/a612e89321a6df9c Calc!A23 `=GAMMALN.PRECISE(F.INV.RT(1E-20,3,9))`: Excel 1087085.7102783846, ours ...853
- gen-m9/01523d247594952e Calc!A377 `=SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`: Excel -1987062826671.328, ours -1987062826671.3352 (the F.INV.RT far tail, T-2491 hand-off)

## Acceptance criteria
- [ ] These 4 gen-m9 known-failure lines disappear from eval/baseline/gen-m9.txt, with no new
      known-failure lines in any baseline.
- [ ] probes/t2435-dist-extremes A68:A81, A108:A112 and A115:A118 pass (or the ones that cannot are
      named with evidence in the statistics.md entries above).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BINOM, normal-density, BETA.DIST-endpoint and F-cdf rules (T-2435, done).
- GAMMA.DIST at a huge shape (T-1891).
