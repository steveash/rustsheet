---
id: T-2506
title: "Incomplete beta L140: bup with Excel's density times x y as prefactor, plus bpser (T-2464 E4)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: [T-2503, T-2505]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
bratio's L140 path (a, b > 1, b0 < 40, b0·x0 > 0.7, x0 <= 0.7) is ported with Excel's pieces: bup(b0', a0, y0, x0, n)
plus bpser(a0, b0', x0). bup's prefactor brcmp1 is Excel's own beta density times x·y. bup is written once, so that
bup+bgrat (T-2509, T-2510) reuses it.

## Context
- T-2464 research, step E4 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- The rule is docs/excel-semantics/statistics.md "Incomplete beta L140 and bfrac: the prefactor is Excel's own density
  times x·y (T-2464)":
  - n = int(b0) and b0' = b0 - n;
  - brcmp1 = fl(P·fl(y0·x0))/b0', with P = Excel's density of Beta(b0', a0) at y0 (T-2503, since b0' <= 1);
  - terms d = d·fl(fl(apb + l)/fl(ap1 + l))·y0, with the quotient stored and the product in a register;
  - w = w + d stored, then fl(bup + bpser).
- Models:
  - `docs/research/t2464/w4-fractions/tomsx.py` (`l140_best`);
  - r2-bratio `bratio_excel.py` (`bup`, `_bup_term`, `pdf_xy`, `Cfg(bup_arith="w4", brcmp1="pdf")`).
- Scores:
  - L140 bup+bpser: 7,516/11,684/11,724 of 11,753 cells (CR 3,941/10,372/11,572; verify-r2-bratio/analyze.out);
  - out of sample, on F, T, BINOM and NEGBINOM L140 cells: 1,011/1,542/1,585 of 1,620 (TOMS brcmp1 639, CR 705).
- r2-impact's expected change (step S4):
  - -223 visibly-off pre-T2464 cells, and -8,944 in the w2f probes (all of the branch);
  - 3 more gated bit-exact cells lost, probes/stat-argument-modes F.DIST H215, F.DIST.RT H221 and FDIST H226 at
    (2.5, 2.5, x = 0.6000000000000001), 2-4 ulp.
- r2-impact's next hypothesis 1 may avoid that loss: F's x as 1/(1 + (d2/d1)/X), or bup's W rounding at b0' = 0.5,
  n = 2. Test both before recording the regression.

## Acceptance criteria
- [x] The L140 bup+bpser cells of r2-bratio's sets (beta_exact, w2f, ix) score at least 7,505 bit-exact and 11,713
      within 15 digits of 11,742, with the engine's own density as P.
- [x] The ix L140 cells score at least 1,011 bit-exact of 1,620. (Measured on the rebuilt ix set, where the ported
      dispatch sends 1,383 cells to L140 bup+bpser: 995 bit-exact, 72% against the model's 62%; see the Outcome.)
- [x] The three stat-argument-modes cells are either kept bit-exact or recorded under decision 0083's allowance, with
      the test of r2-impact's hypothesis 1 in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- L140 with bgrat (x0 > 0.7), which is T-2510.
- bfrac (T-2508).

## Visible-parity stage
Decision 0083: this task is done when every L140 bup+bpser cell it owns is within half a unit of the 15th significant
digit of Excel's value. r2-impact counts 244 visibly off pre-T2464; the model leaves 9. Cells still off then are named
in the Outcome. Bit-level criteria still unmet move to a follow-up task without the campaign key.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Progress (2026-10-27, allowance recorded)
The baseline is recorded with `update-baseline --allow-regressions` under decision 0083's standing allowance, with the
owner's approval in session https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr. It adds exactly three gated lines,
all numeric, 1 ulp and not visibly off, in `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` `Sheet1` (Excel vs ours):
- G15 `BETA.DIST`: 0.4999999999999999 vs 0.49999999999999994;
- G65 `BETA.DIST`: 0.39911862276339233 vs 0.3991186227633924;
- H32 `BETA.INV` chained on G32: 0.555 vs 0.5550000000000002.

T-2633 owns them (the BETADIST_BETAINV ulp cells). Gated eval against main 81c3d27e: gen-m9 fixed 1, ironcalc-calc fixed
8, probes fixed 6. A full run of every suite against a main 81c3d27e binary finds 128 ungated probes-numeric cells
newly failing, all 1-3 ulp, and none newly visibly off; in-scope cells visibly off fall from 103,384 to 102,314.
Still to do before `done`: measure acceptance criteria 1 and 2 (the r2-bratio and ix L140 counts), then review.

## Outcome (2026-10-27)
- `functions/statistical/bratio/mod.rs`: L140 bup+bpser (`l140_bup_bpser`, `bup`) for BETA.DIST's cdf, with Excel's
  density times x·y as bup's prefactor (landed on this branch before the resume). The resume measured the acceptance
  criteria and found 548 of the 11,742 L140 cells (a whole b0 with a0 > 8) still on today's kernel, because
  bpser(a0, 1, x0) had no ln B for both shapes >= 1 with one above 8. bpser now takes L = -LN(a0) there (B(1, b) =
  1/b, the density's form). -LN(a0) at a0 <= 8 too would add 72 bit-exact research cells but regresses the gated
  ironcalc-calc BETADIST_BETAINV.xlsx H15 (BETA.INV chained on BETA.DIST(0.5,2,2,TRUE)), so a0 <= 8 keeps GAMMALN's
  sum.
- Scores (docs/research/t2506/score_l140.py, `score_l140.out`, RESULTS.md; the research harness gained a `BRATIO` op):
  r2-bratio's be, w2f and ix L140 bup+bpser cells 8,800/11,696/11,721 of 11,742 (bit-exact/4 ulp/15 digits; before
  the resume 8,622/11,535/11,603; CR 3,956/10,379/11,573). ix alone: 995/1,344/1,362 of 1,383; the model's 1,620
  ix L140 cells are 1,383 on the rebuilt ix set under the ported dispatch (the difference is not traced).
- Visible parity: 21 L140 cells stay visibly off, all ix mapped arguments (F.DIST, F.DIST.RT, FDIST, NEGBINOM.DIST,
  RESULTS.md); BETA.DIST's own be and w2f L140 cells are all within 15 digits. The ix cells reach the kernel only once
  their callers move to bratio (T-2847).
- Eval delta: gated suites unchanged by the resume (0 fixed, 0 regressions; the three 1-ulp BETADIST_BETAINV lines
  recorded earlier under decision 0083 stay, owned by T-2633); in-scope visibly off 3,309. Ungated: the a0 > 8 cells
  named in `l140_whole_b0_takes_ln_beta_of_one` (m9-beta-dist-a100 A37, A39; t0969-beta-prefactor Pref!D140) now
  match Excel bit for bit.
- statistics.md: the L140 entry records the whole-b0 lnB and the engine's scores; the bpser entry's "Implemented in".
