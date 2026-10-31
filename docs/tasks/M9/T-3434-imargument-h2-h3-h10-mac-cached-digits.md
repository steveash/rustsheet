---
id: T-3434
title: "COMPLEXs.xlsx IMARGUMENT H2, H3, H10: Mac-cached ATAN2 digits the web does not give; name them as M14 desktop-only cells (T-3410's remainder)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The three COMPLEXs.xlsx IMARGUMENT cells whose only gap is Mac desktop digits against Excel for the web have a recorded
ruling and an owner. Either the engine keeps the web value and the cells are named as M14 desktop-only evidence (they
stay known failures), or fresh web evidence shows the engine wrong and a functions-engineering task is filed. Today
they are known failures that no open task owns.

## Context
- Decision 0188 R5: parked as Phase 1b. All three cells are within 4 ulp of Excel and inside Gate A's envelope, so they
  do not count against a Phase 1 exit. R4.4 also forbids fitting them: no fitting below the oracle's reproducibility
  floor (decision 0112, web against desktop up to 4 ulp). Without a ruling they stay unowned.
- Gap triage 2026-10-31 (`make eval` on origin/main 4ff408243). Known failures in eval/baseline/ironcalc-calc.txt, no
  open owner:
  - ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!H2 `IMARGUMENT(D2)`: cached 0.46364760900080615, ours
    0.4636476090008061 (1 ulp).
  - Sheet1!H3 `IMARGUMENT(D3)`: cached -1.1071487177940906, ours -1.1071487177940904 (1 ulp).
  - Sheet1!H10 `IMARGUMENT(D10)`: cached 1.1071487177940906, ours 1.1071487177940904 (1 ulp).
- The workbook was saved by Macintosh Excel 16.0300. The T-0004 calibration recalculated it on Excel for the web, which
  gives ours in all three (docs/research/oracle-calibration.md, "All disagreeing cells"). docs/excel-semantics/
  engineering.md ("Remaining after T-0907") already calls them desktop/web differences.
- H3 and H10 are ATAN2's case. 1.1071487177940906 is the Mac digit that decision 0086 records for ATAN2_POWER.xlsx D26
  `ATAN2(TRUE,2)`, against the web's correctly rounded ...904. Decision 0086's consequences say a desktop ATAN sweep
  "would also cover IMARGUMENT if it shares the kernel" (T-1075 item 14).
- T-3410 (branch agent/research/T-3410-desktop-only-irr-complexs, decision 0196) names COMPLEXs K25 and IRR.xlsx C4 as
  M14 desktop-only cells. It leaves these three out of scope as "within 4 ulp, not visibly off". This task is that
  remainder: read 0196 once it has merged and extend it, or cite it from a new record.
- Precedents: decisions 0074 (IF_ARRAY SIN(4)), 0086 (ATAN2 D14, D26), 0188 R6 (IMTAN N6, IMSQRT AL12, AL26) and 0196.
  Decision 0039 item 4: web-vs-desktop conflicts match the web for now. eval/milestones/ is protected (decision 0044).

## Acceptance criteria
- [ ] Fresh web evidence: a probes case (excel-probe skill, excel-graph) with `IMARGUMENT` and `ATAN2` on the three
      cells' own inputs (D2, D3, D10 of COMPLEXs.xlsx, read with `rustsheet-eval snapshot`), filled. Ours passes it,
      or the task files a functions-engineering task for each cell it fails.
- [ ] If the web agrees with ours: a decision record (`make new-decision`, or an amendment of 0196 if it has merged)
      names the three cells as M14 desktop-only evidence, with their lines in eval/milestones/phase1-exceptions.tsv
      (and exit-exceptions.tsv only if the M9 measure ever counts within-15-digit cells). T-1075 item 14 (ATAN2) gains
      IMARGUMENT's question for Windows desktop. The cells stay known failures, with no baseline edit and no
      `--allow-regressions`.
- [ ] engineering.md's "Remaining after T-0907" bullet cites the probe and the decision.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting a Mac atan kernel (Phase 4, T-1075). COMPLEXs K25 and IRR.xlsx C4 (T-3410). IMTAN N6 and IMSQRT AL12, AL26
  (decision 0188 R6).
