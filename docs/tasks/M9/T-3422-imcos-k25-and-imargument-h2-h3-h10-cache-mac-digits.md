---
id: T-3422
title: "COMPLEXs.xlsx K25 (IMCOS) and H2, H3, H10 (IMARGUMENT) cache Mac 16.0300 digits the web and the engine do not give: record the conflict and the exit exception"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The four COMPLEXs.xlsx cells where the workbook's Mac 16.0300 cached value differs from what Excel for the web gives
are handled the way decision 0039 item 4 and its precedents handle such cells: the engine keeps the web value, the
conflict is recorded, the desktop question becomes a T-1075 item, and the visibly-off K25 stops counting against
the M9 exit and Phase 1 measures through a decision record. This follows decision 0086 (ATAN2_POWER D14, D26) and
decision 0188 R6 (COMPLEXs N6, AL12, AL26).

## Context
- Gap triage 2026-10-31c (`make eval` on b1bdc1c2d): K25 is one of the visibly-off cells attributed to no function
  with no open owner (`make unowned-off`), and the only ironcalc-calc row that `make exit-owners MILESTONE=M9` lists.
  H2, H3 and H10 are within 15 digits (1 ulp) and also had no owner.
- Cells (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx, Mac 16.0300 cache; web values from docs/research/oracle-calibration.md
  "All disagreeing cells", T-0004, excel-graph; engine from this triage's eval):
  - ironcalc-calc/ENGINEERING/COMPLEXs.xlsx (4): Sheet1!K25, H2, H3, H10
  - K25 `IMAGINARY(IMCOS(D25))`, D25 = "4+3i": Mac 7.58155274274655, web 7.58155274274654, ours 7.58155274274654
    (visibly off: IMCOS prints 15 digits, so a last-bit difference in `sin 4 * sinh 3` flips the 15th digit).
  - H2 `IMARGUMENT(D2)`, "12+6i": Mac 0.46364760900080615, web and ours 0.4636476090008061.
  - H3 `IMARGUMENT(D3)`, "1-2j": Mac -1.1071487177940906, web and ours -1.1071487177940904.
  - H10 `IMARGUMENT(D10)`, "1+2i": Mac 1.1071487177940906, web and ours 1.1071487177940904. This is the same ATAN2
    value as ATAN2_POWER D26 (decision 0086, T-1075 item 14).
- docs/excel-semantics/engineering.md already calls H2, H3, H10 and K25 "desktop/web differences ... not fitting
  targets until the desktop oracle exists" (the T-0907 remainder paragraph), but no record or task owns them, and no
  exception line covers K25.
- eval/milestones/exit-exceptions.tsv and phase1-exceptions.tsv are protected (decision 0044): a line is added only in
  the PR that takes the decision naming it.

## Acceptance criteria
- [ ] Fresh excel-graph evidence for the four cells' formulas on their inputs (a probes case, M9 tag, e.g.
      `IMAGINARY(IMCOS("4+3i"))`, `IMARGUMENT("12+6i")`, `IMARGUMENT("1-2j")`, `IMARGUMENT("1+2i")`) confirms the
      calibration's web values. If the web now gives the Mac values, stop and re-scope this task to fit them.
- [ ] A decision record (`make new-decision`), following 0086 and 0188 R6, names the four cells, keeps the engine on
      the web value under decision 0039 item 4, and adds K25 to eval/milestones/exit-exceptions.tsv (M9) and
      eval/milestones/phase1-exceptions.tsv in the same PR. The cells stay `F` lines of eval/baseline/ironcalc-calc.txt.
- [ ] T-1075 gets an item for IMCOS K25 and IMARGUMENT H2, H3, H10 (the desktop question: does Windows desktop give the
      Mac digits?).
- [ ] engineering.md's entry is `Status: CONFLICTING` for these cells, with both values, the probe and the decision.
- [ ] `make exit-owners MILESTONE=M9` no longer lists COMPLEXs K25; `make check` green; reviewers approve.

## Out of scope
- Fitting Mac digits (Phase 4, M14).
- The other COMPLEXs cells already covered by decision 0188 R6 (N6, AL12, AL26).

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when K25 no longer counts as a visibly-off cell against M9 and Phase 1
under a decision record, with the web value confirmed, or K25 matches Excel for the web if the web value has changed.
