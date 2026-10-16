# 0067 — Allow three poi MODE regressions where Excel 15 conflicts with Microsoft 365

- **Status:** accepted (2026-10-16, T-1291)
- **Context:**
  - T-1291 makes MODE, MODE.SNGL and MODE.MULT skip typed text and booleans and propagate a
    direct error, as current Excel does (probes/mode-typed-arguments, 33 cells, excel-graph).
  - Three cells of `poi/FormulaEvalTestData_Copy.xlsx` (saved by Excel 15, AppVersion below 16)
    pass today only because T-0947 implemented their cached `#VALUE!`: EverythingTests!G936
    `MODE(C9,E16,"1",B8)`, H936 `MODE(B8,"1",2,3)` and K936 `MODE("1",1,#DIV/0!,2,3)` (B8 = 1,
    C9 = 1.1, E16 = text "1.1").
  - probes/mode-typed-text-no-mode (16 cells, excel-graph, 2026-10-16) evaluates the same
    formulas on the same inputs in Microsoft 365: A3 `MODE(K2,K3,"1",K1)` and A4
    `MODE(K1,"1",2,3)` are `#N/A`, A2 `MODE("1",1,#DIV/0!,2,3)` is `#DIV/0!`. Typed text without
    a repeated number is `#N/A` in every shape probed (A1, A5, A7, A8, A10, A13, A16).
  - Decision 0030 rule 2: an older desktop build (below 16.x) does not outrank current web
    behavior; decision 0039: in Phases 1-3 the web value is the target. The same situation is
    already recorded for J136 `AND("1",1)` (coercion.md "AND/OR/XOR text arguments").
  - The poi suite is fetched, not vendored, so `make check` and CI do not run it; the
    regression shows only in a local `make eval-baseline` after `rustsheet-eval fetch --suite poi`.
- **Options considered:**
  - *Keep T-0947's `#VALUE!`.* It contradicts 7 M365 probe cells and leaves T-1291 undone.
  - *A `cached_overrides` line per cell (decision 0032).* That would score the cells against
    the web value, but 0032 limits overrides to non-en-US evidence, and an Excel-version
    conflict stays CONFLICTING until a 16.x desktop value decides (0030 rule 2).
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-1291 PR,
  for exactly these three poi cells. They are CONFLICTING (Excel 15 cached vs Microsoft 365 web)
  and stay known failures until 16.x desktop evidence decides (T-1075, M14).
- **Consequences:** `eval/baseline/poi.txt` gains EverythingTests!G936, H936 and K936 and no
  other F line (it also loses `poi/63934.xlsx` Sheet1!B2, a structured-reference VLOOKUP that main
  had already fixed; the fetched suite's baseline was stale); reviewers confirm the baseline diff. coercion.md's MODE entry records the
  conflict. M936 `MODE(B9,2,D7)` (blank single-cell reference) is unaffected: M365 agrees
  (probes/mode-typed-arguments A23).
