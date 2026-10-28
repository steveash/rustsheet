# 0155 — Score COMPLEXs.xlsx N6 IMTAN of a real against the web

- **Status:** accepted (2026-10-28, T-3054, closing T-2774). It adds a fourth class of `cached_overrides` line
  under decision 0032 ("or for a later decision that says so"), after decisions 0090 and 0133. No baseline line is
  hand-edited and no gate is weakened.
- **Context:**
  - IMTAN of a real number is TAN's own x87 kernel on Excel for the web, not the double-angle form
    `sin 2x / (cos 2x + 1)` the engine used. probes/t3054-imtan-real-tan-kernel (excel-graph, 2026-10-28) holds
    ten real arguments where the two print different text: all ten are TAN's kernel (`IMTAN("3.14159")`
    -2.65358979336212E-06, `IMTAN("-17.2737")` -197.642601323436; the double-angle form ...207E-06, ...305), as are
    the 36 `IMTAN("5")` cells of gen-m9 and probes/t2534-im-trig-real-limit D9, D10 (T-2534, T-2774).
  - ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!N6, `IMREAL(IMTAN(D6))` with D6 `COMPLEX(5,0,"i")`, caches Mac
    Excel 16.0300's -3.38051500624658, the double-angle digit, and passes today. The probe's row 1 is the same
    formula on the same inputs (A1:D1 = 5, 0, "i", `COMPLEX(A1,B1,C1)`; N1 `IMREAL(IMTAN(D1))`), and Excel for the
    web gives -3.38051500624659 there.
  - Decision 0039 rule 4 follows the web for web-vs-desktop conflicts in Phases 1–3 but says moving a cell that
    passes on desktop evidence "needs its own decision". T-2534 and T-2774 stopped there: recording N6 as a known
    failure needs `--allow-regressions`.
- **Options considered:**
  - *(a) Keep the double-angle form for reals.* Keeps one desktop cell and misses 47 web cells (gen-m9, probes).
    Rejected: decision 0039 makes excel-graph the oracle of record.
  - *(b) Record N6 as a known failure with `--allow-regressions`.* Leaves a cell that matches the oracle of
    record listed as failing for as long as Phase 4 is not started. Rejected.
  - *(c) A `cached_overrides` line for N6, pointing at the probe's N1.* The existing mechanism; the harness
    refuses a hand-written value and takes only a golden sidecar. Chosen.
- **Decision:** A `cached_overrides` line may replace an Excel-desktop cached value that contradicts Excel for the
  web on the same formula and inputs, when decision 0039 rule 4 settles the conflict for the web and an
  excel-graph probe cell of that formula on those inputs exists. Each such line cites this record and its probe
  in the overrides file; the desktop value is kept in docs/excel-semantics and as an item of T-1075, which settles
  it once the desktop oracle exists (Phase 4).
- **Consequences:**
  - eval/suites/ironcalc-calc/cached-overrides.tsv gains the N6 line; N6 passes on the web value, and the engine's
    IMTAN of a real is TAN's kernel (`excel_tan`).
  - When Phase 4 starts, T-1075 item 17 decides whether N6's desktop value comes back (a desktop-only rule would
    then need a platform switch, which RustSheet does not have).
  - The rule is narrow: it applies only to a desktop cached value that a same-formula, same-input excel-graph
    probe contradicts; any other override keeps the classes of decisions 0032, 0090 and 0133.
