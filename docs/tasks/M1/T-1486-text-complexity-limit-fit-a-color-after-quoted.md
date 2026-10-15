---
id: T-1486
title: "TEXT complexity limit: fit a color after quoted text, _' and a quoted run before @ (follow-up of T-1481)"
milestone: M1
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT's complexity model matches Excel on the three cells left unexplained by T-1481: probes/text-format-length-limit-l A13:A15 and A40, and -m A22. (Note, T-1262: l A13:A15 now pass, as an unterminated quoted string after `_"`; only l A40 and m A22 remain.)

## Context
T-1481 fitted what `@` costs. Left over (see "Not explained" in docs/excel-semantics/number-formats.md, formatter/complexity.rs): `@_` followed by 123..125 quoted characters is `#VALUE!` in Excel (the `_` takes the opening quote as its pad character) and we differ; `"0"` + 118 quoted + `"[Red]0"` (a color after quoted text) works in Excel and we reject it; a quoted `a`, then ` @ `, then 121 quoted fails in Excel and we accept it. Probe more around each before refitting.

## Acceptance criteria
- [ ] Probe rounds pin each of the three misses; the model and the semantics entry follow them; the cells leave the known failures (or the remainder is listed with evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Letters as literals (T-1450), conditions (T-0936).
