---
id: T-2279
title: "SUBSTITUTE: probe the instance_num bound (instance_num 1E+15 is #VALUE! in Excel)"
milestone: M6
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
SUBSTITUTE gives Excel's result for a very large instance_num (`#VALUE!` at 1E+15), with the bound probed.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Observation: docs/excel-semantics/text.md "SUBSTITUTE with an instance_num of
1E+15 is `#VALUE!`" (added by this triage). `fn_substitute` (crates/rustsheet-engine/src/functions/text/common.rs) accepts any
instance_num >= 1 and finds no such match. Probe (excel-probe skill, excel-graph, about 15 cells): instance_num 32767, 32768, 65536,
2^31-1, 2^31, 2^32, 2^53, 1E+15, 1E+300 on text with and without a match, and 0.5, 1.5. Record and implement.
SUBSTITUTE is an M5 function (catalog); filed at M6 with the gen-m6 triage. Why opus: the bound needs a probe.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 1 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/9615a6de0f3ac417 Calc!A144 | `=LOOKUP(SUBSTITUTE(("x~*"&Data!B2),RIGHTB(Data!$E4),CHOOSE(3,Data!C8,Data!A2,Data!C4),(Data!D$6+10000000000...` | "#VALUE!" | "#N/A" |

## Acceptance criteria
- [x] This 1 known-failure line disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] A new probes case (excel-graph) holds the grid; text.md states the bound with cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- REPLACE/MID bounds.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
Probed on excel-graph (probes/substitute-instance-num-bound, 68 cells): SUBSTITUTE floors instance_num and is
`#VALUE!` unless it is in 1..=2147483646 (2^31 - 2), match or not, empty text or old_text included (65536,
2^30, 2147483646.9 fine; 2147483647, 2^31, 1E+10, 1E+15, 1E+300 `#VALUE!`). `substitute_element` checks that
range; text.md states it. Eval: gen-m6/9615a6de0f3ac417 Calc!A144 fixed (gen-m6 known failures -1), the new
probe case passes in full, no regressions.
