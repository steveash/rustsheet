---
id: T-2811
title: "Last digits of the T-2435 probe's distribution cells (BINOM pmf, GAMMA.DIST huge shape, F cdf at a huge df, beta density)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The last digits of the T-2435 probe's accuracy cells match Excel, so probes/t2435-dist-extremes has
no known failure left that is not a rule of T-1891 or T-2808.

## Context
T-2435 settled five argument-range rules of the gen-m9 extremes sweep; its probe,
probes/t2435-dist-extremes (142 formulas, excel-graph, 2026-10-26), also records these last-digit
and accuracy gaps, which no other task names. They are the cells of that case left in
eval/baseline/probes.txt once T-1891 (A21, A22, A25, A26, A33, A34, A101, A102), T-2808 (A68, A69,
A72, A73, A77:A81, A108:A112, A115:A118) and T-2809 (A15, A18) take theirs:
- A14 `BINOM.DIST(1,2,0.9999999999999999,FALSE)`: Excel 1.998401444325278E-15, ours ...2798 (the
  pmf walk at a p that is 1 - 1 ulp; see "BINOM.DIST pmf: closed forms at k = 0 and k = n").
- A20, A23, A24, A32: GAMMA.DIST's cdf at a shape of 1E+9 or 5E+9 (13th digit) and its density at
  1E+10 (1 ulp).
- A35:A38, A40, A41: the F cdf, its density and F.INV.RT at a df2 of 1E+10 or 10000000000.5, where
  `F.DIST(1,10000000000.5,6,TRUE)` is 0.4231900476583052 against Excel's 0.4231900811940569 (the
  6th digit; statistics.md "F-distribution df limit (T-1530)" and T-1538's huge-df entries).
- A66 `BETA.DIST(1E-300,0.5,1E+5,FALSE)`: Excel 1.784121885998999E+152, ours 1.7841218857004317E+152
  (the 10th digit, the beta density at a tiny x and a huge shape).
Each group is an accuracy question about an existing kernel, so take the groups one at a time and
read docs/research/excel-kernel-playbook.md first; splitting this task per group is fine.
One probe also belongs here (T-2435 review round 3): BETA.DIST's end-of-interval 0 is implemented on
the scaled t, so it fires where x differs from A but x - A cancels to 0
(`BETA.DIST(1E+300+1,1,2,FALSE,1E+300,1E+301)`, and the upper-end analogue). Excel's value there is
unprobed; fill it and either confirm the current 0 or key the test on x == A and x == B.

## Acceptance criteria
- [ ] These probes/t2435-dist-extremes cells pass (A14, A20, A23, A24, A32, A35:A38, A40, A41, A66),
      or any that cannot are named with evidence in the statistics.md entry that owns the kernel.
- [ ] The cancelling-x BETA.DIST cells above are probed and the "unprobed" clause in statistics.md
      "BETA.DIST's density is #NUM! at an x of 0 or 1 (T-2435)" is replaced by what Excel gives.
- [ ] No new known-failure lines in any baseline; `make check` green; reviewers approve.

## Out of scope
- The five rules of T-2435 (done), T-1891's huge gamma shapes, T-2808's F.INV.RT and BETAINV
  searches, T-2809's BINOM.INV count.

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns visibly-off cells of BINOM.INV (2), BETAINV (9), GAMMADIST (4), GAMMA.DIST (14) and F.INV.RT (15). Fixing them lifts BINOM.INV to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells. It is also the largest owner of BETAINV and GAMMADIST cells.
