---
id: T-1627
title: "Probe NOW, TODAY, RAND, TRUE, FALSE, SHEET, ISOMITTED, TOROW, WRAPCOLS, WRAPROWS to the evidence floor"
milestone: M2
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): NOW (5), TODAY (6), RAND (4), TRUE (6), FALSE (8), SHEET (16), ISOMITTED (17), TOROW (14), WRAPCOLS (16), WRAPROWS (16) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
volatile results checked by shape-stable formulas (INT(NOW())=TODAY(), RAND() in [0,1), ISNUMBER, arithmetic with them), TRUE()/FALSE() with arguments, SHEET with references, names, text and no argument, ISOMITTED inside LAMBDA with omitted/blank/supplied arguments, TOROW ignore modes 0-3 and scan_by_column, WRAP* wrap_count 0/1/larger than the vector and pad_with. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [x] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [x] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [x] Quirks recorded in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).

## Outcome
Three probe cases (probes/volatile-now-today-shapes 48 cells, volatile-and-info-rand-true-false-sheet 108, dynamic-array-reshape-and-isomitted 132; excel-graph, 2026-10-15) lift all ten functions over the floor (quality/evidence-floor.json 73 -> 63). 21 new-coverage failures, all WRAPROWS/WRAPCOLS (T-1630) or SHEET (T-1631); NOW, TODAY, RAND, TRUE, FALSE, ISOMITTED and TOROW match. Quirks in arrays.md and information.md. The lock drop is logged in decision 0010 (decision 0062 now says so). Follow-up: T-1632 (re-probe the two spill-collision cells B41/C44).

## Handoff (resumed)
- Done and pushed on this branch: three probe cases with excel-graph oracle sidecars (volatile-now-today-shapes, volatile-and-info-rand-true-false-sheet, dynamic-array-reshape-and-isomitted), quirks in arrays.md and information.md, fix tasks T-1630 and T-1631, `make eval-baseline` output (probes baseline +3 case lines, 21 new-coverage F lines; `quality/evidence-floor.json` 73 -> 63). The security, quality and architecture reviewers approved; reviewer-excel-fidelity reported REQUEST_CHANGES on doc wording only (boolean wrap_count, fractional wrap_count, ISOMITTED results, the clock sentence); fixed in a later commit, so it needs one confirming re-run.
- Blocker: `python3 scripts/check_governance.py origin/main` fails with "protected paths changed without a decision record in docs/decisions/: quality/evidence-floor.json". Decision 0062 says probe PRs commit the lower lock, but check_governance.py has no exemption, so every T-1620..T-1628 batch hits it (the CI `hygiene` job will fail). Never hand-edit around it.
- Next (opus): either amend decision 0062 (new decision record via `make new-decision`) so check_governance.py exempts a strict decrease of `below_floor` at an unchanged `floor`, as its own gate change, or add a short decision record to this PR saying the lock was lowered 73 -> 63 by `make eval-baseline`. Then re-run `make check`, re-run reviewers (reviewer-excel-fidelity has not reported on the final head), merge main, and merge.
- Optional reviewer nits: the single-tag M5 case mixes RAND/TRUE/FALSE/SHEET (could split the SHEET cells out); T-1630 and T-1631 have an empty "Out of scope" bullet.
- Resolved (opus, 2026-10-15): the drop 73 -> 63 is logged as a paydown line in decision 0010, the convention for ratchet decreases, and decision 0062 now tells probe PRs to do the same; `check_governance.py` passes unchanged. Both nits addressed: out-of-scope lines filled; the M5 case is left whole (decision 0057 allows it). All four reviewers re-run on the final head.
