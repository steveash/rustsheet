---
id: T-1787
title: "Delete the four private calc_result_to_array_node copies in functions/logical"
milestone: M9
lane: quality
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
There is one definition of "a `CalcResult` as an array element": `cast::calc_result_to_array_node`. The four private copies in functions/logical/ are deleted (directive 3).

## Context
Found by the T-1767 quality review of SPEC-0011, whose driver uses the shared helper instead of adding a fifth copy. The private copies are in functions/logical/mod.rs (`fn calc_result_to_array_node`), logical/scan.rs, logical/bycol_byrow.rs and logical/map_reduce.rs. Check each against cast.rs's version first: if a copy maps some variant differently (a LAMBDA, a range), keep that difference visible at its call site and cite its evidence.

## Acceptance criteria
- [ ] `rg -n "fn calc_result_to_array_node" crates/rustsheet-engine/src` finds only cast.rs.
- [ ] `make eval`: 0 fixes and 0 regressions. `scripts/check_duplication.py` does not rise, and the drop is recorded with `--update`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `array_node_to_calc_result` (functions/util.rs), which is already shared.

## Outcome
Deleted the four private copies; `cast::calc_result_to_array_node` is the only definition. The
BYROW/BYCOL/MAKEARRAY 1x1-array unwrap stays visible as `lambda_result_to_array_node` in
bycol_byrow.rs (probes/byrow-bycol-array-results). Eval: 0 fixes, 0 regressions. Duplication
6155 -> 6111 (`--update`).

## Handoff
Code is complete and all four reviewers approved (security, architecture, quality, excel-fidelity);
`make check` and eval gate were green locally (0 fixes, 0 regressions). CI `hygiene` fails in
`scripts/check_governance.py`: `quality/duplication.json` (written by `check_duplication.py
--update`, 6155 -> 6111) is a protected path and the PR has no decision record. A sonnet run may
not add one. Next: `make new-decision TITLE="..."` (or confirm an existing decision already
covers ratchet-down `--update` of quality/*.json), commit it on this branch, push, then restore
`status: done` and re-merge per the work-loop. PR: #1009.
