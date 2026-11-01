# 0088 — Visible-parity ratchet on in-scope visibly-off cells

- **Status:** accepted (2026-10-20, T-0061). *Amended by decision 0219: the lock is per case, in
  `eval/baseline/visible-parity.txt`.*
- **Context:**
  - Decision 0083 makes visible parity the first target: the 15 significant digits Excel
    shows, and the right type, error and text. The eval gate (`eval/baseline/`) only tracks
    pass/fail per cell. A cell that is already failing can drift from 1 ulp to visibly off, and
    the gate does not notice.
  - The closeness buckets of T-2036 (`close` within 1e-9, `off` for the rest) could not measure
    this. `close` passes numbers right to only 10 digits, and `off` mixes wrong numbers with
    wrong types.
  - T-0061 asked for a ratchet that holds the count of visibly-off in-scope cells, suggesting a
    protected `quality/visible-parity.json`. Unlike the evidence floor, this count moves in
    almost every function PR (fixes lower it) and in probe PRs (new cases can raise it). Under
    `quality/`, each of those PRs would need its own decision record (decision 0010), and the one
    number would conflict between parallel branches about as often as they merge. Running the
    vendored suites a third time in `make check` would add about two minutes.
- **Decision:**
  - The buckets are `pass`, `ulp1`, `ulp4`, `digits15`, `visible_numeric` and `visible_other`
    (crates/rustsheet-eval/src/closeness.rs). A number is within 15 digits when |actual − expected|
    ≤ half a unit in the 15th significant digit of the expected value (the boundary counts as
    within). An expected 0 needs an exact 0. `visible_numeric` and `visible_other` are visibly off.
    The ulp buckets come first, as T-0061 asks: a cell within 4 ulp is never visibly off, even
    where 4 ulp exceed half a unit in the 15th digit (an expected value just under a power of
    ten, such as 9.5, or a subnormal one). Those few cells count toward ulp parity, not against
    visible parity.
  - Per function, report.json also carries `visible`: the visibly-off cells blamed on that
    function. A cell is blamed on the functions it calls other than glue (`GLUE`: IF, IFS,
    IFERROR, IFNA, SWITCH, CHOOSE, LET, AND, OR, NOT, TRUE, FALSE, SUM, DATE, TIME, INDEX, ROW,
    COLUMN). If it calls only glue, the glue functions take the blame.
  - The ratchet counts the visibly-off cells of the vendored gated suites. A cell counts when it
    calls no out-of-scope catalog function. Fetched suites (poi, corpus-bench) are left out, so
    the count is the same on every machine and in CI.
  - The lock is `eval/baseline/visible-parity.json`, next to the baselines. `rustsheet-eval check`
    (`make eval-check`, already in `make check` and CI) fails a full gated run when the count
    differs from the lock: a rise is a REGRESSION, and a drop means the lock is out of date.
    `update-baseline` (`make eval-baseline`) records a drop, and a rise up to the visibly-off cells
    of cases the committed baseline does not know yet (new coverage). Any larger rise needs
    `--allow-regressions`, which needs its own decision record (or decision 0083's standing
    allowance, whose condition 2 already forbids new visibly-off cells). Like the baselines, the
    file is never hand-edited.
- **Bootstrap:** the first lock was taken on T-0061's branch. Probe PRs that merged before it
  added visibly-off cells through their own baselines, which the lock could not see. For those
  cells, T-0061 records the merged count once with `update-baseline --allow-regressions`, and only
  when every baseline reports 0 regressions. The lock went from 1,784 to 1,825, all of it
  T-2115's SHEETS and INFO cases. After T-0061 merges, a PR that adds cases records them with
  its own baseline update, so this does not recur.
- **Consequences:**
  - A PR that fixes visibly-off cells commits the lower lock with its baseline update, so the
    lock carries no slack.
  - A probe PR that adds visibly-off cells records them as new coverage, as it already does for
    new failures.
  - A change that makes an existing failing cell visibly off fails CI, even though the baseline
    already lists the cell, unless the same PR fixes at least as many visibly-off cells. The
    count cannot tell those apart, and the Excel-fidelity reviewer reads the eval report for it.
  - The parity page (decision 0079) shows the new buckets, a "Visibly off" column per function
    (blamed cells) and per suite. Its landing summary states how many in-scope functions are
    visibly off.
  - No protected path changes. `quality/` keeps the hygiene ratchets.
