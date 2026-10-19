---
id: T-2004
title: "LOG10 and one-argument LOG: Excel's log10 kernel (correctly rounded hypothesis; LOG(1000) is 3)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
LOG10(x) and LOG(x) with the base omitted return Excel's bits: LOG(1000) is 3 and LOG10(1.3) is
0.11394335230683679, from one log10 kernel shared by both functions.

## Context
Gap triage 2026-10-19. The failing cells (`make eval-show CASE=ironcalc-calc/MATH_AND_TRIGONOMETRY/LOG_LOG10_LN.xlsx`,
desktop cache, Microsoft Macintosh Excel 16.x):
- `LOG!B5` `LOG(A5)`, A5 = 1000: expected 3, actual 2.9999999999999996.
- `LOG!B12` `LOG(A12)`, A12 = 0.004: expected -2.3979400086720375, actual -2.397940008672037.
- `LOG!B13` `LOG(A13)`, A13 = 1.3: expected 0.11394335230683679, actual 0.11394335230683676.
- `LOG10!B13` `LOG10(A13)`: expected 0.11394335230683679, actual 0.11394335230683678.
- Possibly related (web, excel-graph): `probes/geomean-root-models` Root!I6 and I9 `10^(LOG10(E)/C)`, one ulp
  each. They also go through `^`, so they are not acceptance cells. Scratch check: a correctly rounded log10
  makes I6 match with Python's `**` standing in for Excel's `^`.

Root cause, two parts:
1. `fn_log` (functions/math_and_trigonometry/mathematical.rs) computes `f64::log(x, 10.0)`, which is ln(x)/ln(10).
   Excel's one-argument LOG equals its LOG10 in every row of the workbook (B2:B14 of the LOG and LOG10 sheets are equal).
2. `fn_log10` calls the platform `f64::log10` (glibc), which is not correctly rounded. log10(1.3) is
   0.1139433523068367840..., so the double Excel returns (…679, 0.58e-17 away) is the correctly rounded one and
   glibc's (…678, 0.81e-17 away) is not. The MULTINOMIAL fit found the same thing on web evidence: Excel sums correctly
   rounded `log10(k)`, and glibc's log10 misses at k = 11, 40, 43, ... (numbers.md "MULTINOMIAL is inexact",
   `multinomial.rs` `log10_of`, which uses a table).
Hypothesis to test, not yet a rule: Excel's log10 is correctly rounded, or equivalently an x87 `FLDLG2; FYL2X` result
rounded once to a double. These two can differ only very near a rounding midpoint. See numbers.md "LOG with one argument
is LOG10" (added by this triage) and docs/research/excel-kernel-playbook.md.
The only evidence is one desktop cell where glibc is wrong (1.3). The web oracle has not been asked at any point where
glibc's log10 is not correctly rounded, so a short web probe has to come first (decision 0039: in Phases 1-3 the web
value is the target where the two disagree). This is why the task is opus.

## Acceptance criteria
- [ ] A `probes` case (excel-probe skill, oracle-filled on excel-graph) holds LOG10(x) and LOG(x) for the 13 workbook
      inputs, at least 20 doubles where glibc `log10` is not correctly rounded (the MULTINOMIAL k list is a start), and
      exact powers 10^k for k from -20 to 20. It records whether the web agrees with the desktop cell LOG10(1.3).
- [ ] The kernel that matches is in the engine, written once and used by LOG10, by one-argument LOG and by
      MULTINOMIAL's `log10_of` (or that table is kept, with the reason written in its comment). The 4 lines above leave
      `eval/baseline/ironcalc-calc.txt`, the new probe cells pass, and no baseline gains a line.
- [ ] numbers.md "LOG with one argument is LOG10" gets the probe evidence, the status VERIFIED (or the hypothesis
      rewritten from the evidence), and `Implemented in`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Two-argument LOG(x, b) with b other than 10. No cell fails today and there is no evidence for how it is computed.
  Add a few such calls to the probe if that is cheap, but fit them in a follow-up task.
- LN, which passes all 318 of its evidence cells.
- The Mac-only ATAN2/IMARGUMENT/IMABS last digits (decision 0030 conflict; T-1075).
