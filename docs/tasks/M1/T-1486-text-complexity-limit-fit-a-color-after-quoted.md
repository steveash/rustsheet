---
id: T-1486
title: "TEXT complexity limit: fit a color after quoted text, _' and a quoted run before @ (follow-up of T-1481)"
milestone: M1
lane: functions-text
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT's complexity model matches Excel on the three cells left unexplained by T-1481: probes/text-format-length-limit-l A13:A15 and A40, and -m A22. (Note, T-1262: l A13:A15 now pass, as an unterminated quoted string after `_"`; only l A40 and m A22 remain.)

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
Six probe rounds (probes/text-format-length-limit-n to -s; 156 cells, excel-graph) pinned the
misses and refitted `formatter/complexity.rs`:
- a section with digit placeholders pays 3 once, plus 2 per placeholder run (was 5 per run);
- a later section that holds a placeholder and quoted text pays 1 more;
- after quoted text, a token pays 4 directly, 2 after unquoted text and 1 after a color or condition;
- a placeholder run pays 1 after quoted text, or 0 after a color or condition;
- the condition-with-quoted 3 only applies with a placeholder.

The `@_"` cells were not a cost question: `_` takes the quote, which leaves an unclosed one. TEXT now
rejects an unterminated code (an open quote or bracket, or a trailing `\`, `_` or `*`) for any value,
even when the section the value uses is complete (`TEXT(5,"0;""")` is `#VALUE!`).

Eval: probes l A13:A15, l A40 and m A22 are fixed (5 cells). The new rounds add 12 known failures.
Eleven are unquoted letters after `_"`/`*"` (n A1, A2, A6:A9, o A35, A37:A40; T-1450): Excel accepts
`a` and `aa` there and rejects `aaa` and longer runs, while our lexer rejects all of them. One is an unmatched
condition printing General (q A16, T-0936). All are recorded in docs/excel-semantics/number-formats.md.

## Blocked because
Review hit the three-round limit (work-loop step 5). Security, architecture and quality approved.
Excel fidelity's third round asked for one thing: evidence for the trailing `\`/`_`/`*` part of
`is_unterminated`. Probe round s (probes/text-format-length-limit-s, 9 cells) supplies it, and every
cell confirms the rule. The doc and code now cite that evidence. To unblock: re-run
`reviewer-excel-fidelity` on the PR; if it approves, set `status: done` and merge.
