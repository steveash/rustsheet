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

Eval: probes l A13:A15, l A40 and m A22 are fixed (5 cells). After the merge with main (below), the new
rounds add 6 known failures, each with an owner and recorded in docs/excel-semantics/number-formats.md:
n A2 and A7:A9 (Excel accepts `a` and `aa` after `_"` but rejects `aaa` and longer runs, while we print
them; T-1881), o A37 (`*` takes `"` as its fill, and we print a space for the fill; T-1735) and q A16
(an unmatched condition printing General; T-0936).

## Blocked because
Review hit the three-round limit (work-loop step 5). Security, architecture and quality approved.
Excel fidelity's third round asked for one thing: evidence for the trailing `\`/`_`/`*` part of
`is_unterminated`. Probe round s (probes/text-format-length-limit-s, 9 cells) supplies it, and every
cell confirms the rule. The doc and code now cite that evidence. To unblock: re-run
`reviewer-excel-fidelity` on the PR; if it approves, set `status: done` and merge.

## Merged up to main (2026-10-18)
The branch was merged with origin/main (372 commits; merge commit, no rebase). Conflicts:
complexity.rs tests (both kept), format.rs (this branch's `value_error`/`format_error` with main's
`TEXT_TOO_LONG`, `BUDDHIST_ERA_OFFSET` and `general_text`), text/common.rs imports, the semantics
entry, and eval/baseline/probes.txt (main's copy, then regenerated). Main's T-1656 `@` placement
and T-1714 AM/PM, A/P and locale costs sit on top of this refit, and their tests pass. Main's T-1262
(bare letters print as literals) already fixed l A13:A15 and now also n A1, A6, o A35, A38:A40.
That leaves 6 new-coverage known failures, not 12: n A2, A7:A9 (`@_"aaa` and longer runs are
`#VALUE!` in Excel, but we print the letters; T-1881), o A37 (`TEXT(7,"0*""a")` is "7a" in Excel
and "7 a" with us; T-1735) and q A16 (T-0936). Probes against main: l A40 and m A22 are fixed, and there
are no new failures. Main had brought the duplication lock down to 4510, and this branch's
tuple tables in `after_quoted_text_and_placeholder_runs` put it 21 lines over. They are now
`prefix|n|suffix` strings, with the same rows. Still to do: re-run `reviewer-excel-fidelity`, then mark the task done and merge.

### Review complete (2026-10-18)
The excel-fidelity re-review approved at 52d7df54: rounds l-s back the refit and the unterminated
rule (round s the trailing `\`/`_`/`*`), and every remaining known failure has an open owner
(T-1881, T-1735, T-0936). Security, architecture and quality approved earlier rounds; quality
re-approved the merge-up.
