---
id: T-1356
title: "ARRAYTOTEXT writes numbers with 15 significant digits like VALUETOTEXT and concatenation"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ARRAYTOTEXT writes numbers as Excel's General text (15 significant digits, E notation for large values), the same text VALUETOTEXT and `&` give.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/d38b52bfa1a9f9e6 Calc!A2 `=(IF((1.005<Data!B6),MATCH(2,Data!B1:E12,1))&ARRAYTOTEXT(YEARFRAC(1911,31142)))`:
Excel "FALSE80.0277777777778", engine "FALSE80.02777777777777". `format_array_value` in
crates/rustsheet-engine/src/functions/text/arraytotext.rs uses Rust's shortest `{}` text: `ARRAYTOTEXT(1/3)` is
"0.3333333333333333" and `ARRAYTOTEXT({1E+20})` "100000000000000000000", while `VALUETOTEXT(1/3)` is "0.333333333333333" and
`""&1E+20` "1E+20". docs/excel-semantics/text.md "ARRAYTOTEXT number text". Reuse the number-to-text helper VALUETOTEXT and
concatenation use; do not write another.

## Acceptance criteria
- [ ] gen-m7/d38b52bfa1a9f9e6 A2 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: ARRAYTOTEXT of 1/3, 1E+20 and an array of both, in format 0 and 1, equal VALUETOTEXT's text.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
