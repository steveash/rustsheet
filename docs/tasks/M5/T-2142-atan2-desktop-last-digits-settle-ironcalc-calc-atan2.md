---
id: T-2142
title: "ATAN2 desktop last digits: settle ironcalc-calc ATAN2_POWER D14 and D26 (Mac desktop 0.58 ulp off, web and engine correctly rounded)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The two ironcalc-calc ATAN2 known failures, `MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx` Sheet1!D14 and D26, get a decided
target. Either a desktop ATAN2 formulation reproduces them without losing a web cell the engine passes today, or the
cells are recorded as an Excel-vs-Excel last-digit conflict the way decision 0074 recorded desktop SIN(4). They are
2 of the 14 ironcalc-calc cells that fail in M5's function scope (report.md "Milestone function scope", 2026-10-20),
and no open task owns them. T-2111 lists them as "no open task". T-2116 settles only IF_ARRAY C4/E4.

## Context
- Failing cells (gap triage 2026-10-20; desktop cache, Microsoft Macintosh Excel 16.0300):
  - D14 `ATAN2(A14,B14)`, x = 1, y = 3.141592653589793. Excel 1.2626272556789115, engine 1.2626272556789118.
  - D26 `ATAN2(A26,B26)`, x = TRUE, y = 2. Excel 1.1071487177940906, engine 1.1071487177940904.
  - Exact atan(pi) = 1.26262725567891167..., and exact atan(2) = 1.10714871779409050... (Python `decimal`, 60 digits).
    The engine's values are correctly rounded, 0.41 and 0.42 ulp from the exact value. The desktop values are
    0.59 and 0.58 ulp off. Both points are within 0.09 ulp of a rounding midpoint. That is the same pattern as
    desktop SIN(4), decision 0074.
  - The other 25 ATAN2 cells of the workbook (D2-D28; 20 of the 27 are numbers, the rest errors) pass. Excel for
    the web returns the engine's value at D14 and D26 (docs/research/oracle-calibration.md, "All disagreeing cells").
- Web evidence the engine already matches: probes/atan2-quadrants (123 numeric cells), atan2-error-edges,
  atan2-kept-subnormal, and column G of asin-atan-sign-source. numbers.md, "ATAN2 is ATAN(y/x) plus or minus pi":
  the model is glibc `atan` of Excel's quotient, implemented in `fn_atan2`
  (crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs).
- numbers.md, "Excel for the web and desktop Excel differ in the last digits", already lists ATAN2. Decision 0030
  rule 4 targets desktop values where they exist. Decision 0039 matches the web in Phases 1-3 when settling a
  conflict needs new desktop evidence.
- Candidates to try in the fit lab (`cd tools && uv run rustsheet-fitlab`, docs/architecture/fit-lab.md;
  `fitlab/exact.py` has the x87 model): x87 `FPATAN` of (y, x) with a 64-bit result stored to a double; `FPATAN` of
  the double quotient against 1; atan of the quotient rounded to 64 bits first; fdlibm/musl and Apple-libm-style
  `atan` (a faithful but not correctly rounded kernel, as the Mac SIN kernel turned out to be). Score each one on
  the 20 numeric desktop cells and the web probe cells.

## Acceptance criteria
- [x] numbers.md ("Excel for the web and desktop Excel differ in the last digits", ATAN2 paragraph added by the
      triage that filed this task) records a fit-lab run. For each candidate it gives the count on the 20 numeric desktop ATAN2 cells of
      ATAN2_POWER.xlsx and on the web cells of probes/atan2-quadrants, atan2-error-edges, atan2-kept-subnormal and
      asin-atan-sign-source G.
- [x] (n/a: no candidate qualifies) If one candidate keeps every web cell the engine passes today and also gives D14 and D26: `fn_atan2` uses it.
      Then those 2 known-failure lines disappear from eval/baseline/ironcalc-calc.txt, with no new ones in any
      gated suite, and IMARGUMENT is checked for the same kernel (numbers.md says it may share it).
- [x] Otherwise: a decision record (`make new-decision`) applies decisions 0030/0039/0074 to D14 and D26. The engine
      keeps the web value, numbers.md names the two cells as an Excel-vs-Excel conflict, and docs/milestones.md's
      M5 ironcalc-calc criterion names the exception if the record grants one. No baseline edit, no
      `--allow-regressions`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The IF_ARRAY SIN(4) cells (decision 0074, T-2116).
- IMARGUMENT's own known failures (T-2044), unless the shared kernel fixes them for free.
- Windows desktop evidence (M14, T-1075).

## Outcome
No candidate gives both desktop cells. The engine's model (a correctly rounded atan of Excel's quotient) scores 18/20 desktop
and 257/257 web (probes/atan2-quadrants 120, atan2-error-edges 17, asin-atan-sign-source G 120). fdlibm `atan` (19/248) and a
π/4 reduction (19/231) give D14 but not D26, and they lose web cells. numbers.md records all nine candidates. Decision 0086 keeps the
web value and names D14 and D26 as an Excel-vs-Excel conflict. Its M14 item is T-1075 item 14, and docs/milestones.md adds the
two cells to M5's ironcalc-calc exception. probes/atan2-kept-subnormal (16 cells) was not
scored by the fit: its inputs are formulas of subnormal differences. Every candidate takes the same tiny-quotient path there, and
the engine passes all 16. Eval delta: none (no engine change; the two baseline lines stay).
