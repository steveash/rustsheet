# 0192 — RATE's flat-secant hold loses 13 Gate A points and T-3348 is re-scoped

- **Status:** accepted (2026-10-31, T-3348). Owner ruling (2026-10-31, in chat with the repository owner): accept the
  trade (+164 cells of Excel's type, -13 Gate A points) and merge, with the 13 points owned by P0 task T-3424. Takes
  effect when the T-3348 reviewers approve its PR.
- **Context:**
  - T-3348 changes `compute_rate` (financial_util.rs) in two ways. A flat secant (f1 = f0) holds at r1 and returns
    it when it passes the convergence test or the cap's test. An iterate at or below -1 runs on (financial.md
    "RATE holds on a flat secant and runs on below -1"). Reviewer check 7 classes this as a structural model,
    kind (c). The hold-out and fresh-sweep results are in T-3348's Outcome.
  - Gains, measured as cells of Excel's type, old model -> new:
    - probes/t3348-rate-fv-nonzero: 126 -> 186 of 186 (+60).
    - probes/t3348-rate-below-minus-one: 20 -> 50 of 50 (+30).
    - probes-numeric/t3348-rate-gate-a-sweep, the training sweep: 274 -> 298 of 300 (+25, -1).
    - probes-numeric/t3348-rate-gate-a-fresh, the sweep drawn after the freeze: 249 -> 286 of 300 (+49, -12).
    - The 960-cell fit set of "RATE is a secant on the FV residual" stays 960/960 bit-exact.
    - No gated cell regresses.
  - Losses: 13 points where the old model's `#NUM!` matched Excel and the hold now returns a number, which moves
    them out of the Gate A envelope (decision 0188 R2):
    - t3348-rate-gate-a-sweep Calc!A296.
    - t3348-rate-gate-a-fresh Calc!A59, A70, A74, A77, A89, A90, A95, A157, A162, A173, A275 and A292.

    All 13 have |pv| and |fv| from 1e6 to 1e12. The held |f| is between 2.4e-7 and 5.5e-5, inside the hold's
    bound, and Excel returns other holds across the same range of |f|. Both probes-numeric cases are ungated.
  - No rule on the hold separates these 13. Across the 1,796 RATE oracle cells, Excel returns 133 holds and errs on
    13 below the 1e-4 bound, and errs on 95 and returns 3 above it (sweep A272, fresh A251 and A272):
    - An absolute bound on |f|: Excel errs at 2.4e-7 and returns up to 9.5e-4.
    - A bound relative to the residual's largest term: errors and returns overlap from 5e-20 to 1.4e-16.
    - A zero step (r1 = r0) against two distinct iterates: Excel errs on 12 of 125 zero-step holds and on 1 of 21
      distinct ones.
    - A cut on the step count: the 13 hold at steps 11 to 119, inside the range of returned holds (up to step 197,
      fresh A88).

    Excel's own iterates differ from the model's in the residual's rounding noise. Sweep A272 returns a value
    about 2 ulps from the model's held iterate. So the 13 are the noise floor's path, not a missing stop rule.
  - T-3348's acceptance criteria asked for the Gate A envelope with 0 points where one side errs and the other is
    a number, and for every probe cell within 15 digits. This change cannot meet either.
- **Options considered:**
  - *Revert the hold.* This loses the 164 cells of Excel's type the change gained, among them the task's own
    example, to save 13.
  - *Return `#NUM!` where the old model matched.* That is a keep-matched branch, which decisions 0056 and 0082
    forbid. No general rule from the list above does it.
  - *Keep the hold and disclose the losses.* This decision.
- **Decision:** keep the flat-secant hold and the run below -1. Accept the 13 lost points as known misses owned by
  T-3424 (P0). T-3348 closes on what it delivers: the `#NUM!` class fixed, the run below -1, and no gated regression.
  Its Gate A criterion moves to T-3424 and its 15-digit criterion to T-3389. RATE's Gate A is not met, and no
  exception is claimed: it stays failed until T-3424 settles it or the owner rules under decision 0188.
- **Consequences:**
  - The baselines change additively only. `eval/baseline/probes.txt` gains the T-3348 probe cases and their 7 known
    failures, and no line is removed. The probes-numeric cases are ungated.
  - T-3424 owns 16 type cells:
    - the 13 above;
    - 3 that neither model gets right: sweep A272, fresh A251 and fresh A272, where Excel returns a number and the
      engine gives `#NUM!`.

    Excel's `#NUM!` on the 13 remains its goal, through a residual that reproduces Excel's noise, not a rule on
    the hold.
  - T-3389, raised to P1, owns the 4 of those 7 that are beyond 15 digits (fv-nonzero A138, A171, A180, A185). They
    are visible misses, not bit residue. The other 3 (A112, A153, A175) are 1 to 2 ulps off, which is Phase 1b work
    (decision 0188 R5).
  - Reviewers confirm that the baseline diff only adds lines.
