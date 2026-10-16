---
id: T-1669
title: "Collate þ as th (expansion) and fix œ=oe in XLOOKUP, XMATCH single-cell"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine matches Excel on every failing cell of probes/text-collation-longer (T-1662).

## Context
Oracle (excel-graph, 2026-10-16) shows `þ` = `th` (an expansion like `æ`/`ß`: `"þ"="th"`, `"þ"&"a"` = `"t"&"h"&"a"`, `"þa">"tb"`, `XMATCH("th",þ,0)` = 1), but `cmp_text_excel` (functions/util/collation.rs) treats `þ` as a tiebreak accent on `t` (Calc!A74-A76). Also `XLOOKUP("oe",Data!B1,Data!B1,"nf",0)` must return `œ` (engine #N/A, Calc!A102), and `XMATCH("th",Data!C1,0)`, `XMATCH("1",Data!D1,0)`, `XMATCH("ab",Data!A1,0)` return #VALUE! in the engine where Excel gives 1, #N/A, #N/A (A104, A105, A107; find out why a single-cell lookup_array of a text-formula cell errors). Run `rustsheet-eval run --suite probes --filter text-collation-longer`.

## Acceptance criteria
- [ ] Calc!A74-A76, A102, A104, A105, A107 of probes/text-collation-longer pass; baseline updated.
- [ ] coercion.md notes `þ` = `th` implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
