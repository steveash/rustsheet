# 0087 — Named milestone exit exceptions live in eval/milestones/exit-exceptions.tsv

- **Status:** accepted (2026-10-20, T-2176). It records exceptions that decisions 0074 and 0085
  already made and weakens no gate: no baseline, ratchet or CI check changes.
- **Context:**
  - The ironcalc-calc exit measures of M2 and M5 (docs/milestones.md; decision 0085 item 1) leave
    out cells that a decision names: `LOGICAL/IF_ARRAY.xlsx` Sheet1!C11 from M2 (decision 0074),
    Sheet1!C4 and E4 from M5 (decision 0085). Decision 0085 item 3 lets the PR that blocks an M5 fit
    task under decision 0056's time-box name more cells within 4 ulp.
  - Those cells were named only in the prose of docs/milestones.md, so the "Milestone function
    scope" rows of report.md showed `failing` with them included and the steward subtracted them
    by hand.
- **Decision:**
  1. The cells a decision leaves out of a milestone's function-scope exit measure are lines of
     `eval/milestones/exit-exceptions.tsv`: `case!Sheet!A1`, the first milestone whose row leaves
     the cell out (every later cumulative row does too), the decision that names it, and the task
     that owns the cell. It holds only cells a decision names. A line is added in the PR that takes
     that decision, and removed when the cell passes (report.md lists such a line as stale).
  2. The file hides no cell from the baseline or the gate: every listed cell stays an F line of
     eval/baseline/ironcalc-calc.txt, `eval-check` is unchanged, and each row's `failing` keeps
     its meaning. The rows only gain `excepted` and the column "failing excl. exceptions", which
     is the exit measure.
  3. It covers the ironcalc-calc function-scope rows only; decision 0085 gives the probe, gen-m5
     and coverage measures no exceptions.
- **Consequences:**
  - `rustsheet-eval run` and `check` refuse a malformed line, a milestone that is not `M<digits>`,
    a decision that is not four digits, a task that is not `T-<digits>` and a repeated cell.
  - eval/milestones/ stays protected (decisions 0043, 0044), so every new exception needs a
    decision record, as item 1 requires anyway.
  - T-2177's milestone exit section reads "failing excl. exceptions" for the ironcalc-calc measure.
