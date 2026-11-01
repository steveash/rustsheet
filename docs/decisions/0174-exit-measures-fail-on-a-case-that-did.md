# 0174 — Exit measures fail on a case that did not run

- **Status:** accepted (2026-10-30, T-3171). It amends decisions 0135 and 0146.
- **Context:** The milestone exit rows counted only scored cells. A case that fails to load or evaluate
  (`EngineError`), panics or times out adds no cell, so it added nothing visibly off. On 2026-10-29 the M2 exit row
  read `met` while the eight `probes/t3097-nesting-paren-*` cases did not load ("Formula is nested too deeply").
  `make exit-owners` already listed those cases, so the owner report and the exit row disagreed. An unloadable case
  is the most visible difference there is, because Excel shows a value in every cell. Two measures already handled
  this: the frozen milestone lists need `cases_run == cases` (decision 0146 item 5), and M8's poi subset fails on a
  crashed or errored case that holds a subset formula (decision 0146 item 2(b)).
- **Decision:**
  - A case whose status is `EngineError`, `Panic` or `Timeout` (`CaseStatus::did_not_run`, the one definition,
    which the poi subset now uses too) fails every exit measure that reads it. That covers three measures:
    - **Generated suite:** a required suite, such as `gen-m<k>`, smoke or corpus-bench, is not met while one of
      its cases did not run.
    - **Probe row:** the probe row of the case's own milestone is not met while one of its cases did not run.
    - **ironcalc-calc:** an ironcalc-calc case that did not run has no cells to classify by function, so it fails
      the measure in every row that has one.
  - The report shows the cases as `N cases did not run` in the exit table and as a `did not run` column in
    "Probe milestones". The exit row lists them as `case!*` (`gen_blocked_cases`, `calc_blocked_cases`,
    `probe_blocked_cases`), and `scripts/exit_owners.py` lists them as whole cases.
  - Pending cases: a pending case that runs cleanly keeps its `Pending` status and never counts. One that crashes
    has a crash status and counts like any other case, because a crash needs no oracle value to be wrong. An
    InvalidCase (an unreadable case file) is a harness problem and stays out of the measures.
- **Consequences:** No exit row can read `met` while a case it measures does not run. A crashing case now holds an
  exit open until it is fixed or pinned (as T-3165 pinned the eight paren-depth cases). An ironcalc-calc crash
  holds every calc-measured exit (M2, M5, M6, M9) at once, which is deliberate: none of its cells can be
  attributed to a milestone.
