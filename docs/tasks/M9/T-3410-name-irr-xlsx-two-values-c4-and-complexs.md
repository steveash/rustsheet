---
id: T-3410
title: "Name IRR.xlsx Two Values!C4 and COMPLEXs.xlsx K25 as M14 desktop-only cells (engine = web, the cache differs)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The two visibly-off ironcalc-calc cells whose only open question is which Excel build produced the cached value have
an owner and a recorded ruling: fresh excel-graph probes confirm that Excel for the web gives the engine's value, a
decision record names both cells as M14 desktop-only evidence (as decisions 0074, 0086 and 0188 R6 did for IF_ARRAY,
ATAN2_POWER and COMPLEXs N6/AL12/AL26), and they leave the M9 ironcalc-calc exit count and the Phase 1 rate while
staying known failures.

## Context
- Gap triage 2026-10-31c (opus refill). `make exit-owners MILESTONE=M9` lists COMPLEXs.xlsx K25 with no open M9 owner;
  IRR.xlsx Two Values!C4 is visibly off (0 against nonzero) and no open task names it (T-2131 and T-2928 own L17 and,
  through it, M17 of IRR.xlsx, not C4). Both are in the M9 row's ironcalc-calc count (82 visibly off against an
  allowance of 36 on this run).
- The two cells (eval, 2026-10-31):
  - ironcalc-calc/FINANCIAL/IRR.xlsx Two Values!C4 `=IRR(A4:B4)`, flows {-4, 4} (exact root 0): cached
    2.220446049250313E-16 (2^-52), ours 0. docs/research/oracle-calibration.md (T-0004, 2026-10-04) lists the same cell
    with excel-graph giving 0, "two-flow IRR 2.2e-16 vs 0", one of its "both are Excel" last-digit groups. The workbook's
    app.xml says Microsoft Excel Online 16.0300, so which build computed the cached value is not certain.
    Triage note (scratch, not evidence): the shipped secant (financial.md "IRR is a secant in d = 1 - v") run in plain
    doubles from guess 0.1 stops at d2 = 8.3E-17 and returns 1/(1 - d2) - 1 = 2^-52, the cached value; the engine's
    twice-rounded quotient (`irr_from_discount`, `extended_quotient`) turns 1/(1 - 2^-53) into a tie that rounds to 1,
    so 0, the web's value. That is consistent with a build without the web's double rounding computing the cache.
  - ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!K25 `=IMAGINARY(IMCOS(D25))`, D25 = COMPLEX(4,3) = "4+3i": cached
    7.58155274274655 (Microsoft Macintosh Excel), ours 7.58155274274654, the excel-graph value in oracle-calibration.md.
    engineering.md "IMTAN and IMCOT use the double-angle form" already calls K25 a desktop/web difference and "not a
    fitting target until the desktop oracle exists"; only the exception lines and an M14 item are missing.
- Precedent: decision 0188 R6 put COMPLEXs N6, AL12 and AL26 in eval/milestones/exit-exceptions.tsv (M9) and
  eval/milestones/phase1-exceptions.tsv with T-1075 as owner; decision 0134 says how phase1-exceptions lines are added
  (only in the PR that takes the decision naming the cell; decision 0044 protects eval/milestones/).
- Cells:
  - ironcalc-calc/FINANCIAL/IRR.xlsx (1): Two Values!C4
  - ironcalc-calc/ENGINEERING/COMPLEXs.xlsx (1): Sheet1!K25
- The other COMPLEXs desktop/web cells (IMARGUMENT H2, H3, H10) are within 4 ulp, not visibly off: out of scope.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes/cases/ (excel-graph filled, milestone tag M9) holds at least `IRR({-4,4})`,
      `IRR({-4,4},0.1)`, `IRR({4,-4})`, `IRR({-1,1})`, `IRR({-2,1,1})` (an exact zero root of three flows) and
      `IMAGINARY(IMCOS("4+3i"))`, `IMREAL(IMCOS("4+3i"))`; its Context lists the inputs. If the web value of either
      ironcalc cell is no longer the engine's, stop and re-file the cell as an engine task instead.
- [ ] A decision record (`make new-decision`) names IRR.xlsx Two Values!C4 and COMPLEXs.xlsx Sheet1!K25 as M14
      desktop-only evidence, citing the probe cells and oracle-calibration.md; the same PR adds their lines to
      eval/milestones/exit-exceptions.tsv (milestone M9) and eval/milestones/phase1-exceptions.tsv, owner T-1075.
- [ ] T-1075 gains an item for each (the question to ask desktop Excel), and financial.md gets an entry for IRR at an
      exact zero root (web 0, the cache 2^-52; the plain-double secant reproduces the cache) citing the probe cells.
- [ ] The M9 row of report.md no longer counts the two cells as ironcalc-calc visibly off; both stay in
      eval/baseline/ironcalc-calc.txt (no baseline edit).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing IRR's final quotient or IMCOS: the engine follows the web (decision 0039 item 4) unless the probe shows the
  web gives the cached value.
- IRR.xlsx General!L17 and M17 (T-2928, T-2131).
