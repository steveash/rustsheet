---
id: T-2829
title: "F.TEST gen-m9/b6593d7dd13f2dac A306: the one cell bratio leaves visibly off (T-2681)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2681]
campaign: visible
---

## Goal
`F.TEST(Data!B6:E12,Data!A1:D10)` (gen-m9/b6593d7dd13f2dac Calc!A306) has Excel's value
1.23966888990022E-69 to 15 significant digits, without losing a bit-exact F.TEST cell.

## Context
After T-2681 F.TEST's tail for both shapes above 1 is bratio's `bpser` with Excel's pieces
(fisher.rs `f_test_p`, f_test_tail.rs); this cell (a = 6, b = 11.5, x = 6.29E-13) is the only
F.TEST cell on file still visibly off, 25 ulp (docs/research/t2681/README.md "The 8 cells").
Its data hold 123456789.123 next to values near 1. Excel's p is 19 ulp from the exact tail at the
two-pass f; the exact variances move the model's p by 3 ulp only; the z that would give Excel's
p is 48 units (2^-53) from bratio's, about eight ulps of x. Ideas: how Excel forms x from the two
variances when one is 8E11 times the other (x = df_s/(df_b f + df_s) against df_s v_s/(df_b v_b +
df_s v_s), and the variance sums in x87 registers); probe the same shapes with the data scaled.
Use `docs/research/t2681/score_bratio.py` to score.

## Acceptance criteria
- [ ] A306 within half a unit of the 15th significant digit of Excel's value, or the cause
      recorded in docs/excel-semantics/statistics.md with the probe that shows it.
- [ ] No new visibly-off F.TEST cell (`uv run rustsheet-fitlab f-test`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells 1 to 3 ulp off (T-2830).
