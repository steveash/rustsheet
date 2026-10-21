# 0094 — Fifteen-digit parity checkpoint: 400 Excel cells per function with edge sweeps, zero visibly-off cells

- **Status:** accepted (2026-10-21, T-0062; asked for by the owner in session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a). Builds on decision 0083 (visible
  parity first) and decision 0088 (the visible-parity lock).
- **Context:**
  - The owner wants to know when RustSheet can claim Excel parity to 15 significant digits, and
    wants that claim to rest on good coverage of each function's inputs, edge values included.
  - The eval of 2026-10-21 (`main` 7dab5361) shows the count of visibly-off cells is not yet such
    a measure:
    - **Thin coverage.** All 506 in-scope functions are implemented and have Excel cells, but 358
      have fewer than 400 and 96 have fewer than 50 (median 191). 78 of the 358 have no cell in a
      generated suite. M7 and M9 functions have no generator edge pools at all (T-2109 and T-2110
      covered M2–M6 params only).
    - **A partial count.** The decision 0088 lock (1,825 cells) counts the gated suites only. All
      vendored suites together have 5,804 visibly-off cells in 231 functions; most of the
      difference is the ungated probes-numeric suite.
    - **Coverage finds more errors.** BETA.DIST went from 541 to 1,308 visibly-off cells in one day
      once T-0969 added dense Student-t sweeps. A zero for a thinly tested function means little.
  - Phase 1's exit (≥ 99.9% bit-exact, 100% within 4 ulp) is stricter than 15 digits, so this
    checkpoint is reached well before it and needs its own definition.
- **Decision:**
  - *The checkpoint.* RustSheet has **fifteen-digit parity** when all of the following hold:
    1. **Coverage.** Every in-scope catalog function has at least 400 Excel-backed cells (target
       500). At least 200 of them are in a generated suite whose formulas have the function as
       their outer call, drawn with `--focus` and edge pools on its value parameters, across the
       argument modes its catalog signature lists.
    2. **No visible error.** No in-scope function has a visibly-off cell (decision 0083's rule) in
       any vendored suite, gated or not, probes-numeric included, nor in poi or corpus-bench when
       they are fetched. Accepted `DEVIATION`s (directive 5) are excluded.
    3. **Stability.** Both hold on two consecutive steward runs.
  - *Exemptions from the 400-cell rule.* Some functions have no inputs or only a small, closed
    set of them: PI, TRUE, FALSE, NA, the volatile functions (NOW, TODAY, RAND, RANDBETWEEN,
    RANDARRAY), and functions whose every input form is enumerable (for example INFO's type
    texts). They need each input form covered instead, at least 20 Excel cells (the evidence
    floor). The T-0062 report section lists each exemption with its reason. Adding one takes a
    line in the catalog, which is reviewed like any catalog change.
  - *Measurement.* The `make eval` report gains a "Fifteen-digit parity checkpoint" section
    (T-0062). It shows, per function, its Excel cells, generated cells, exemption and visibly-off
    count, and a met or not-met line overall. The steward copies that line into the docs/status.md
    scoreboard on every run.
  - *Sweeps.* Coverage comes from oracle-filled generated suites, one task per milestone or
    category (T-2252 to T-2258). Each sweep runs the gap-triager on what it finds, as T-2114 did,
    and files or extends a `campaign: visible` task for every visibly-off family before it is
    done. The generator first gets `--functions` and `--category` (T-0063) and the missing edge
    pools (T-2247 to T-2250). All of these tasks carry `campaign: visible`.
  - *Dependencies come along.* `scripts/tasks.py next` orders every open task that an open
    campaign task depends on, directly or not, as a campaign task too (`in_campaign`). Without
    that, a probe or sweep a campaign fix waits on (on 2026-10-21: T-2043, T-2081, T-2145,
    T-2187, T-2220 and seven others) waits behind the milestone order and stalls the fix.
    Decision 0083 still keeps the key off probes. The promotion follows the dependency, not a
    tag, so it ends when the dependent task closes.
- **Consequences:**
  - The visibly-off count will rise before it falls, as about 100,000 new Excel cells land. The
    decision 0088 lock counts them as new coverage through `update-baseline`, so the gate does not
    block the sweeps.
  - The checkpoint is a report, not a gate. It never blocks a PR.
  - Phase 1's exit criteria and milestone order are unchanged.
