# 0115 — Kernel port tasks may lose listed probes-numeric bit-exact cells for a net gain

- **Status:** accepted (2026-10-25, T-2504; ruled by the owner in session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a, "net gain, listed losses"). It supersedes the stricter
  wording first drafted in SPEC-0019's port rule 2: 0 previously passing probes-numeric cells changed, unless Excel
  agrees.
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
- **Decision:** a port task under SPEC-0019 may lose previously bit-exact probes-numeric cells when all of these hold:
  1. it fixes far more cells than it loses, counted from the main-vs-HEAD comparison of probes-numeric;
  2. no cell becomes visibly off (beyond half a unit in the 15th significant digit), and no cell changes type or
     error, unless Excel agrees with the new value. "Excel agrees" means another oracle reading of the same formula
     gives the new value (decision 0112's build-dependent bits, a duplicate cell), or the input is a misparsed xlsx
     number, checked at x ± 1 ulp;
  3. the task Outcome lists every lost cell with its ulp distance. When there are many, they go in a committed file
     that the Outcome cites.

  Gated suites are unchanged: 0 regressions, or decision 0083's standing allowance, or a decision record.
- **Consequences:**
  - Every SPEC-0019 port task runs `rustsheet-eval diff` (added by T-2505) between origin/main's and HEAD's
    probes-numeric reports. It produces the fixed, lost, changed-still-failing and newly-visibly-off lists
    mechanically.
  - The Excel-fidelity reviewer checks conditions 1-3 from that output. A cell that becomes visibly off, or changes
    type or error, without Excel's agreement needs a decision record naming it, or the task keeps that region on its
    delegate (SPEC-0019 "The migration rule").
  - The listed lost cells are bit-level debt. They go to the task's bit-level follow-up (decision 0083) or to T-2512 or
    T-2586.
  - SPEC-0019 port rule 2 and the port-rule criterion of T-2505, T-2506, T-2508 to T-2511 and T-2585 state this
    rule. T-2503 and T-2507 are bound through their `spec:` line.
