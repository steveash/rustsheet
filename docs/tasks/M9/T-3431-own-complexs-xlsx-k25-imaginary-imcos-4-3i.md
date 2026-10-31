---
id: T-3431
title: "Own COMPLEXs.xlsx K25: IMAGINARY(IMCOS(\"4+3i\")) caches Mac's 7.58155274274655 at a 15-digit tie where the web and ours print ...654 (T-1075 item and exit exception, decision 0188 R6)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!K25 has an owner and a recorded verdict: it is the one visibly-off
COMPLEXs.xlsx cell that no open task names, and it counts against the M9 ironcalc-calc exit measure (48 visibly off
against an allowance of 36 in the 2026-10-31 run). If Excel for the web confirms it prints ...654, as the calibration
says, K25 joins N6, AL12 and AL26 as a Mac-cache cell. It gets a T-1075 item and an exit-exception line, and the
engine stays as it is. If the web prints ...655, the IMCOS kernel is fixed instead.

## Context
- Gap triage 2026-10-31e (`make eval` on origin/main 4ff408243, then `make unowned-off`): K25 is the only
  unattributed visibly-off cell no open task owns. It is unattributed because it nests IMCOS in IMAGINARY.
- Cell: `=IMAGINARY(IMCOS(D25))`, where D25 is `COMPLEX(A25,B25,C25)`, A25 4 and B25 3, so the argument is "4+3i".
  The workbook caches 7.58155274274655 (Mac 16.0300). The engine gives 7.58155274274654 (verdict NumericClose,
  `make eval-show CASE=ironcalc-calc/ENGINEERING/COMPLEXs.xlsx`).
- Evidence that the web agrees with the engine: docs/research/oracle-calibration.md "All disagreeing cells" lists
  COMPLEXs.xlsx K25 with the Excel cache 7.58155274274655 and the excel-graph value 7.58155274274654.
  docs/excel-semantics/engineering.md already calls "IMARGUMENT H2, H3, H10 and IMCOS K25" desktop/web differences
  that are "not fitting targets until the desktop oracle exists". No task carries K25, though: T-1075 items 16 and 17
  cover AL12, AL26 and N6 only, and eval/milestones/exit-exceptions.tsv has no K25 line.
- The digits sit at a 15-digit tie. The double product `-SIN(4)*SINH(3)` is 7.581552742746545. The exact value is
  7.58155274274654435... (50-digit Taylor series, gap triage), so ...654 is also the correctly rounded 15-digit text.
  The Mac value is off both the web and the math. Decision 0188 R6 (as for N6, AL12 and AL26 under decision 0039
  item 4) has the engine follow Excel for the web.
- Related: probes/t2535-complex-text-tie (IM* text at a 15-digit tie), T-2535 (IMSIN and IMCOS through sinh and cosh,
  done).

## Acceptance criteria
- [ ] A filled probe cell (excel-graph) for `IMAGINARY(IMCOS("4+3i"))` and `IMCOS("4+3i")`, in a new or existing
      probes case. Its Excel values are quoted here and in engineering.md's "Remaining after T-0907" paragraph.
- [ ] If the web prints 7.58155274274654: T-1075 gets an item for K25 beside items 16 and 17 (the question for
      Windows desktop and the Mac value). eval/milestones/exit-exceptions.tsv gets
      `ironcalc-calc/ENGINEERING/COMPLEXs.xlsx!Sheet1!K25	M9	0188	T-1075` under the N6/AL12/AL26 comment, which is
      extended to name K25 (decision 0044 protects eval/milestones/; the line cites decision 0188
      R6, as the N6, AL12 and AL26 lines do). `make unowned-off` then no longer lists K25, and K25 stays a known failure in
      eval/baseline/ironcalc-calc.txt (no baseline line changes).
- [ ] If the web prints 7.58155274274655 instead: the IMCOS kernel in crates/rustsheet-engine/src/functions/engineering/
      gives it, and the K25 known-failure line disappears from eval/baseline/ironcalc-calc.txt with no new line in
      any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMARGUMENT H2, H3 and H10 (within 15 digits, bit-level desktop/web cells; Phase 1b).
- Probing desktop Excel (T-1074, T-1075).

