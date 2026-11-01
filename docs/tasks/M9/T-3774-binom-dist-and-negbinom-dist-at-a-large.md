---
id: T-3774
title: "BINOM.DIST and NEGBINOM.DIST at a large n: Excel's unidentified |d| 2^-53 bd0 term and the basym last digits (T-3447 fit attempt 2)"
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
Every cell of probes/t3352-binom-negbinom-large-n is within 15 significant digits of Excel. This is T-3447's second fit
attempt, which owns its cells after attempt 1 (decision 0056).

## Context
- Read first: T-3447's "Fit attempts" section (candidates and counts), statistics.md "BINOM.DIST and NEGBINOM.DIST at a
  large n: basym and Loader, within 1E-11 of the math (T-3352)", and `docs/research/t3447/pmf_regress.py`.
- Attempt 1's finding for the pmf cells, which take `binom.rs` `loader_pmf` on `loader::excel_dbinom_exponent`:
  - Excel's off-mean error is about |d| 2^-53, with d = k - n p.
  - It contains the engine's terms: fl(n p), fl(n q), and -r(fl(2kv)) with v stored.
  - It also has a rest of rms 0.77 |d| 2^-53, partly smooth along runs of consecutive p.
  - Rounding v, k + M, s v, M v or d v, or computing fl(2kv) on a 64-bit v, does not explain the rest.
  - Find that term: a regressor whose coefficient is near ±1 and that brings the residual well below 0.77. Then put it
    in `loader::excel_bd0` (shared with BETA.DIST's density and POISSON.DIST, so their gated cells must not regress).
- The cdf cells are basym in doubles (`bratio/basym.rs`, Excel's arithmetic not identified), bfrac at a large shape, or
  walk tails. They are within 5E-13 of Excel and were not attempted. A48 `BINOM.DIST(600,600000,0.001,TRUE)` was within
  15 digits on statrs and is not on basym: ours 0.5108555229841736, Excel 0.5108555229841731.
- Cells (probes/t3352-binom-negbinom-large-n, Calc!):
  - pmf (57): A5, A9, A11, A19, A21, A25, A27, A31, A33, A37, A39, A45, A47, A51, A55, A57, A61, A63, A69, A73, A75, A79, A81, A83, A85, A87, A91, A93, A97, A99, A103, A105, A109, A111, A115, A117, A121, A123, A127, A135, A137, A139, A151, A153, A155, A163, A167, A171, A173, A175, A179, A187, A191, A197, A199, A201, A203
  - cdf (18): A6, A8, A12, A28, A34, A48, A52, A70, A88, A126, A128, A130, A138, A140, A142, A146, A174, A206
- Score against Excel first; the math only tells which side is noisier.
- `pmf_regress.py` needs mpmath (tools/.venv). It does not yet contain two things from attempt 1: the candidate table's
  scorer (104 pmf cells of probes/t3352) and the 64-bit-v fl(2kv) regressor. Add them when extending it. Helpers like
  its `rnd`, `stirlerr` and `bd0` also exist in docs/research/t2464/r3-loader-bd0/r3lib.py.
- If this attempt also fails to raise the bit-exact count, block it as "time-boxed (decision 0056)".

## Acceptance criteria
- [ ] All 208 cells of probes/t3352-binom-negbinom-large-n are within 15 significant digits of Excel; no gated cell
      regresses (BETA.DIST density and POISSON.DIST share `excel_bd0`).
- [ ] The Gate A floor on probes-numeric/t3352-binom-negbinom-sweep still holds (0 outside both).
- [ ] Decision 0188 R4 guards for any fit, recorded in the Outcome:
  - a hold-out split fixed before scoring;
  - a fresh sweep after the freeze;
  - the Gate A envelope result;
  - a `FITTED` tag and range;
  - the reproducibility floor;
  - no keep-matched branch.
- [ ] What was found is in docs/excel-semantics/statistics.md, and the attempt (candidates and counts) is in this file.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity on cells already within 15 digits (Phase 1b, decision 0188 R5).
