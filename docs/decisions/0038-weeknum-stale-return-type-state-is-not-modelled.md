# 0038 — WEEKNUM stale return_type state is not modelled

- **Status:** superseded by decision 0052 (2026-10-13). The stale state is a known gap, not a
  deviation; T-1484 specifies reproducing it. Accepted 2026-10-06, T-1025.
- **Context:**
  - In Excel, `WEEKNUM(serial, return_type)` with a return_type of 22 or more, or negative, is not
    an error: it reuses the start weekday of the last WEEKNUM with a valid type evaluated earlier
    in the same recalc, and is `#NUM!` only when none has run (docs/excel-semantics/dates.md,
    "WEEKNUM return_type edge cases"; probes/weeknum-stale-return-type, probes/weeknum-invalid-type
    and the 94-cell first version of probes/weeknum-serial-0, oracle excel-graph, T-1023).
  - The result is therefore a function of Excel's calc order, not of the workbook's formulas and
    values. On a fresh load the probes fit "bottom to top within a sheet": Up!A1, Mixed!A3/A4,
    Type21!A1 and Type1!A1 take the state of the valid WEEKNUM below them, while Down!A2 is
    `#NUM!` with a valid WEEKNUM above it. Down!A2 is also `#NUM!` though three other sheets hold
    valid WEEKNUMs, so either the state resets per sheet or Down was evaluated first; the evidence
    cannot tell. After edits Excel's order follows its calc chain (stored in `calcChain.xml` and
    reshaped by edit history), so one workbook can give different values in different sessions.
  - RustSheet evaluates spill anchors first and then every cell in storage order (sheet, then row
    ascending; `Model::run_pass`), and a pass can be abandoned and restarted when a spill
    reorders anchors, so a cell may be evaluated more than once per recalc. Its order is neither
    Excel's nor stable across restarts.
  - This is not a directive 5 (untrusted input) deviation, so decision 0014 does not cover it.
    The exception rests on Excel's value not being a function of the workbook at all, so no
    deterministic engine can match it from the file alone (the route of
    docs/excel-semantics/README.md "Deliberate deviations" and decision 0033).
- **Options considered:**
  - *(a) Model the state in RustSheet's own order:* a per-recalc "last valid start weekday" in
    the evaluation state, set by valid WEEKNUMs and read by invalid ones. In storage order it
    gets Up!A1 and Type1!A1 wrong (no state yet, or the wrong one), turns the passing Down!A2
    into 16, and gets Mixed!A3/A4 and Type21!A1 right only because an earlier sheet happens to
    leave "start Monday". The value would also be a hidden dependency the dependency tracking
    cannot see: an incremental recalc, a restart, or a new formula on another sheet would change
    a cell none of its inputs touch. Rejected: it trades known failures for accidental passes and
    nondeterminism.
  - *(b) Model it in a reconstructed Excel order* (bottom to top within a sheet, a guessed sheet
    order, honouring `calcChain.xml`). Needs Excel's full calc order, which is out of scope for
    T-1025 and for Phase 1 (decision 0027), and still could not follow edit history. Rejected.
  - *(c) Keep `#NUM!` for every out-of-range return_type and document it as a deviation.* This is
    Excel's value whenever no valid WEEKNUM ran earlier in the recalc (all 8 cells of
    probes/weeknum-invalid-type, Down!A2), and the result stays a pure function of the formula.
    Chosen.
- **Decision:** `WEEKNUM` with a return_type of 22 or more, or negative, is always `#NUM!`. The
  stale-state behavior is documented as `Status: DEVIATION (decision 0038)` in
  docs/excel-semantics/dates.md, and the probe cells where Excel reused a state stay known
  failures.
- **Consequences:**
  - `fn_weeknum` keeps its current behavior; its comment cites this decision instead of a TODO.
  - probes/weeknum-stale-return-type Up!A1, Mixed!A3, Mixed!A4, Type21!A1 and Type1!A1 stay F
    lines in eval/baseline/probes.txt, as decision 0033's cells do. They do not go into a
    `deviations` file: decision 0036 limits that file to directive 5 deviations, and moving them
    would need a decision that amends 0036.
  - Revisit if RustSheet ever reproduces Excel's calc order (calc chain import and ordering), or
    if other calc-order-dependent functions turn up that the same mechanism would serve.
