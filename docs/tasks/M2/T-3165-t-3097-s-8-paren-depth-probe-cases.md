---
id: T-3165
title: "T-3097's 8 paren-depth probe cases count against M2 exit: pin them as decision-0036 deviations or lift MAX_FORMULA_DEPTH past Excel's paren nesting"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The M2 exit row's probes column no longer counts T-3097's eight paren-depth cases as unowned
known failures: each is either an accepted, pinned deviation (decision 0036) or matches Excel.

## Context
T-3097 (#2312) added 44 nesting probes. Eight of them do not run: `probes/t3097-nesting-paren-128-stored`, `probes/t3097-nesting-paren-128-typed`, `probes/t3097-nesting-paren-255-stored`, `probes/t3097-nesting-paren-255-typed`, `probes/t3097-nesting-paren-abs-64-stored`, `probes/t3097-nesting-paren-abs-64-typed`, `probes/t3097-nesting-paren-abs-65-stored` and `probes/t3097-nesting-paren-abs-65-typed` fail to load as a whole (not only some cells) with "Formula: Formula is nested too deeply"
(`MAX_FORMULA_DEPTH` = 256 in `crates/rustsheet-engine/src/expressions/parser/depth.rs`), where
Excel accepts them. T-3097's Outcome calls this the depth-bound DEVIATION (directive 5, decision
0014) but baselined them as known failures, so the 2026-10-29 18:47 UTC steward run's
`make exit-owners MILESTONE=M2` listed all eight as having no open M2 owner, and they sit in the
M0-M2 probe row's known failures. Two ways out; the implementer picks with evidence:
- Lift the bound: count only what needs the native stack (e.g. a parenthesis group could cost less
  than a call frame) so Excel's paren nesting fits, with the fuzz and recursion tests still
  bounded (directive 5).
- Keep the bound: pin each case's cells in `eval/suites/probes/deviations` against a DEVIATION
  entry in docs/excel-semantics (decision 0036; check that the deviations file can pin a case that
  fails to load, and extend `crates/rustsheet-eval/src/deviations.rs` if it cannot).

## Progress
- 2026-10-29 (opus run, session budget ran out before an implementation): all eight cases are `status=EngineError`,
  `checked=0` (`make eval-show`: "invalid input for A1: Formula: Formula is nested too deeply"), for the `-stored`
  cases too. The spec adapter enters every formula through the API, and the parser refuses the too-deep formula.
  The baseline records them as `F <case> !EngineError` with `C <case> 0`. So the "keep the bound" route cannot pin
  them with today's `deviations` file: `deviations.rs` pins a scored cell (`Sheet!A1`, compared in
  `runner.rs` around line 437), and an EngineError case scores no cells (`runner.rs` `Ok(Ok(Err(e)))` branch).
  Options seen, not yet weighed:
  (1) a case-level deviation line (pinned `!EngineError` for the whole case), which needs a format extension in
  deviations.rs and its readers (report, probe_milestones, milestone_lists, and `scripts/exit_owners.py`
  `PASSING`);
  (2) have the spec adapter mirror what RustSheet does by design: a refused typed formula leaves the cell as it
  was (`EditError::InvalidInput`), and a stored one loads as a parse error like the xlsx importer's. The cells then
  score, and per-cell deviation lines pin A1/B1;
  (3) lift `MAX_FORMULA_DEPTH`. 255 parentheses cost about 510 levels at `PRIMARY_DEPTH` 2, about 1 MiB of parser
  stack by depth.rs's 2 KB per level, so a group would have to cost less than a call frame.
  (2) changes harness policy (today any refused input fails the case loudly), so it probably needs a decision
  record.

## Acceptance criteria
- [ ] `make exit-owners MILESTONE=M2` no longer lists the eight t3097-nesting-paren cases.
- [ ] The DEVIATION entry (if kept) names the bound, the cases, and Excel's own limit as probed.
- [ ] No regressions; `make check` green; reviewers approve.

## Out of scope
- The 65-call function nesting cap (T-3097, done) and T-3143's follow-ups.
