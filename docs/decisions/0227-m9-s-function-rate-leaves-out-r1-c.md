# 0227 — M9's function rate leaves out R1(c) Phase 1b cells

- **Status:** accepted (2026-11-01, T-3645; owner ruling in session
  https://claude.ai/code/session_01XrFnZsvUtb1zjvKKsLYAQF: "yea lets go ahead and do this; i want to get to a
  phase 1 exit as fast as possible without any embarrassing gaps", on the proposal to make M9's rate stop counting
  Phase 1b cells).
- **Amends:**
  - decision 0118 item 3 and decision 0146 item 3: the every-suite Phase 1 rate, which M9's exit row reads,
    also leaves out decision 0188 R1(c)'s Phase 1b cells, as it leaves out listed cells (decision 0134) and
    M14-owned cells (decision 0143);
  - decision 0211, the rate bullet: `exit_owners.py` lists the attributed cells of the functions below the rate
    less the R3 passes and the Phase 1b cells.
- **Context:**
  - A Phase 1b cell is a visibly-off number where both the engine and Excel are within 1e-9 relative of the
    50-digit math (decision 0188 R1(c)). Decision 0188 R5 makes last-digit work on such cells Phase 1b, after
    Phase 1 exits, and R1(c) already leaves them out of the milestone probe rows and the ironcalc-calc rows.
  - M9's exit row also requires every M9 function at the every-suite rate (decision 0146 item 3), which counted
    every attributed visibly-off cell, Phase 1b or not. So the same cells blocked Phase 1 through the rate while
    their tasks were deferred until after it. On 2026-11-01 (main 4e2f5f3df with T-3587's math sidecars), 211
    of M9's exit cells had no M9 owner. All were F.DIST cells of probes/t3354-f-dist-tiny-x-huge-d2,
    probes-numeric/t3354-f-dist-sweep and t2220-f-dist-sweep-d2-1, owned only by `defer: phase-1b` tasks
    (T-3479, T-2790).
  - Gate B (decision 0188 R1(b)) is the user-facing parity gate: at least 99.9% of each function's attributed
    cells within 15 digits of Excel on gen-*, ironcalc-calc, corpus-bench, calipers, poi, smoke and fixtures. It
    is the Phase 1 rate, and it stays the 15-digit share whatever the math says.
- **Decision:**
  - The every-suite rate of `v15_checkpoint` leaves out each visibly-off, non-deviation Excel cell that R1(c)
    classes as Phase 1b and that neither list above holds, from both `rate_cells` and `rate_visible`
    (`LeftOutKind::Phase1b`, `rate_phase1b`, `phase1b_left_out`). The report says how many, per function.
  - Gate B does not change: it still counts every such cell of its suites.
  - `exit_owners.py` leaves the same cells out of a rate criterion (`counted_in_rate`). It already left out R3
    passes (decision 0211), which the every-suite rate still counts; only Gate B reads them as passing. That
    difference predates this decision and is left as it is.
- **Consequences:**
  - No user-representative cell stops counting: Gate B, the `function rate` line of `make phase1-exit`, is
    untouched. Within 1e-9 can still show in a 10th or 11th displayed digit, so Gate B is what keeps that gap
    out of users' files. On the 2026-11-01 report all 32,473 Phase 1b cells were in probes (233) and
    probes-numeric (32,240), and none was in a Gate B suite. What leaves is the every-suite share's count of
    cells that already sit within 1e-9 of the true value on both sides. They stay known failures, stay in
    `visible`, and are worked in Phase 1b.
  - On the 2026-11-01 report this lifts T.DIST to the rate (571 of its 572 counted off cells are Phase 1b) and
    takes 31,814 Phase 1b cells out of F.DIST's count. It leaves 65 M9 functions below the rate: their off cells
    are outside Gate A's envelope or have no math value yet. A function gains Phase 1b cells only with a Gate A
    definition and math sidecars (SPEC-0029, T-3587).
  - The `at_rate` count ("every suite, information") rises with it. It stays information; Gate B gates.
  - `make exit-owners MILESTONE=M9` no longer lists Phase 1b rate cells. With T-3636 and T-3637 (#2695) it
    reported 0 unowned M9 exit cells on that report.
