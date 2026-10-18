# 0074 — Desktop SIN(4) in IF_ARRAY stays a web-vs-desktop conflict

- **Status:** accepted (2026-10-18, T-1876)
- **Context:**
  - `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx` caches SIN(4) as -0.7568024953079283 in three array members:
    `Sheet1!C4`, `E4` and `C11`. The cache comes from Microsoft Macintosh Excel 16.0300. C11 is a
    member of `IF(FALSE,A1:A4,B1:C4)`, so it copies C4, and it counts toward M2's exit (T-1877).
  - Excel for the web gives -0.7568024953079282 (docs/research/oracle-calibration.md), and so does
    the engine. That value is correctly rounded: the true sin(4) is 0.44 ulp from it and 0.06 ulp
    from the rounding midpoint, so the desktop value is 0.56 ulp off.
  - T-1876 ran a fit over 224 web SIN/COS cells (probes-numeric/m9-trig-reduction and
    probes/trig-huge-argument with |x| < 2^27) and the 38 desktop SIN/COS cells of
    ironcalc-calc TRIGONOMETRIC.xlsx `SIN_COS_TAN` (Mac 16.0300). No candidate gives the desktop
    SIN(4) (numbers.md, "Excel for the web and desktop Excel differ in the last digits"):
    - The engine's model (x87 π₆₄ reduction, then a correctly rounded sin or cos of the remainder)
      scores 224/224 web and 38/38 desktop. Rounding the remainder or the result to 64 bits
      first scores the same.
    - A correctly rounded sin, glibc and fdlibm/musl each score 115/224 web and 38/38 desktop.
    - Every other candidate keeps fewer cells.
    - All of them give ...282 for SIN(4), except a double-π reduction, which gives ...284.
  - All 38 desktop TRIGONOMETRIC cells are correctly rounded (at most 0.43 ulp off). So the Mac
    SIN kernel is faithful but not correctly rounded, which fits the platform libm. No formulation
    can be fitted from one cell.
- **Decision:**
  - The engine keeps the web value for SIN(4).
  - The three IF_ARRAY cells are named in numbers.md as an Excel-vs-Excel last-digit conflict.
    They stay in eval/baseline/ironcalc-calc.txt as known failures. Decision 0030 rule 4 asks for
    a fit to the desktop value, but no candidate reaches it without losing web cells, and decision
    0039 rule 4 matches the web for now.
  - Under decision 0039 rule 3, their only open question is desktop evidence. It is an item of the
    M14 task T-1075 (does Windows desktop Excel agree with the Mac cache?). That task counts as
    "a filed task" for M2's ironcalc-calc criterion, so C11 does not block the M2 exit.
- **Consequences:**
  - docs/milestones.md notes the exception under M2's ironcalc-calc criterion.
  - No baseline edit and no `--allow-regressions`: the three cells keep failing visibly.
  - When Phase 4 starts and Windows desktop SIN(4) can be probed, T-1075 settles the target. If
    Windows desktop also gives ...283, and a larger desktop sweep shows a kernel, that kernel gets
    its own functions-math task.
