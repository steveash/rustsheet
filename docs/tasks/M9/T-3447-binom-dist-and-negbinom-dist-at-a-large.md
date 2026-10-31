---
id: T-3447
title: "BINOM.DIST and NEGBINOM.DIST at a large n: the 75 probe cells within 5E-12 of Excel but not 15 digits (Loader pmf and basym residue)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Every cell of probes/t3352-binom-negbinom-large-n is within 15 significant digits of Excel. T-3352 fixed the gross loss
(42 to 133 of 208 within 15 digits; all 208 within 5.2E-12 of the math), and this task owns the 75 cells left over.

## Context
- T-3352's Outcome and statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n: basym and Loader, within 1E-11 of the
  math (T-3352)".
- The pmf cells (past the walk) take `binom.rs` `loader_pmf` on `loader::excel_dbinom_exponent`. They are up to 4.4E-12
  from Excel, for example A117 `BINOM.DIST(499952565,1000000000,0.5,FALSE)` and A187
  `NEGBINOM.DIST(100028284,100000000,0.5,FALSE)` at 2.4E-12, while Excel is itself up to 4.1E-12 off the math.
  Hypothesis, not verified: the rounding of n·p into bd0 and the bd0 series' arithmetic at k around 1E8, which T-2586's
  model does not capture.
- The cdf cells are basym in doubles (`bratio/basym.rs`, Excel's arithmetic not identified), bfrac at a large shape, or
  walk tails. They are within 5E-13 of Excel. A48 `BINOM.DIST(600,600000,0.001,TRUE)` was within 15 digits on main
  (statrs) and is not on basym: ours 0.5108555229841736, Excel 0.5108555229841731.
- Cells (probes/t3352-binom-negbinom-large-n, Calc!):
  - pmf (57): A5, A9, A11, A19, A21, A25, A27, A31, A33, A37, A39, A45, A47, A51, A55, A57, A61, A63, A69, A73, A75, A79, A81, A83, A85, A87, A91, A93, A97, A99, A103, A105, A109, A111, A115, A117, A121, A123, A127, A135, A137, A139, A151, A153, A155, A163, A167, A171, A173, A175, A179, A187, A191, A197, A199, A201, A203
  - cdf (18): A6, A8, A12, A28, A34, A48, A52, A70, A88, A126, A128, A130, A138, A140, A142, A146, A174, A206
- Score against Excel first; the math reference (T-3352's scratch scorer: 50-digit continued fraction and log-gamma)
  only tells which side is noisier. Any fit follows decision 0188 R4.

## Acceptance criteria
- [ ] All 208 cells of probes/t3352-binom-negbinom-large-n are within 15 significant digits of Excel; no gated cell
      regresses.
- [ ] The Gate A floor on probes-numeric/t3352-binom-negbinom-sweep still holds (0 outside both).
- [ ] What was found is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity on cells already within 15 digits (Phase 1b, decision 0188 R5).
