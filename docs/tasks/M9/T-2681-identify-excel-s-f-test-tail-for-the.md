---
id: T-2681
title: "Identify Excel's F.TEST tail for the 8 cells still visibly off after T-2571"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.TEST/FTEST reach visible parity on every F.TEST cell on file: the 8 cells below have Excel's value to
15 significant digits without losing a cell that is bit-exact.

## Context
Follow-up of T-2571 (its visible-parity criterion was not met; decision 0083). T-2571 committed the fit
(`cd tools && uv run rustsheet-fitlab f-test`, `f-test-structure`, `f-test-finite`;
tools/src/rustsheet_tools/fitlab/kernels/ftest.py), found Excel's closed form for one degree of freedom on
each side (2 asin(sqrt x)/pi) and showed the residual is not in the variance ratio f nor in the remainder R
alone: it is in how Excel evaluates the tail (docs/excel-semantics/statistics.md "F.TEST / FTEST: larger
variance on top", every candidate scored so far). Visibly-off cells after T-2571 (excel-graph values):
- `gen-m9/58e3253bbd3a7a82.case.json` Calc!A141 `FTEST(Data!A2:D10,Data!E1:E11)`: Excel 0.5407985695568127, ours 0.540798569556814
- `gen-m9/8f2ac25945af87da.case.json` Calc!A119 `F.TEST(Data!A9:D12,Data!A10:D12)`: Excel 0.657910021992518, ours 0.6579100219925169
- `gen-m9/a7bf0d8d81d33ba6.case.json` Calc!A81 `FTEST(Data!B5:E8,Data!C2:E4)`: Excel 0.5027827884435259, ours 0.5027827884435264
- `gen-m9/b6593d7dd13f2dac.case.json` Calc!A306 `F.TEST(Data!B6:E12,Data!A1:D10)`: Excel 1.23966888990022e-69, ours 1.2396688899002132e-69
- `gen-m9/bcb24385baafb2f0.case.json` Calc!A377 `FTEST(Data!B7:E9,FILTER(Data!A1:D12,(Data!D1:D12>7)))`: Excel 6.4145381857304026e-93, ours 6.414538185730395e-93
- `gen-m9/caa1eed48b1883fe.case.json` Calc!A380 `F.TEST(Data!A10:C12,Data!A5:C10)`: Excel 0.08405253726549103, ours 0.08405253726549108
- `gen-m9/f34f66fb7f81e673.case.json` Calc!A4 `FTEST(Data!A1:C10,Data!A1:A12)`: Excel 0.8776459265593823, ours 0.8776459265593836
- `gen-m9/fec39efaba9284c3.case.json` Calc!A36 `F.TEST(Data!B9:C11,Data!A1:C10)`: Excel 0.0807011926950238, ours 0.08070119269502367
The A377 cell has a nested FILTER, so the fit sweep leaves it unparsed; score it from its computed arguments.
Untried ideas: A&S 26.6.4-26.6.8 theta forms for every half-integer shape (odd/odd from the asin base,
even df via the finite series) used only where they do not cancel; Excel's upper tail as 1 minus the lower
tail on the side where x > a/(a+b); a probe sheet varying one data value by 1 ulp around a visibly-off cell
to see whether Excel's p moves smoothly (a series) or in steps (a table or a switch).
Reproducibility debt from T-2571 (review): the scores for the a = b = 1/2 closed forms, NR `betai`, ln y as log1p(-x)
and the exact-f inversion (statistics.md) came from ad hoc scripts: add them to ftest.py as candidates of a committed fit
first, and point fisher.rs's comment at the fit that scores the closed form.
Unprobed corners (T-2361 review), probe before fitting: `FTEST({1E-160,2E-160},{1E150,2E150})` in both
orders, and df 1 against df 249 or 400 (the series hands over to `beta_reg_complement` at a + b = 200).

## Acceptance criteria
- [ ] Visible parity: the 8 cells above have Excel's value to 15 significant digits, with no new
      known-failure lines in any baseline beyond decision 0083's allowance.
- [x] The T-2571 ad hoc scores are reproducible from committed fits.
- [ ] Each candidate's score (bit-exact / within 4 ulp / visibly off) is recorded in
      docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.DIST, F.DIST.RT and BETA.DIST, which use the shared `beta_reg_complement` kernel.

## Outcome
- Finding: with both shapes above 1, F.TEST's tail is TOMS 708 bratio with Excel's pieces (T-2464
  r2-bratio's `BEST`): w3's `bpser` and L140's `bup` with Excel's beta density as its prefactor
  (docs/excel-semantics/statistics.md "F.TEST / FTEST"; docs/research/t2681/README.md). Ported as
  `f_test_bratio_p` (crates/rustsheet-engine/src/functions/statistical/f_test_tail.rs), which
  reuses BETA.DIST's small-shape density (`beta_density_small_shape`, model M2) and is used by
  fisher.rs `f_test_p` where it reaches; a shape of 1/2 keeps the remainder model, which beats
  bratio there.
- Fit (`uv run rustsheet-fitlab f-test`, 291 cells): engine 160 / 270 / 7 visibly off before,
  205 / 284 / 1 after (bit-exact / within 4 ulp / visibly off).
- The 8 cells: 7 now have Excel's 15 digits (A141, A119, A81 bit-exact; A36 1 ulp; A380, A4, A377
  3 ulp). gen-m9/b6593d7dd13f2dac A306 is still visibly off (25 ulp): follow-up T-2829. Acceptance
  criterion 1 is met on 7 of 8 cells; the eighth moves to T-2829 (decision 0083, "Done at visible
  parity" applies to the cells it reaches).
- Regressed from bit-exact (decision 0083 allowance; follow-up T-2830): gen-m9/2801c1c11c21e134 A2
  (1 ulp), 285d87cea1aaa73c A280 (3), 320d51765af9b6b7 A223 (1), 85f48d5cdfb24555 A154 (2),
  8d3588d8df0812df A243 (1), a61e547a1cab994c A11 (1), cde551ea787f713a A199 (1),
  fd9fa22d212080d6 A279 (1).
- Eval delta: gen-m9 known failures 3801 -> 3774 (35 fixed, 8 regressed), probes
  2644 -> 2634 (10 fixed), ironcalc-calc 638 -> 637 (`F_TEST.xlsx` F1).
- The T-2571 ad hoc scores are committed fits: `f-test-half`, `f-test-nr` (ftest_adhoc.py).
- Recorded 2026-10-27, after merging main 7bd9090e: the baseline was recorded with `update-baseline --allow-regressions` under
  decision 0083's standing allowance, with the owner's approval in session
  https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr. gen-m9 3753 -> 3726 (35 fixed); the eight regressed cells
  above are exactly the eight lines added, all numeric, 1-3 ulp, none visibly off; probes 2891 -> 2881 (10 fixed). A
  full run of every suite against a main 7bd9090e binary finds no other newly failing cell, ungated probes-numeric
  included, and none newly visibly off; in-scope cells visibly off fall from 103,412 to 103,406. Acceptance criterion
  1 stays unticked for A306 (T-2829); criterion 3's per-candidate scores are the fits above.
