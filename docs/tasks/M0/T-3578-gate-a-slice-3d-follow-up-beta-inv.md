---
id: T-3578
title: "Gate A slice 3d follow-up: BETA.INV s1 table (excel-graph refuses the generated workbook) and the GAMMA.INV/CHISQ huge-shape batch with its R3 points"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The rest of T-3529: a filled BETA.INV s1 Gate A table, and a huge-shape batch for GAMMA.INV, CHISQ.INV and
CHISQ.INV.RT with the decision 0188 R3 points of "GAMMA.INV at a huge shape, closed-form branch" marked.

## Context
- T-3529 landed the definitions of all five functions (`tools/src/rustsheet_tools/accuracy/defs/gamma.py`,
  `beta.py`) and filled tables for GAMMA.INV, CHISQ.INV, CHISQ.INV.RT and BETA.DIST.
- BETA.INV: `rustsheet-accuracy generate BETA.INV` writes 194 points, but `rustsheet-oracle fill -b excel-graph`
  fails at the workbook's createSession three runs in a row (retries exhausted), where the other four tables filled
  in the same runs. Excel has refused workbooks before (statistics.md, FileCorruptTryRepair in T-2435 and T-2982).
  Find the offending formula(s), probably by bisecting the batch, and record the refusal as a quirk.
- Huge shapes: the s1 batches stop at a shape of 1E7 (df 2E7), because the generator's series and continued fraction
  for the incomplete gamma need about 20·sqrt(a) terms. mpmath's `gammainc` gives NoConvergence from a shape of about
  1E5. The R3 region starts at shape 1E9 (closed form from about 2E10). It needs a uniform asymptotic expansion
  (Temme, DiDonato and Morris 1986) in the generator at 50 digits, cross-checked against the series where both run.
  The points then need marks in `math-over-excel.tsv` (protected: a decision record citing 0188 R3's huge-shape
  case), and the engine's points are known failures of T-3364 until it lands.
- The BETA.INV deep-tail R3 points ("Tiny-p quantiles") need the BETA.INV table. CHISQ.INV.RT's tiny-p points in
  T-3529's s1 batch did not show Excel more than G = 1e-4 off the math (see T-3529's Outcome).

## Acceptance criteria
- [ ] BETA.INV s1 filled and committed (or a decision on the refused formulas, recorded in docs/excel-semantics).
- [ ] A huge-shape batch for the three gamma quantiles, filled, with the R3 points marked and cited.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (their owner tasks).
