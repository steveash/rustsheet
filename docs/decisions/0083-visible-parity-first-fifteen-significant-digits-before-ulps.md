# 0083 — Visible parity first: fifteen significant digits before ulps and bits

- **Status:** accepted (2026-10-20, T-0060; asked for by the owner in session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a)
- **Context:**
  - The owner wants every in-scope function to reach three levels of parity, in this order:
    first visible parity (the 15 significant digits Excel shows are the same, and the type,
    error and text are right), then within a few ulp, then bit-exact. Agents had been spending
    their time on last-ulp fits and edge-case probes while ordinary values were still visibly
    wrong.
  - The eval run of 2026-10-20 (`main` at 35a5e71) was measured with this rule: a numeric cell
    is visibly off when |actual − expected| is more than half a unit in the 15th significant
    digit of the expected value. Each failing cell was blamed on the specialised function in its
    formula, not on glue such as IF, SUM or DATE.
    - 110 of the 505 scored in-scope functions have a visibly-off number (4,613 cells). 131 have
      a wrong type, error or text (1,459 cells). Together that is 186 functions (37%).
    - Of the 95 functions whose own kernel gives the visibly-off numbers, 90 are M9.
    - The text problems are mostly TEXT and displayed-text formats, which are M10. The wrong
      numbers in gen-m8a and gen-m8b come from defined names and table totals rows, which are M8.
    - Examples: `GAMMA(10)` is 362880.0000000024 (Excel 362880). `TEXT(36.5,"0")` is "36"
      (Excel "37"). `PMT(0.05,1,10000,-500,1)` differs in the 15th digit.
  - `scripts/tasks.py next` orders ready tasks by milestone before priority (decisions 0023
    and 0026), so all of that work waits behind every M5–M8 task.
  - On the literal-argument cells, a 50-digit mpmath reference splits the numeric work in two:
    - For GAMMA, the incomplete gamma family, PMT, T.DIST, BETA.DIST, POISSON and ACOTH, Excel is
      close to correctly rounded (median relative error about 2e-16) and the engine is not
      (median 1e-15 to 8e-13). This needs accurate kernels, not fitting.
    - For BESSELJ/BESSELY at x ≥ 8, NORM.S.INV in the centre and the iterative solvers, Excel
      itself is inaccurate. Its algorithm has to be reproduced.
  - A further 612 cells are within half a unit but still display differently, because they sit
    on a rounding boundary of the 15th digit. Only near-bit parity removes those. Excel for the
    web and desktop Excel already differ by up to 4 ulp on some of these functions
    (docs/research/oracle-calibration.md).
- **Decision:**
  - *Order of work.* Visible parity comes first, then ≤ 4 ulp, then bit-exact.
  - *Campaign key.* A task whose target is visible parity carries `campaign: visible` in its
    front matter. `scripts/tasks.py` accepts only the campaign names it lists. `next` orders
    campaign tasks after P0 tasks and handoffs but before the milestone order of decision 0026;
    tasks outside the campaign keep that order.
    - The key goes on numeric-precision tasks for functions with visibly-off cells.
    - It also goes on wrong-type, error and text tasks for everyday features (TEXT and
      displayed-text formats, defined names, table totals).
    - It does not go on tasks for exotic locales or calendars, or on probes.
  - *Done at visible parity.* A campaign task's acceptance target is that every cell it owns is
    within half a unit of the 15th significant digit of Excel's value (an expected 0 must be
    exactly 0), or has the right type, error and text.
    - Bit-level criteria that are not met when that target is reached move to a follow-up task.
      The worker files it, without the campaign key, and names it in the Outcome. The campaign
      task is then done.
    - The fifteen-digit bucket of T-0061 measures this once it lands. Until then the worker
      shows it per cell from `make eval-show`.
  - *Standing ulp-regression allowance.* A campaign PR may run
    `rustsheet-eval update-baseline --allow-regressions` without a decision record of its own
    (precedents: decisions 0064, 0065, 0067, 0069) when all of the following hold:
    1. every newly failing cell is numeric and stays within 4 ulp of Excel;
    2. no cell becomes visibly off, and no non-numeric cell regresses;
    3. the PR fixes more visibly-off cells than it regresses;
    4. the task's Outcome lists every regressed cell, and a follow-up ulp task names them.

    Reviewers check all four from the eval report and the baseline diff. Any other regression
    still needs its own decision record.
  - *Triage.* The gap-triager ranks visibly-off groups (numeric or text) before the
    milestone order, and gives the task it files for such a group `campaign: visible`.
- **Consequences:**
  - The campaign tasks at the start are:
    - harness: T-0061;
    - kernels Excel computes accurately: T-2128, T-2129, T-2130, T-0969, T-0967, T-2119, T-0971,
      T-0972, T-0973, T-0974, T-0977, T-1890, T-2034 and T-2032;
    - Excel algorithms to reproduce: T-2131, T-2005, T-2007, T-2045, T-1592, T-0908 and T-0925;
    - names and tables: T-1639, T-1640, T-1671 and T-1699;
    - TEXT and displayed text: T-1711, T-1730, T-1731, T-1732, T-1733, T-1734, T-1735, T-1737,
      T-1738, T-1740, T-0934, T-0936, T-0937, T-0954, T-0980, T-0981, T-0982, T-1366, T-2072,
      T-2073, T-2074 and T-1883.

    Each of them carries a "Visible-parity stage" section stating its stage-1 target.
  - Milestone exit criteria and the `**Critical lane:**` line are unchanged. Workers reach
    campaign tasks through `make next-task LANE=functions` (and the other lanes) as before. The
    campaign only reorders what they pick inside a lane.
  - The campaign ends when T-0061's report shows no visibly-off cell in an in-scope function, or
    when the owner ends it. A later decision then removes the key and the allowance.
