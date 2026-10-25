# 0118 — Phase 1 exit counts each cell toward its formula's single non-glue function

- **Status:** accepted (2026-10-25, owner decision in session
  https://claude.ai/code/session_014ASuNyVecGvKjcZUDfWLKA; SPEC-0022, T-0067). Amends decision
  0114 (item 4, the Phase 1 measure) and how decision 0088's per-function `visible` blame is used.
  Decision 0094's checkpoint is unchanged.
- **Context:**
  - Decision 0114 says Phase 1 exits when every in-scope function has ≥ 99.9% of its Excel cells
    within 15 significant digits. `v15_checkpoint` measures each function's share as follows:
    - the cells are every Excel cell that calls the function;
    - the visibly-off cells are blamed on every non-glue function the formula calls (decision
      0088's `visible`).
  - Under that count, one off sub-expression counts against every specialised function in a long
    formula. Counted the same way for closeness, 485 of 506 functions would show as off.
  - The 2026-10-25 assessment instead counted a cell toward a function only when that function
    was the formula's single non-routing call. It found 308 functions never visibly off and 127
    below 99%. The owner judged that count the fair measure of each function's parity.
  - SPEC-0022 (the daily progress dashboard) needs one official exit number.
- **Decision:**
  1. A function's Phase 1 share is measured over its **attributed** Excel cells: the scored,
     non-deviation Excel-evidence cells whose formula has that function as its only call outside
     the harness glue list (`closeness::GLUE`). A glue-only formula with a single call is
     attributed to that call.
  2. A function with fewer than 20 attributed cells (`EVIDENCE_FLOOR`) is measured over every
     Excel cell that calls it instead.
  3. The function meets the Phase 1 rate when at least 99.9% of those cells are not visibly off,
     using decision 0088's buckets and `milestone_exit::RATE_NUM / RATE_DEN`. Coverage stays
     decision 0094's (400 Excel cells, 200 generated, counted over every cell that calls the
     function).
  4. `v15_checkpoint`'s `meets_rate`, `at_rate`, `phase1_met` and its Phase 1 line use this
     count (T-0068).
  5. The stricter count, where every cell that calls the function and every non-glue blame
     counts, is still reported next to it. It is labelled as information and decides nothing.
- **Consequences:**
  - T-0068 implements the attributed counts and switches `v15_checkpoint` to them. Until it
    merges, the report's Phase 1 line still shows the stricter count.
  - The progress dashboard (SPEC-0022) shows the attributed count as the exit measure, and the
    stricter one labelled as information.
  - Decision 0088's lock and per-suite visibly-off counts are unchanged, and so is its
    per-function `visible` field, which stays as the information count. Decision 0094's
    zero-visibly-off checkpoint is unchanged.
  - No gate or baseline changes. The CI gate still forbids any bit regression.
  - A future change to `GLUE` changes the exit measure, so it needs a decision record.
