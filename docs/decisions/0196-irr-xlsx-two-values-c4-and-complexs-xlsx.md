# 0196 — IRR.xlsx Two Values C4 and COMPLEXs.xlsx K25 are M14 desktop-only cells

- **Status:** accepted (2026-10-31, T-3410). It applies decisions 0039, 0087, 0134 and 0188 R6 to two more cells and
  weakens no gate.
- **Context:**
  - ironcalc-calc/FINANCIAL/IRR.xlsx Two Values!C4 `=IRR(A4:B4)`, flows {-4, 4} (exact root 0), caches
    2.220446049250313E-16 (2^-52). The engine gives 0. The workbook's app.xml names Microsoft Excel Online 16.0300,
    but Excel for the web today gives 0 for the same formula on the same cells: probes/t3410-irr-zero-root-imcos
    Calc!C4 (excel-graph, 2026-10-31), as do `IRR({-4,4})` C1, `IRR({-4,4},0.1)` C2, `IRR({-4,4},-0.5)` C8 and
    `IRR({-1,1})` C5; C7 `IRR({-4,4})=0` is TRUE. docs/research/oracle-calibration.md (T-0004) recorded the same cell
    as 2.2e-16 cached against 0 on excel-graph. The engine matches all 13 cells of the probe, including
    `IRR({4,-4})` = -2^-52 (C3) and `IRR({-2,1,1})` = -2.886801908630332E-12 (C6).
  - ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!K25 `=IMAGINARY(IMCOS(D25))`, D25 = "4+3i", caches Mac 16.0300's
    7.58155274274655. Excel for the web gives 7.58155274274654 (probe Calc!C9, C12; IMCOS C11 is
    "-6.58066304055116+7.58155274274654i"), and so does the engine. engineering.md already calls K25 a desktop/web
    difference, not a fitting target.
  - Both cells are visibly off, so they count against the M9 row's ironcalc-calc measure and the Phase 1 rate, and
    no engine change can fix them without leaving the web, which decision 0039 item 4 has the engine follow.
- **Decision:** The engine keeps the web values. Both cells are M14 desktop-only evidence, owned by T-1075 (items 23
  and 24). They get lines in eval/milestones/exit-exceptions.tsv (from M9) and eval/milestones/phase1-exceptions.tsv,
  as decision 0188 R6 did for COMPLEXs.xlsx N6, AL12 and AL26. They stay known failures in
  eval/baseline/ironcalc-calc.txt: no baseline edit, no `--allow-regressions`.
- **Consequences:**
  - The M9 ironcalc-calc exit count and the IRR and IMCOS Phase 1 rates no longer count these two cells.
  - If desktop Excel (Phase 4, T-1074) gives the web values, the cached values came from an older build and the
    lines stay. If it gives the cached values, T-1075 files an engine task under decision 0030's rules for
    web-versus-desktop conflicts.
