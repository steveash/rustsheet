---
id: T-3158
title: "F.DIST density residue after M-final: own the sweep cells T-3034 left visibly off (about 18,700) and ship T-2586's power-term step"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2586]
defer: phase-1b
---

## Goal
`F.DIST(x,d1,d2,FALSE)` reaches visible parity (within half a unit of the 15th significant digit of Excel) on the
T-2220, T-2513 and T-2586 density sweeps in probes-numeric, beyond what T-3034's M-final port gave: the cells
M-final still misses, mostly where the power term P = c·ln x has |P| >= 4, move to within 15 digits once T-2586
identifies Excel's exponential step for this density.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
- Gap triage 2026-10-29 (opus refill). `scripts/exit_owners.py --all-off` on the 2026-10-29 report (run on
  5d089378, which predates T-3034's merge 12649586b) lists 35,991 visibly-off F.DIST density cells with no open
  owner, all in the 28 probes-numeric cases below. T-3034 (done) owned them and measured visibly-off F.DIST cells
  38,027 -> 18,722 on landing; its Outcome hands the rest to "T-2586's open power term", but T-2586 is a research
  task (no engine change) and no engine task picks the cells up. This task is that engine owner.
- Root cause (statistics.md "F density is not the beta density, R's df or a GAMMALN formula (T-2513)", round 4 and
  5 updates; T-3034's Outcome): M-final is at visible parity only for |c·ln x| < 4 (99-100%), about 87% for |P| in
  [4, 32) and 10-36% for |P| >= 32, plus a +-1.5-unit scatter at exact-product shapes and +3 to +5 units at d1 = 1,
  d <= 2. T-2586 item 6 is cracking the power term (candidates in its round-5 note: an exp2 split, x87
  F2XM1/FSCALE, value-space x^(a-1)·(1+r)^-h with per-factor pow roundings, probed at (2,2) side 1).
- Examples (pre-T-3034 report; re-read them with `make eval-show` first, they may have moved):
  `make eval-show CASE=probes-numeric/t2220-f-dist-sweep-pdf.case.json`, e.g. Calc!A1000
  `F.DIST(100,100,100,FALSE)` Excel 9.32516532636366E-73, ours 9.325165326364092E-73; Calc!A190
  `F.DIST(100,2,50,FALSE)` Excel 6.710886399999994E-19, ours 6.710886399999962E-19.
- Code: `excel_f_density` in crates/rustsheet-engine/src/functions/statistical/f_density.rs (T-3034); reuse
  `x87_exp` / `x87_log10` primitives, never copy them. Code must not read docs/ (decision 0066).
- Stale neighbour: T-2790 (todo) targets `f_pdf_accurate`'s 1 - y cancellation, code T-3034 removed; its 224 t2220
  cells are inside this task's t2220-f-dist-sweep-pdf count. Re-triage T-2790 (drop it if M-final settled its cells)
  in this task's first PR. T-2219, T-2642 and T-2647 keep the cells they name.

### Cases owned
Unowned visibly-off F.DIST density cells per case at the triage report (pre-T-3034; recount on main first):
- probes-numeric/t2586-r5-f-smallr-a: 5600
- probes-numeric/t2586-r4-f-power-a: 4360
- probes-numeric/t2586-r5-f-bigconst-a: 3838
- probes-numeric/t2586-r5-f-combine-a: 3464
- probes-numeric/t2586-r4-f-small-df-a: 1872
- probes-numeric/t2586-r4-f-brcomp-big: 1549
- probes-numeric/t2586-r5-f-bigconst-hold: 1419
- probes-numeric/t2586-r4-f-power-hold: 1273
- probes-numeric/t2586-r5-f-final-hold: 1220
- probes-numeric/t2586-r5-f-smallr-hold: 1151
- probes-numeric/t2586-r5-f-combine-hold: 968
- probes-numeric/t2513-r3-fdensity-sw: 865
- probes-numeric/t2586-r5-f-edges-a: 806
- probes-numeric/t2586-verify-r4-f-power-hold: 805
- probes-numeric/t2586-r4-f-brcomp-hold: 742
- probes-numeric/t2586-verify-r5-f-combine-hold: 701
- probes-numeric/t2586-verify-r5-f-bigconst-hold: 686
- probes-numeric/t2586-verify-r5-f-smallr-hold: 680
- probes-numeric/t2586-verify-r5-f-final-hold: 567
- probes-numeric/t2586-r5-f-edges-hold: 565
- probes-numeric/t2586-r4-f-small-df-hold: 543
- probes-numeric/t2586-verify-r4-f-brcomp-hold: 523
- probes-numeric/t2513-r3-fdensity-hold: 522
- probes-numeric/t2586-verify-r5-f-edges-hold: 443
- probes-numeric/t2513-verify-r3-fdensity-hold: 435
- probes-numeric/t2220-f-dist-sweep-pdf: 231
- probes-numeric/t2586-verify-r4-f-small-df-hold: 104
- probes-numeric/t2513-r3-fdensity-holdsw: 59

## Acceptance criteria
- [ ] Re-measured on main: the per-case visibly-off counts after T-3034 are recorded in the first Outcome line.
- [ ] Once T-2586 has a held-out-scored power-term model, `excel_f_density` uses it where it gains: F.DIST's
      visibly-off density cells in the cases above fall by the model's measured margin, and no gated suite loses
      a cell (`make eval-check`); regions where it loses keep M-final, with the comparison in the Outcome.
- [ ] No new known-failure lines in any baseline (`make eval-baseline`).
- [ ] statistics.md's F density entry says which form ships and the measured counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F cdf and tails (T-2508, T-2510); F.INV (T-2511, T-2767).
- Bit-exactness: last-digit leftovers go to a `defer: phase-1b` follow-up named in the Outcome.

## Visible-parity stage
Decision 0083 and decision 0114: done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value, or, if T-2586's power term is time-boxed without a model (decision 0056), when the gain it
does give has shipped and the rest is re-filed with the negatives cited. Stage `visible`: Excel's own density is
not correctly rounded here (its worst error on the T-2220 sweep is 7.8E-14), so only Excel's operation order gives
its digits.
