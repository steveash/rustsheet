# 0118 — Phase 1 exit counts each cell toward its formula's single non-glue function

- **Status:** accepted (2026-10-25, owner decision in session
  https://claude.ai/code/session_014ASuNyVecGvKjcZUDfWLKA; SPEC-0022, T-0067).
  Amended by decision 0188 (2026-10-31): the exit counts this measure on user-representative suites only (Gate B).
- **Amends:**
  - decision 0114, items 1 (the Phase 1 exit) and 4 (its measure);
  - how decision 0088's per-function `visible` blame is used;
  - the process part of decision 0094: the steward no longer copies the checkpoint line into
    docs/status.md. SPEC-0022 and T-0076 replace that step once T-0076 lands. Decision 0094's
    checkpoint definition is unchanged.
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
  1. **Attributed cells.** A function's Phase 1 share (decision 0114 items 1 and 4) is measured
     over its attributed Excel cells: the scored, non-deviation Excel-evidence cells whose formula
     calls exactly one distinct function outside the harness glue list (`closeness::GLUE`).
     - Repeated calls of that function count as one, so `GAMMA(1)+GAMMA(2)` is attributed to
       GAMMA.
     - A formula that calls only glue is attributed only when it calls exactly one distinct glue
       function.
  2. **Fallback.** A function with fewer than 20 attributed cells (`EVIDENCE_FLOOR`) is measured
     over every Excel cell that calls it instead.
  3. **The rate.** The function meets the Phase 1 rate when at least 99.9% of those cells are not
     visibly off. That uses decision 0088's buckets and `milestone_exit::RATE_NUM / RATE_DEN`.
     Coverage stays decision 0094's (400 Excel cells, 200 generated, counted over every cell that
     calls the function).
  4. **Report.** `v15_checkpoint`'s `meets_rate`, `at_rate`, `phase1_met` and its Phase 1 line
     use this count (T-0080).
  5. **Information only.** The stricter count, where every cell that calls the function and every
     non-glue blame counts, is still reported next to it. It is labelled as information and gates
     nothing: no exit, checkpoint, task pick or CI job reads it.
  6. **Unattributed cells.** Some visibly-off cells are attributed to no function: those that call
     two or more distinct non-glue functions, and glue-only formulas with two or more distinct
     calls. They still fall under decision 0114 item 1's rule that every other cell has a filed
     task or a directive-5 `DEVIATION`. Phase 1 does not exit while such a cell has neither.
- **Difference from decision 0088's `blamed`:** a glue-only formula with two or more distinct calls,
  such as `IF(AND(A1,B1),1,2)`, is attributed to nobody here. Under 0088 it blames both IF and AND.
  The two counts differ there on purpose: 0088's blame feeds the information count, and this rule
  feeds the exit. T-0068 tests both.
- **Consequences:**
  - T-0068 computes the attributed counts, and T-0080 switches `v15_checkpoint` to them. Until
    T-0080 merges, the report's Phase 1 line still shows the stricter count.
  - The progress dashboard (SPEC-0022) shows the attributed count as the exit measure, and the
    stricter one labelled as information.
  - Decision 0088's lock and per-suite visibly-off counts are unchanged. Its per-function `visible`
    field is unchanged too; it now feeds only the information count.
  - Decision 0094's zero-visibly-off checkpoint is unchanged.
  - No gate or baseline changes. The CI gate still forbids any bit regression.
  - A future change to `GLUE` changes the exit measure, so it needs a decision record.
