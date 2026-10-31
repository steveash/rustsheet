# 0115 — Kernel port tasks may lose listed probes-numeric bit-exact cells for a net gain

- **Status:** accepted (2026-10-25, T-2504; the owner's ruling "net gain, listed losses", given in session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a on 2026-10-25). The per-function thresholds below refine
  that ruling (T-2504 review round 1). The second rule, "per-caller best", is the owner's ruling in the same session
  on 2026-10-25, and refines decision 0114 for the beta family.
- **Context:**
  - SPEC-0019 ports Excel's incomplete beta (TOMS 708 `bratio`) one branch at a time. The port tasks are T-2503,
    T-2505 to T-2511 and T-2585.
  - Most of this family's cells are in probes-numeric, an ungated suite (`gate = false`), so the gated `eval-check`
    and its baselines never see them. T-2119 lost 58 probes-numeric bit-exact cells and made 4 visibly off, and that
    showed up only in a reviewer's manual comparison (decision 0111).
  - Excel's own pieces still miss some cells, so even the best research model loses bit-exact cells where the
    engine's current kernel lands on Excel's last bit by chance. r2-impact's ladder (docs/research/t2464/r2-impact,
    section 4) loses 362 such cells by step S3 and 514 by S7, against about 1,450 visibly-off cells fixed. A rule of
    0 changed cells would stop most of the port.
- **Decision:** a SPEC-0019 port task may lose previously bit-exact probes-numeric cells when all of these hold.
  Conditions 1 to 3 apply per function, meaning the specialised function the eval report blames (one SPEC-0019
  `Caller`). They are counted from the main-vs-HEAD comparison that `rustsheet-eval report-diff` (T-2679) prints.
  1. Its fixed cells are at least 2 × its lost cells. A cell is **fixed** when, comparing HEAD with main, it is
     bit-exact now and was not before, or it was visibly off and is now within 15 digits, or it was failing and moved
     closer to Excel's value. A cell is **lost** when it was bit-exact and is not now, or it was already failing and
     moved further from Excel's value. A cell that is both (bit-exact lost, yet still within 15 digits) counts as
     lost.
  2. Its visibly-off count goes down, net.
  3. A function with no fixed cell has no lost cell.
  4. No cell becomes visibly off (beyond half a unit in the 15th significant digit), and no cell changes type or
     error, unless Excel agrees with the new value. "Excel agrees" means another oracle reading of the same formula
     gives the new value (decision 0112's build-dependent bits, a duplicate cell), or the input is a misparsed xlsx
     number, checked at x ± 1 ulp.
  5. The task Outcome lists every lost cell with its ulp distance. When there are many, they go in a committed file
     that the Outcome cites.

  Gated suites are unchanged: 0 regressions, or decision 0083's standing allowance, or a decision record.

  **Per-caller best (refines decision 0114 for the beta family).** The port brings in Excel's pieces. Each function
  (SPEC-0019 `Caller`) may instead take a correctly rounded or accurate piece for a branch, wherever that scores
  better on the oracle's visible cells and passes conditions 1-5. The Outcome gives both scores. The port tasks keep
  `campaign: accuracy`. On r2-bratio's committed data the correctly rounded value is visibly off on fewer BETA.DIST
  cells (2,449 against 2,615 of 30,257) and on about twice as many cells of the other callers (947 against 455 of
  9,757), so neither choice is right for every caller.
- **Consequences:**
  - Every SPEC-0019 port task runs `rustsheet-eval report-diff` between origin/main's and HEAD's probes-numeric
    reports. The command prints, per function, the fixed, lost, changed-still-failing and newly-visibly-off lists and
    the fixed/lost ratio. Until it lands, T-2503 and T-2507 (in progress) compare by hand with the same buckets.
  - The Excel-fidelity reviewer checks the conditions from that output. A function that misses one is narrowed: its
    caller keeps the branch, or a measured region of it, on its delegate (SPEC-0019 "The migration rule"). The other
    way out is a decision record naming the cells.
  - The listed lost cells are bit-level debt. They go to the task's bit-level follow-up (decision 0083), or to T-2512
    or T-2586.
  - SPEC-0019's port rule 2 states this rule, and the port tasks' criteria cite this record.
