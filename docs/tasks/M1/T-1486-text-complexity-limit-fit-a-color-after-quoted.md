---
id: T-1486
title: "TEXT complexity limit: fit a color after quoted text, _' and a quoted run before @ (follow-up of T-1481)"
milestone: M1
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT's complexity model matches Excel on the three cells left unexplained by T-1481: probes/text-format-length-limit-l A13:A15 and A40, and -m A22.

## Context
T-1481 fitted what `@` costs. Left over (see "Not explained" in docs/excel-semantics/number-formats.md, formatter/complexity.rs): `@_` followed by 123..125 quoted characters is `#VALUE!` in Excel (the `_` takes the opening quote as its pad character) and we differ; `"0"` + 118 quoted + `"[Red]0"` (a color after quoted text) works in Excel and we reject it; a quoted `a`, then ` @ `, then 121 quoted fails in Excel and we accept it. Probe more around each before refitting.

## Acceptance criteria
- [x] Probe rounds pin each of the three misses; the model and the semantics entry follow them; the cells leave the known failures (or the remainder is listed with evidence).
- [x] `make check` green; reviewers approve.

## Out of scope
- Letters as literals (T-1450), conditions (T-0936).

## Handoff (resumed)
Escalated by a sonnet run before any code change: the task is numeric fitting that needs several
oracle probe rounds (probes/text-format-length-limit-l and -m live in the oracle history, not as
suites under eval/suites/probes in this checkout; look in eval/ for the case ids first).
`rustsheet-oracle status` showed excel-graph OK, so probing is possible. Starting points:
`formatter/complexity.rs` (`Section::settle`, `QUOTED_SWITCH_COST`, `TEXT_TOKEN_EXTRA_REFUND`)
and the "Not explained" bullet in docs/excel-semantics/number-formats.md. Next: probe each of the
three misses (`@_` + 122..126 quoted; `"0"` + n quoted + `[Red]0` for several n; quoted `a` + ` @ `
+ n quoted for several n), refit, update the semantics entry and the baseline.

## Outcome
Three probe rounds (probes/text-format-length-limit-n, -o, -p; 107 cells, excel-graph) pinned the
misses and refitted the model in `formatter/complexity.rs`: a section with digit placeholders pays
3 once plus 2 per placeholder run (was 5 per run); after quoted text a token pays 4 directly and 2
when unquoted text, a color or a condition comes between, a placeholder run pays 1, or 0 after a
color or condition. The `@_"` cells were not a cost question: `_` takes the quote, leaving an
unclosed one, and TEXT on text with an unparseable code is now `#VALUE!` (it returned the text).
Eval: probes l A13:A15, A40 and m A22 fixed (5 cells); the new rounds add 7 known failures, all
unquoted letters after `_"`/`*"` (T-1450's scope), recorded in docs/excel-semantics/number-formats.md.
