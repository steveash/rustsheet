---
id: T-2774
title: "IMTAN of a real number takes TAN's kernel (blocked: COMPLEXs.xlsx N6 desktop cache, T-1075 item 17)"
milestone: M9
lane: functions-engineering
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-1075]
campaign: visible
---

## Goal
IMTAN of a real number (imaginary part 0) below 2^27 gives Excel's value: TAN's own kernel (`excel_tan`) on Excel for the
web, once the web-vs-desktop conflict on ironcalc-calc COMPLEXs.xlsx N6 is settled.

## Context
- Split from T-2534. IMTAN of a real is TAN on Excel for the web: `IMTAN("5")` -3.38051500624659 (35 gen-m9 cells),
  `IMTAN(123456789.123)` 56.4157574800738, `IMTAN(3.14159)` -2.65358979336212E-06 (probes/t2534-im-trig-real-limit D1,
  D8-D10; engineering.md "IM* trigonometry of a real number is SIN, COS and TAN"). The engine keeps the double-angle form
  `sin 2x / (cos 2x + 1)` with libm (`tan_parts` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs).
- Switching to `excel_tan` (one line: `real_trig(x, suffix, cell, None, excel_tan)` in `imtan`) regresses
  ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!N6 `IMREAL(IMTAN(D6))`, which caches Mac 16.0300's -3.38051500624658 and
  passes today. Decision 0039 keeps desktop evidence a cell passes against; moving it to the web value needs its own decision
  record and `make eval-baseline --allow-regressions`, which the T-2534 run could not take.
- Its open question is item 17 of T-1075 (desktop-only, decision 0039 rule 3: it blocks no milestone).

Owned cells (52):
- The 35 `=IMTAN("5")` cells of gen-m9: 01810fc89c5ec2c0 A18, A116; 060b18586fcff6e6 A16; 09e6a8028f5acfae A270;
  1068c44afcd56d53 A219; 3cf48801617a72ad A81; 408aa11aa923a4d9 A67, A114; 516488da78a6b08c A88, A182; 5b6cc12a94f7bf14 A220;
  5bdef392d16a5efb A324; 65dc4e4da669f98a A126, A314; 65ff9a692665eba9 A345; 6d16afd40600302b A53; 76b07b57099ccc8d A312;
  7820ebc42c7a56a6 A72; 7a819ffbbeff8fcc A215, A313, A362; 7c7e5bff12d93365 A35; 831f987e63e2412c A276; 8cbfd5e99d2e82da A271;
  9fa170a9f17624fd A97; a1b4181c04804efc A9, A254; b5aef3645cec1b95 A268; c2d778463fe008ed A256; ca182d96dedee546 A205;
  cb80ca09be0e12bc A118; e707d43a891ba9b5 A17; e85daedce5c2675e A109; f8bc18a0e45a0e1a A225; fb73d8163ad87fc6 A363.
- gen-m9/65ff9a692665eba9 A100, e882060cf4613302 A146, ef33be4fd2389f3c A225 (`IMTAN` of 123456789.123),
  79dac8dde81afda9 A154 (`IMTAN(3.14159)`).
- probes/t2534-im-trig-real-limit Sheet1!D1, D8, D9, D10.
- gen-m9/756a9f7bc3a06bcc Calc!A338 (`IMTAN` of Data!A12, "3.14159": Excel -2.65358979336212E-06, ours
  -2.65358979336207E-06), from T-3054, dropped into this task.
- probes/t3054-imtan-real-tan-kernel Sheet1!N1, A2, A4:A11 (T-3054, excel-graph, 2026-10-28): nine real arguments where TAN's
  kernel and the double-angle form print different text, all TAN's. Its N1 is N6's own formula,
  `IMREAL(IMTAN(COMPLEX(5,0,"i")))`, and the web gives -3.38051500624659 there, so the conflict is the platform, not the
  formula. Decision 0030 rule 4 (kept by decision 0039) lists IMTAN among the last-digit web-vs-desktop functions: the
  desktop N6 is the fitting target, and these web cells are not targets until T-1075 item 17 is answered.

## Acceptance criteria
- [ ] The owned cells leave the baselines, and N6 is either still passing or recorded per a decision record that reviewers
      approve (with the T-1075 item 17 answer, or a decision to follow the web now).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Complex arguments of IMTAN (the double-angle form stays).

## Blocked because
Web-vs-desktop conflict on IMTAN(5): see Context. Unblocks when T-1075 item 17 is answered, or a decision record
authorizes following the web for N6.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): IMTAN 1. This task now owns them:
- gen-m9/e85daedce5c2675e: Calc!A109
The added cell is `IMTAN("5")`, the real-number case this task is blocked on, so the same desktop answer (T-1075 item 17) settles it; it is owned, but not picked, until then.
