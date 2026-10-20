---
id: T-2116
title: "Decision record: how M5's exit criteria are measured, and the milestone of fit tasks for M5 functions"
milestone: M5
lane: research
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A decision record in docs/decisions/ (`make new-decision`) fixes how each of M5's four exit criteria
is measured. docs/milestones.md's M5 section points to it. Each M5 exit gap known today then has an
owner: a task id in a milestone that the pick order reaches while M5 is current, or a named exception.

## Context
M5 exit (docs/milestones.md): "catalog coverage 100% for these categories; bit-exact ≥ 99.9% on
generated suites (≥ 20k cells); 100% on ironcalc-calc for these categories; its `probes` cases
(decision 0051) at 100% bit-exact". M6 reuses the wording ("same as M5"). Open questions, from the
2026-10-20 planner run:
1. **Which ironcalc-calc cells count?** M2 used "every called function has catalog milestone ≤ M2"
   (T-1472). The natural extension is a cumulative ≤ M5 rule, which T-2111 reports. The catalog
   milestone, not Microsoft's category, decides: LAMBDA, LET, SEQUENCE, RANDARRAY, REGEX* and
   TEXTSPLIT are in Microsoft's Logical, Math or Text categories but have catalog milestone M7.
2. **IF_ARRAY C4 and E4.** `LOGICAL/IF_ARRAY.xlsx` Sheet1!C4 and E4 are the Mac-desktop SIN(4) cells
   of decision 0074. That decision exempted only C11, the cell in M2's scope. C4 (`IF(...,SIN(..),COS(..))`)
   and E4 (`SIN(A1:A4)`) enter M5's scope. Extend the exception to them or not.
3. **Fit tasks filed under M9 for M5 functions.** ironcalc-calc cells of M5 functions fail in
   T-0917 (ATANH/ACOTH/COTH/CSCH/TANH/SINH, functions-math), T-0915 (SIN/COS/TAN/SEC/CSC/COT
   argument reduction), T-0929 (VALUE percent, functions-text) and T-1944 (EXP edges). All four carry
   milestone M9, from the numeric track of decision 0056. While M5 is the current milestone, the pick
   order (decisions 0023 and 0026) reaches them only after M5–M8. Decide whether they move to M5 (edit
   their front matter in this PR) or stay in M9 with a stated reason. Decision 0056's time-box and
   "residue within 4 ulp" rule apply either way: say how a time-boxed, blocked fit task counts
   against "100% on ironcalc-calc".
4. **Catalog coverage.** Define "100%" for the T-2112 section, (a) ≥ 20 Excel cells and (b) cells in
   gen-m5. Volatile RAND and RANDBETWEEN can never have scored gen-m5 cells; decide whether probes
   (shape checks such as `probes/volatile-and-info-rand-true-false-sheet`) satisfy them. INFO and
   SHEETS are `oracle: none`; T-2113/T-2115 give them web evidence, but bare `SHEETS()` and
   `INFO("numfile")` stay web-unprobeable. Say whether an M14 item counts as their filed task
   (decision 0039 rule 3). Also say whether the out-of-scope M5 names (CALL, PY, WEBSERVICE, ...,
   decision 0027) are excluded, as `scope: out` implies.
5. **Generated suites.** Confirm that the criterion counts gen-m5 alone (≥ 20,000 scored Excel-backed
   cells, T-2114), excludes accepted deviations as decision 0036 does, and is measured from the suite
   row of `make eval`.
Precedents: decision 0051 (probe milestones), 0074 (IF_ARRAY exception), 0039 (web oracle of record,
M14 items), 0056 (numeric fits), 0062 (evidence floor). The model is T-1470, a research-lane
decision task that settled M2's probe accounting.

## Acceptance criteria
- [ ] One accepted decision record answers questions 1–5 above, each in its own bullet, citing the
      2026-10-20 numbers (report.md) it relies on.
- [ ] docs/milestones.md M5 exit text cites the decision and states any named exception, as M2's text
      cites decision 0074.
- [ ] If the decision moves tasks to M5, their front matter `milestone` is edited in this PR and
      `python3 scripts/tasks.py validate` passes. No task is created or closed by this PR.
- [ ] `make check` green (scripts/check_decisions.py); reviewers approve.

## Out of scope
- Implementing the report sections (T-2111, T-2112).
- Fitting any function.
- Changing M6–M10 exit text, beyond noting that M6's "same as M5" inherits the definitions.
