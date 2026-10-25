# 0109 — Allow two BESSEL.xlsx Mac-desktop cells to regress to the excel-graph value

- **Status:** accepted (2026-10-24, T-2547)
- **Context:**
  - T-2547 gives the BESSELJ/BESSELY asymptotic form (x >= 8) Excel's own sine and cosine of the
    phase (`excel_sin_cos`, the x87 64-bit-pi reduction SIN and COS use) and Excel's `#NUM!` past
    the phase 134217727. That makes every cell of probes/bessel-large-x (24) and
    probes/t2547-bessel-phase-cut (88) bit-exact against excel-graph, and fixes 21 gen-m9 and 30
    probes known failures (docs/excel-semantics/engineering.md "BESSELJ / BESSELY").
  - Two cells of `ironcalc-calc/ENGINEERING/BESSEL.xlsx`, cached by Microsoft Macintosh Excel
    16.0300, passed before with libm's sine and now differ: All!D18 `BESSELJ(100000,5)` (cached
    0.001846551246313725, now 0.0018465512463137278, 13 ulp) and All!F18 `BESSELY(100000,5)`
    (cached 0.001719431947654535, now 0.0017194319476545319, 14 ulp). excel-graph gives exactly
    the new values for the same formulas (probes/t2547-bessel-phase-cut A85, A86), and matches the
    new kernel on all fourteen `BESSELJ/BESSELY(100000, 0..6)` cells (A75:A88). A Mac build has no
    x87 sine, so its reduction of the phase 99999.2 differs in the last digits.
  - Decision 0039 rule 4: excel-graph is the oracle of record for Phases 1-3, but moving a cell
    that passes against cached desktop evidence to the web value needs its own decision.
- **Options considered:**
  - *Keep libm's sine above x = 100.* It keeps two Mac-cached cells and leaves the 51 web-evidenced
    cells (gen-m9 and probes) failing, among them every visibly-off `#NUM!` cell of T-2547.
  - *Switch the sine by x.* No x splits the evidence: the conflict is at x = 1e5, where excel-graph
    is bit-exact with the new kernel on fourteen cells.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-2547 PR for
  exactly these two cells. T-1075 (M14, desktop probes) gets an item to settle Windows-desktop
  `BESSELJ(100000,5)` and `BESSELY(100000,5)`.
- **Consequences:** `eval/baseline/ironcalc-calc.txt` gains exactly the lines for
  `ENGINEERING/BESSEL.xlsx` All!D18 and All!F18; reviewers confirm the baseline diff adds no other
  line. If a Windows desktop probe later agrees with the Mac cache, the kernel question reopens
  under T-1075.
