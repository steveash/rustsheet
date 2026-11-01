---
id: T-3740
title: "BINOM.DIST.RANGE past the mean: Excel's exact upper-tail form (21 probe rows within 6E-14)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BINOM.DIST.RANGE ranges whose lower end is past the mean are bit-exact with Excel (or within its 1-ulp noise) on the
21 probe rows where Excel's range is not its own BINOM.DIST cdf difference.

## Context
- Found by T-3444 (2026-11-01). probes/t3444-binom-range-inv-large-n Range rows 6, 12, 18, ..., 126 (every sixth row,
  ranges from mean + 3 sd to mean + 5 sd, n 1E4 to 2^31 - 2) differ from Excel's own
  `BINOM.DIST(s2,...)-BINOM.DIST(s1-1,...)` in column B by 2E-16 to 6E-14. statistics.md "BINOM.DIST.RANGE and
  BINOM.INV at a large n sit on BINOM.DIST's own cdf (T-3444)".
- T-3444 takes the upper-tail difference P(X > s1 - 1) - P(X > s2) (`binom.rs` `excel_binom_sf`, bratio's w) past the
  double-double walk: 18 of the 21 rows land within 1E-15 relative, none bit-exact; A6 and A12 (n = 1E4) stay 1.6E-14
  and 2.4E-14 off, A54 9E-15; A18 and A36 are walked and stay 1E-16 and 4.6E-15 off.
- Cells (probes/t3444-binom-range-inv-large-n, Range!): A6, A12, A18, A24, A30, A36, A42, A48, A54, A60, A66, A72, A78, A84, A90, A96, A102, A108, A114, A120, A126
- Candidates to score: the tails' other bratio branch (w1 complement), a pmf sum over the range from the mode side, a
  threshold other than the mean (the mode, cdf(s1 - 1) > 0.5), and the walk's behavior at n = 1E4 to 1E5. Score them
  against the 21 rows plus the 105 cdf-difference rows (none may move off). A choice among ranked forms owes the
  decision 0188 R4 hold-out and fresh sweep (work-loop step 4, "Numeric fits").

## Acceptance criteria
- [ ] The 21 rows' Range!A cells pass, or the remaining misses are characterized in statistics.md with the forms tried.
- [ ] No other cell of probes/t3444-binom-range-inv-large-n or any gated case regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST's own last digits (T-3447), which the other failing Range and Inv cells of the probe carry.
