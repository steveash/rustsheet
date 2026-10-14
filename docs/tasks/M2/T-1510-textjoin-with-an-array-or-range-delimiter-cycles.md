---
id: T-1510
title: "TEXTJOIN with an array or range delimiter cycles the delimiters"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTJOIN with an array literal or a range as the delimiter cycles through its elements between the joined values, as Excel does (`TEXTJOIN({",",";"},FALSE,{1,2,3})` is `1,2;3`).

## Context
Found in T-1482. Probe case `textjoin-delimiter-array` (A1-A6, excel-graph) fails: the engine reads only a scalar delimiter (`fn_textjoin` in functions/text/concatenate.rs, `get_string(&args[0])`). Semantics in `docs/excel-semantics/text.md` (TEXTJOIN over arrays).

## Acceptance criteria
- [ ] `probes/textjoin-delimiter-array` passes and its known failures leave the baseline.
- [ ] Delimiter cycling interacts correctly with the 32,767 limit (`JoinSize`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
