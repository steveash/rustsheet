# 0114 — Phase 1 exits on fifteen-digit parity; semantics, then accuracy, then algorithm

- **Status:** accepted (2026-10-25, owner decision in session
  https://claude.ai/code/session_015FFUAimJC9YQNhjLfmSH6Y). Amends docs/milestones.md (the Phase 1
  exit and the milestone exit measures), decision 0083 (pick order inside the visible campaign) and
  decision 0094 (adds a rate measure next to its zero-visibly-off checkpoint). Keeps decision 0052:
  accepted deviations stay directive-5 limits only, and prime directive 1 is unchanged.
- **Context:**
  - The owner's priority, restated: **≥ 99.9% of each function's Excel cells right to the 15
    significant digits Excel shows** matters more than bit parity. Decision 0083 already ordered the
    work that way, but the Phase 1 exit still required ≥ 99.9% bit-exact and 100% within 4 ulp per
    function, and the milestone exit measures (decision 0085) count bit failures. The planner,
    gap-triager and steward read those targets, so last-digit fit tasks kept being filed and picked
    (T-2146, T-2122, T-2144, T-2384).
  - Owner assessment of the 2026-10-25 eval (`main` 2c52346, all suites fetched; method in the
    assessment page, session above). Attributing each visibly-off cell to the one non-glue function
    of its formula:
    - 310 of 506 in-scope functions are at ≥ 99.9% within 15 digits; 127 are below 99%.
    - The 127 have 7,716 visibly-off cells: 1,688 wrong type, error or text (**semantics**), 5,565
      numbers within 1e-9 relative, mostly in the 14th–15th digit (**accuracy**), and 463 further
      off (**algorithm**, plus a few semantic bugs such as table `[#Totals]` spans).
    - Fixing only the semantic cells lifts 45 of the 127 to ≥ 99.9%; the accuracy cells lift 35
      more; the last 47 need Excel's algorithm.
    - A 50-digit mpmath reference on 873 failing literal-argument cells splits the accuracy cells in
      two. For GAMMA.DIST, CHISQ.*, POISSON, F.DIST and BETA.DIST, Excel is accurate (median
      relative error 2e-16 to 1e-15) and the engine is not (2e-15 to 5e-14): a more accurate kernel
      fixes them with no fitting. For BINOM.DIST and T.DIST, Excel is the inaccurate one (3e-15 to
      4e-15) and the engine is already right to 5e-17: matching Excel's 15 digits there means
      reproducing Excel's algorithm, which is the same work as bit parity (T-0969's time-box).
    - Precision work was about 9% of merges since 2026-10-14; about half of it was last-bit fitting
      of cells already within 15 digits.
  - The owner chose to keep matching Excel where Excel is the noisy one (prime directive 1): those
    functions are worked last, under decision 0056's time-box, and are never deviations.
- **Decision:**
  1. **Phase 1 exits on fifteen-digit parity.** Every in-scope function has ≥ 99.9% of its Excel
     cells within 15 significant digits with the right type, error and text (decision 0083's rule,
     measured by decision 0088's buckets), plus decision 0094's coverage (400 Excel cells, 200 in a
     generated suite). Every other cell has a filed task or a directive-5 `DEVIATION`. Wherever a
     Phase 1 milestone's exit criteria in docs/milestones.md say "bit-exact", Phase 1 reads it as
     "not visibly off". Bit-exact and within-4-ulp parity move to **Phase 1b**, worked after
     Phase 1 exits and before Phase 3 (release); they are reported on every run and never gate
     Phase 1. The CI gate is unchanged: no cell may regress, bit for bit (decision 0083's
     allowance for ulp trade-offs still applies).
  2. **Campaign stages.** `campaign:` takes `semantics`, `accuracy` or `visible`, and
     `scripts/tasks.py next` picks them in that order, after P0 tasks, handoffs and re-reviews and
     before the milestone order:
     - `semantics`: wrong type, error or text, and wrong numbers whose cause is a semantic rule
       (argument checks, error kinds, result-string formatting, reference resolution). Worked first.
     - `accuracy`: visibly-off numbers in a family where a high-precision reference shows Excel is
       close to correctly rounded and the engine is not. The fix is a more accurate kernel, scored
       against the reference and the oracle, never fitted to Excel's last bits.
     - `visible`: every other visibly-off task, including reproducing Excel's own algorithm where
       Excel is the inaccurate one (iterative solvers, inverses, Student-t and binomial tails,
       Bessel). These stay under decision 0056's time-box.
     An open task that a campaign task depends on is picked at the earliest stage among the
     campaign tasks that depend on it (decision 0094's promotion, per stage).
  3. **Parked bit work.** A task whose cells are all already within 15 digits (last-digit fits,
     ulp residue, operation-order cracking) carries `defer: phase-1b`. `next` and `ready` never
     pick it while the key is set; `list` and the board still show it. The steward removes the key
     from every task when it adds the Phase 1 exit line to docs/status.md.
  4. **Measures.** The eval report gains, per function, the share of its Excel cells that are not
     visibly off, and a Phase 1 line counting the in-scope functions at ≥ 99.9%
     (`v15_checkpoint`). The milestone exit rows (decision 0085) count visibly-off cells, not
     bit failures, for the generated suite, ironcalc-calc and probes, and still show the bit-level
     counts beside them for Phase 1b.
- **Consequences:**
  - Workers pick semantics tasks first. The 2026-10-25 assessment filed one per family (CELL,
    bond argument checks, IM* result text, distribution domain errors, FORECAST.ETS argument
    errors, statistical error kinds, FORMULATEXT, table `[#Totals]` spans), then accuracy tasks
    (incomplete gamma, the BETA.DIST cdf switch, F.DIST).
  - FORECAST.ETS values need desktop Excel (T-2425); they are M14 work, and their cells are filed
    tasks under decision 0039 rule 3, so they do not block the Phase 1 exit.
  - Fit-lab work continues only for `visible` tasks whose cells are visibly off, time-boxed.
  - The gap-triager tags groups by stage, and files last-digit groups with `defer: phase-1b`.
