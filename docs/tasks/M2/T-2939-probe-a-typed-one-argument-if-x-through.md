---
id: T-2939
title: "Probe a typed one-argument IF(x) through typed entry on excel-graph (probes/typed-if-one-argument stays pending: the stored form is refused)"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
We know from Excel for the web what a typed `=IF(x)` (IF with only a condition, an M2 function) does. Either Excel
refuses the entry, or it stores the formula and gives values. The engine's behavior is then recorded as matching, or a fix
task is filed that names the cells.

## Context
- T-1990 (done) built probes/typed-if-one-argument, which stores `=IF(TRUE)`, `=IF(FALSE)`, `=IF(1)`, `=IF(0)`,
  `=IF({1,0})`, `=IF(A1:B1)`, `=IF("x")`, `=IF(NA())`, `=ISLOGICAL(IF(TRUE))` and `=IF(TRUE)+1` in column E. excel-graph
  refuses every workbook that stores a one-argument IF: `createSession` returns 501 `FileCorruptTryRepair`. The case is
  still pending. docs/status.md (2026-10-27 snapshot) still lists it among the failing pending fills.
- docs/excel-semantics/arrays.md ("Functions passed by name", the paragraph that starts "A one-argument `IF(x)` stored in
  a file") says whether Excel accepts the typed form "is entry-only and unanswerable here (desktop question, M14)". Since
  then, typed entry has been probed on excel-graph. An `{"entered": "=..."}` cell is left blank in the file, and the
  oracle types the text through Graph `values`. Excel's refusal comes back as a 400 `InvalidArgument` (T-2040, T-2497,
  decision 0103; tools/src/rustsheet_tools/spec2xlsx.py "Typed entry"). The workbook itself then holds no one-argument IF,
  so it loads. So the question can be answered on the web oracle of record (decision 0039), and it needs no desktop.
- Ours: a typed `=IF(x)` goes through the one-argument eta path, `Model::if_pick`
  (crates/rustsheet-engine/src/functions/logical/mod.rs, T-1947). That gives `#VALUE!` for each condition that is not
  an error, passes an error on, and lifts over arrays (probes/eta-if-one-argument, 40/40).
- Method (excel-probe skill): add a new case, for example probes/typed-if-one-argument-entered (tag M2). Each column-E
  question of the old case becomes an `entered` cell. Probe a single entry `=IF(TRUE)` first, in a case of its own,
  because a refused entry fails the whole fill (T-2040 committed only the accepted control). Next to each accepted entry,
  add `=FORMULATEXT(..)`, `=ISFORMULA(..)` and `=TYPE(..)`, so the case shows what Excel stored and what it evaluates.
  The old stored-form case stays as the record of the 501. Do not delete it.

## Acceptance criteria
- [x] Either a committed, oracle-filled typed-entry case with Excel's values for the entries above, or (if Excel refuses
      them) the recorded `InvalidArgument` result for each refused entry, with the accepted control committed and the
      refusal reproducible as T-2040 describes.
- [x] docs/excel-semantics/arrays.md replaces the "entry-only and unanswerable here" sentence with the probed rule,
      Status and Evidence. T-1075 has no item for this question (checked 2026-10-27), so no M14 item changes.
- [x] If ours differs (Excel refuses where we accept, or Excel's values differ from `#VALUE!` per element), a fix task is
      filed (engine lane, M2) that names the case and cells. Under decision 0103 a refusal belongs to `Input::Parse`
      typed input, not to stored formulas. If ours matches, the new case passes in `make eval`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The eta path `_xleta.IF` (T-1947) and IF with two or three arguments.
- Getting a stored one-argument IF to load in excel-graph, or how desktop Excel writes one (M14).
- Implementing a refusal or a value change (the follow-up task does that).

## Outcome
Excel for the web refuses every typed one-argument IF: `=IF(TRUE)`, `=IF(FALSE)`, `=IF(1)`, `=IF(0)`, `=IF({1,0})`,
`=IF(A1:B1)`, `=IF("x")`, `=IF(NA())`, `=ISLOGICAL(IF(TRUE))` and `=IF(TRUE)+1`, each typed alone, gave Graph PATCH 400
`InvalidArgument` (2026-10-28). The accepted controls are committed as probes/t2939-typed-if-two-argument-controls
(`=IF(TRUE,)` 0, `=IF(TRUE,1)` 1, `=IF(FALSE,1)` FALSE, with FORMULATEXT and ISFORMULA; 9/9 pass). No TYPE column: the
oracle-typed values already show each type. Ours already
matches: the typed argument-count refusal (T-2568) refuses all ten, and the new unit test
`typed_if_with_one_argument_is_refused` pins that. arrays.md now records the rule in place of "entry-only and
unanswerable here". The stored-form case probes/typed-if-one-argument stays pending as the record of the 501. No fix
task is needed. Eval delta: probes +1 case, 9 cells, all passing; 0 regressions.
