# 0086 — Desktop ATAN2 last digits in ATAN2_POWER stay a web-vs-desktop conflict

- **Status:** accepted (2026-10-20, T-2142). It applies decisions 0030, 0039 and 0074 to two more
  cells and weakens no gate.
- **Context:**
  - `ironcalc-calc/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx` was saved by Microsoft Macintosh Excel
    16.0300. Its Sheet1!D14 `ATAN2(A14,B14)` (x = 1, y = 3.141592653589793) caches
    1.2626272556789115, and D26 `ATAN2(A26,B26)` (x = TRUE, y = 2) caches 1.1071487177940906.
    Excel for the web gives ...118 and ...904 (docs/research/oracle-calibration.md), and so does
    the engine. Both quotients are exact, so the difference lies in the atan kernel. The web values
    are correctly rounded (0.41 and 0.42 ulp off), and the desktop values are 0.59 and 0.58 ulp
    off. Both points are within 0.09 ulp of a rounding midpoint, the same shape as desktop SIN(4)
    in decision 0074.
  - They are 2 of the 14 failing cells in the M5 row of "Milestone function scope"
    (`make eval`, 2026-10-20). Decision 0085 item 2 left them to this task.
  - T-2142 compared candidates on the workbook's 20 numeric desktop ATAN2 cells and on 257 web cells
    (probes/atan2-quadrants, atan2-error-edges, asin-atan-sign-source G). numbers.md, "Excel for
    the web and desktop Excel differ in the last digits", lists every candidate's score. The
    engine's model (a correctly rounded atan of Excel's quotient) scores 18 desktop and 257 web.
    fdlibm/musl `atan` and a π/4 + atan((q−1)/(q+1)) reduction both reproduce D14, but neither
    gives D26, and they score only 248 and 231 web. No candidate gives both desktop cells.
- **Decision:**
  - The engine keeps the web value for ATAN2.
  - D14 and D26 are named in numbers.md as an Excel-vs-Excel last-digit conflict. They stay in
    eval/baseline/ironcalc-calc.txt as known failures: no baseline edit and no
    `--allow-regressions`.
  - Under decision 0039 rule 3, their only open question is desktop evidence. That question is
    item 14 of the M14 task T-1075, and that item counts as their filed task. So they join M5's
    ironcalc-calc exception in docs/milestones.md next to IF_ARRAY C4, E4 and C11 (decision 0085
    item 2).
- **Consequences:**
  - M5's ironcalc-calc exception now lists five cells. The M5 function-scope row needs 0 failing
    cells apart from those five and the time-boxed cells of decision 0085 item 3.
  - When Phase 4 starts, T-1075 item 14 settles the target. If Windows desktop gives the Mac
    values, a desktop ATAN sweep near rounding midpoints gets its own functions-math task. That
    sweep would also cover IMARGUMENT if it shares the kernel.
