# 0091 — TEXTSPLIT padded-size probe cells are accepted directive-5 deviations

- **Status:** accepted (2026-10-21, T-2223). It adds four lines to eval/suites/probes/deviations under decision 0036.
  No gate changes and no baseline line is hand-edited.
- **Context:**
  - T-1781 bounded TEXTSPLIT's padded result (rows x widest row) with `limits::check_array_cells`, the 2^20-cell
    array bound of the arrays.md entry "Array results above 1,048,576 cells are #ERROR!" (Status: DEVIATION,
    decision 0014). That bound exists to cap allocation on untrusted input (CLAUDE.md directive 5). A 32,767-char
    text can otherwise pad about 2^28 cells.
  - probes/textsplit-padded-size A1–A4 (excel-graph) measure Excel past that bound. `ROWS` and `COLUMNS` of
    TEXTSPLIT over 1500 `,` then 1500 `;` are 1501 (A1, A2). The 1025 x 1025 text, just above 2^20 cells, gives
    1025 (A3, A4). The engine gives `#ERROR!` for all four, by design. A5 (exactly 2^20 cells) and A6 pass.
  - The four cells were left as F lines in eval/baseline/probes.txt. So M7's probe-milestone row counts them
    as known failures, although they are the documented deviation.
  - probes/lifted-range-rows C1–C3 and E1, the same bound reached through a lifted range, are already listed in
    eval/suites/probes/deviations.
- **Decision:** List probes/textsplit-padded-size `Calc!A1`..`Calc!A4` in eval/suites/probes/deviations. Each line
  cites decision 0014 and the arrays.md entry, with `#ERROR!` pinned. They score as deviations. If the engine ever
  gives another value that is not Excel's, the cell fails again (decision 0036).
- **Consequences:**
  - `make eval-baseline` drops their four F lines from eval/baseline/probes.txt. The M7 probe-milestone known
    failures fall by 4.
  - Raising the bound, or answering Excel's value within it, is out of scope. Doing so would make the cells match
    Excel, and their lines would then go.
