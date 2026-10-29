# 0146 — Exit measures of M1, M8, M9, M10 and M11

- **Status:** accepted (2026-10-28, T-0088). It defines measures and weakens no gate: no baseline,
  ratchet or CI check changes. T-0089 implements the rows. It answers the questions of T-2889 (M8),
  T-2890 (M9), T-2891 (M10) and T-0487 (M11), which are dropped as superseded.
- **Context:**
  - docs/milestones.md's Phase 1 exit needs "the exit criteria of M1, M2, M5–M10 and M11" met.
    `milestone_exit.rs` has rows for M2 (decision 0135), M5 and M6 (decision 0085) and M7 (decision
    0140). M1 and M8–M11 had none, so `make exit-owners MILESTONE=M9` stops with "report.json has no
    milestone_exit row for M9" and nobody can tell from a run whether those exits are met.
  - Decision 0114 reads every "bit-exact" in a Phase 1 exit as "not visibly off" and moves bit and
    4-ulp parity to Phase 1b. Decision 0135 has `exit_owners.py` list only visibly-off exit cells.
  - Numbers: `make eval` on `main` 899b49fd (2026-10-28, poi, calipers and corpus-bench not fetched),
    and `rustsheet-eval run --suite poi --suite corpus-bench --suite calipers` on 57bf1efe (`main`
    0f43b80f plus the T-0088 claim) after `rustsheet-eval fetch`:
    - **M1:** smoke 46 cells, 0 failing; gen-m1 3,750 cells, 0 failing; the frozen list
      `m1-probes` (decision 0043) 89 cases, 1,858 cells, 0 known failures. The report's
      `milestone_lists` row counts known failures but not visibly-off cells.
    - **M8:** gen-m8a (defined names and 3-D references) 12 cases, 3,577 cells; gen-m8b (tables and
      structured references) 9 cases, 3,368 cells; both 100%, both `gate = false`. No generated suite
      covers external references or sheet add, rename and move; their evidence is in `probes` and
      poi. Probe row M8: 132 cases, 1,262 cells, 9 visibly off. poi: 336 cases (205 pending as
      web-vs-desktop conflicts, decision 0019; 10 crashes, 14 errors), 27,209 cells, 95 visibly off
      (99.65% not visibly off over the whole suite). The report cannot tell which poi cells use a
      defined name, a structured, 3-D or external reference.
    - **M9:** ironcalc-calc 73,155 cells; its cumulative M9 function-scope row holds the same 73,155
      cells (every in-scope function has catalog milestone ≤ M9), 567 failing, 5 excepted, 138
      visibly off (99.81% not visibly off). `v15_checkpoint` (decisions 0094, 0118, 0143): 181 of the
      263 in-scope M9 functions are covered and at ≥ 99.9% of their attributed cells within 15
      digits. Probe row M9: 503 cases, 39,158 cells, 2,033 known failures, 578 visibly off.
    - **M10:** gen-m10 10,400 displayed-text items, 10,324 passing (99.27%), and 1,690 value cells, 0
      failing. probes holds 410 text items, all passing. Probe row M10: 104 cases, 7,406 cells, 50
      visibly off; the probe rows count value cells only. M10 has no in-scope functions (TEXT, FIXED
      and DOLLAR are catalog M5).
    - **M11:** corpus-bench 912 workbooks, 320 scored against excel-graph sidecars (592 crash-only
      until filled), 205,366 formula cells, 205,018 bit-exact, 340 visibly off (99.834% not visibly
      off). Probe row M11: 4 cases, 32 cells, 0 failing. The best published SpreadsheetBench score
      is witan's 99.8% of cells (xlsx-corpus-bench README, docs/research/test-suites-and-oracle.md
      §9), with a 1e-9 relative comparator on Mac desktop Excel truth.
- **Decision:** every new row follows decisions 0085, 0114, 0135 and 0140: it reads only rows the
  report computes, a missing input is `None` (`n/a`) and never met, a measure shown but not required
  has its counts and a null verdict, each row names the probe row it read, and the bit-level counts
  stay beside the visible ones for Phase 1b. M2 keeps decision 0135's measures and M7 decision
  0140's; with this record every Phase 1 milestone (M1, M2, M5–M11) has defined measures.
  Named exceptions (decision 0087) stay function-scope only: `eval/milestones/exit-exceptions.tsv`
  applies to M9's ironcalc-calc measure as its lines already say, and this record adds no line;
  M1, M8, M10 and M11 have no ironcalc-calc criterion, so no exception file applies to them.
  1. **M1.** Required: (a) gen-m1 has at least 3,000 scored cells with 0 visibly off (the exit's
     "≥ 3,000 oracle cells"); (b) smoke has 0 visibly-off cells; (c) the probe measure reads the
     frozen `m1-probes` list (decision 0043), not a probe-milestone row, and needs 0 visibly-off
     cells. The report adds a visibly-off count and the visibly-off cells to each `milestone_lists`
     row so (c) is read at visible parity. Coverage and ironcalc-calc are not M1 criteria (M1 has no
     function scope; both `n/a`). The parity-bug list and the API v0 criterion are not cell measures
     and stay out of the row, as M2's structural criterion does (decision 0135 item 1).
  2. **M8.** Required: (a) **feature suites:** gen-m8a and gen-m8b each have at least 3,000 scored
     cells (the M1 floor; the exit names no count) and 0 visibly off. External references and sheet
     operations have no generated suite; they are measured by the M8 probe row, which decision 0051
     already assigns their probe cases, so no suite is needed for the exit. (b) **POI subset:** the
     scored poi cells whose formula (or array anchor's formula) contains a defined name, a
     structured reference, a 3-D reference or an external-workbook reference, as the engine's parser
     classifies it. At least 99.5% of them are not visibly off; deviations are left out of the
     denominator (decision 0036); pending cases (decision 0019's conflicts) are not scored. A poi
     case that crashes or errors and contains a subset formula fails the measure until it runs, and
     exit-owners lists it as `case!*`. The report gains this subset count; until it exists the
     measure is `n/a`. (c) The M8 probe row has 0 visibly-off cells. Coverage and ironcalc-calc:
     `n/a` (M8 has no functions).
  3. **M9.** Required: (a) **ironcalc-calc overall** is the cumulative M9 function-scope row less its
     exit exceptions (decision 0087; every existing line carries over), with at least 0.9995 of its
     scored cells not visibly off: the allowance is `floor(0.0005 * scored)`, 36 today. (b) **Dense
     sweeps per numeric function** are `v15_checkpoint`'s per-function rows, not a second count
     (directive 3): every in-scope function of catalog milestone M9 is `covered` (decision 0094) and
     `meets_rate` (≥ 99.9% of its attributed cells not visibly off, decisions 0118 and 0143). The
     attributed cells already include gen-m9, probes-numeric and the per-function sweep probes. The
     exit's "100% within 4 ULP" is Phase 1b and is not read. (c) The M9 probe row has 0 visibly-off
     cells. Catalog coverage is shown, not required (item (b) covers it per function); gen-m9 alone
     is not a criterion. Exit-owners lists (a)'s visibly-off cells, the `attributed_off` cells
     (decision 0142) of each M9 function failing (b), and (c)'s cells.
  4. **M10.** Required: (a) **formatted-text suite:** gen-m10's displayed-text items, at least 10,000
     of them, with 0 failing (a text item has no 15-digit reading, so "100%" is exact), and gen-m10's
     value cells with 0 visibly off. probes' text items are not part of the suite. (b) **Probes:** the
     M10 probe row has 0 visibly-off value cells and 0 failing text items; the report adds text-item
     counts and failing text cells to each probe row. Decision 0051's assignment is unchanged: a
     `probes/catalog/*` case of TEXT, FIXED or DOLLAR counts toward M5, its function's milestone.
     Exit-owners lists failing text items as `case!Sheet!A1`, like value cells. Coverage and
     ironcalc-calc: `n/a`.
  5. **M11.** Required: (a) **corpus-bench:** its scored formula cells (Excel-backed sidecar values
     only; deviations left out; crash-only workbooks without a sidecar not scored), at least 200,000
     of them, with at least 0.998 not visibly off. Visible parity (decision 0114) is the comparator;
     it is stricter than the leaderboard's 1e-9 relative tolerance for numbers. "Matching or beating
     the best published engine" is the fixed rate 99.8% (witan, SpreadsheetBench), re-read only when
     the pinned corpus or manifest changes. When corpus-bench was not fetched the row is `n/a`.
     (b) The M11 probe row has 0 visibly-off cells. poi, calipers and the FUSE subset (T-0431) are
     information, not M11 measures (poi's subset is M8's). Coverage and ironcalc-calc: `n/a`.
- **Consequences:**
  - docs/milestones.md's M1 and M8–M11 exit texts cite this decision. The steward and the planner
    run `make exit-owners MILESTONE=M<k>` for every Phase 1 milestone once T-0089 lands.
  - T-0089 and the tasks split from it build the rows and the report measures items 1–5 need:
    visibly-off counts on `milestone_lists` (T-0092), text-item counts on probe rows (T-0093), the
    poi M8 subset (T-0094), more than one suite per row with the M1 and M11 rows (T-0089), and the
    M9, M8 and M10 rows (T-0095, T-0096, T-0097), each with unit tests for a met and an unmet row.
  - On today's numbers M1 is met; M8 is not (9 probe cells; the POI subset is `n/a`); M9 is not
    (138 > 36, 82 functions, 578 probe cells); M10 is not (76 text items, 50 probe cells); M11 is
    met on corpus-bench (99.834%, 205,366 cells) and its probes.
  - T-0089 (2026-10-29) builds the M1 and M11 rows. Each row reads a list of required suites, each
    with its own floor and rate. The first suite is the row's `gen_*` measure, named in
    `gen_suite` (`gen-m1`, or `corpus-bench` for M11). The rest go in `more_suites` (M1's smoke).
    M1's `probe_row` names the `m1-probes` list it reads. `scripts/exit_owners.py` lists the
    visibly-off cells of every required suite (by `gen_suite`, falling back to `gen-m<k>`, then
    `more_suites`) and of a `milestone_lists` row whose name is the exit row's `probe_row`. The
    M2, M5, M6 and M7 rows read and print as before.
  - T-0095 (2026-10-29) builds the M9 row of item 3. It requires no suite (`gen_suite` is empty).
    (a) reads the cumulative M9 function-scope row: scored cells are its cells less deviations and
    exit exceptions, met at `meets_rate` with 9,995 over a denominator of 10,000 (every rate in
    `milestone_exit` moved from over 1,000 to over 10,000, same shares). (b) reads `v15_checkpoint`'s
    rows, which now carry each function's catalog `milestone`, and counts the M9 ones not
    `meets_rate`. `scripts/exit_owners.py` lists, beside the function-scope and probe cells, the
    `attributed_off` cells of each function the row names in `below_rate_functions`, as
    `rate:<FUNCTION>`, through the same reader `--all-off` uses.
  - T-0096 (2026-10-29) builds the M8 row of item 2. gen-m8a is the row's `gen_suite`; gen-m8b and
    the poi subset (`poi M8 subset`, read from the report's `poi_m8_subset` section, T-0094) are
    `more_suites`, each with 3,000 scored cells (gen-m8a and gen-m8b) or 99.5% (the subset, 9,950 over
    10,000). A `more_suites` entry carries `blocked_cases` (`case!*`); the subset is not met while it
    holds one. `scripts/exit_owners.py` lists each `more_suites` entry's visibly-off cells and
    blocked cases from the entry's own lists, so a measure over another suite's cells needs no suite
    match.
  - T-0097 (2026-10-29) builds the M10 row of item 4. gen-m10 is the row's `gen_suite` with no
    value-cell floor and none visibly off, and its displayed-text items are a criterion of the same
    suite measure: at least 10,000 (`M10_MIN_TEXT`) with none failing, reported as
    `gen_text_items`, `gen_text_failing` and `gen_text_failing_cells` (`case!Sheet!A1`, from the
    report's failing text items). The M10 probe row's `text_items` and `text_failing` (T-0093) are
    copied to `probe_text_items` and `probe_text_failing`, and `probes_met` needs both counts of
    failures at 0. Other rows leave the text fields null. `scripts/exit_owners.py` lists the failing
    text items as `gen-m10 text` and `probes text` (the probe row's `text_failing_cells`, only when
    the exit row carries `probe_text_items`), and the progress page shows both text counts.
    `scripts/exit_owners.py` reads a function-scope row only where the exit row gives `calc_met` a
    verdict: a null `calc_met` (M7's shown-only row, and M1, M8, M10 and M11, which have no
    ironcalc-calc criterion) lists no ironcalc-calc cell, even when the report has a scope row for
    the milestone (it had listed 119 for M10 and for M11).
