---
id: T-1258
title: "REPLACE with a huge start_num is #VALUE!: probe the bound"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
REPLACE (and, if the probe shows the same bound, MID/REPLACEB/MIDB) returns `#VALUE!` for a start_num (and num_chars) above Excel's limit, as Excel does.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09): gen-m6/a62335e72e44cb4e Calc!A9
`=(Data!C5&((Data!E12&Data!B9)&REPLACE(Data!A4,123456789.123,3,"a*")))` with A4 = -0.0001: Excel `#VALUE!`, engine
"1.0050.5-0.0001a*" (`REPLACE(-0.0001,123456789.123,3,"a*")` appends). `replace_element`
(crates/rustsheet-engine/src/functions/text/string_format.rs) only rejects start_num < 1 and num_chars < 0.
The bound is unprobed: probe it first (excel-probe skill) with `REPLACE("abc",32767,1,"x")`, `(..,32768,..)`,
`(..,2^31,..)`, `REPLACE("abc",1,32767,"x")`, `(..,1,32768,"x")`, and the same for MID, MIDB, REPLACEB and LEFT/RIGHT num_chars.
Record the rule in docs/excel-semantics/text.md.

## Acceptance criteria
- [x] An excel-graph probe case with the cells above passes; text.md records the rule (VERIFIED).
- [x] gen-m6/a62335e72e44cb4e A9 passes in `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell
      and no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve.

## Out of scope
- UTF-16 counting of positions (T-0249).

## Outcome
Done in the T-0219 PR: the same probe (probes/text-count-argument-bounds J, K, L, F20-F24) shows REPLACE is
#VALUE! above 32,767 for start_num and num_chars, MID/MIDB from 2^31, LEFT/RIGHT never; `replace_element` bounds
both by `MAX_TEXT_UNITS`. gen-m6/a62335e72e44cb4e A9 passes, no new gen-m6 failure (971 -> 972 passing).
