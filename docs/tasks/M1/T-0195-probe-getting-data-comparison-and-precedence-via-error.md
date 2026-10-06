---
id: T-0195
title: "Probe #GETTING_DATA comparison and precedence via ERROR.TYPE, and stored t=e error values"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Pin which error wins when `#GETTING_DATA` meets `#N/A` or `#DIV/0!`, and whether a stored `t="e"` error constant loads, since T-0175 could not tell them apart.

## Context
Follow-up of T-0175 (docs/excel-semantics/syntax.md "Newer error literals"). The Graph API reports `#GETTING_DATA` as `#N/A`, so probes/error-literals-new A7-A9, A13 and A18 cannot distinguish the two. Wrap each in `ERROR.TYPE(...)` (case `eval/suites/probes/cases/error-literals-new2.case.json`): `=ERROR.TYPE(#N/A=#GETTING_DATA)`, `=ERROR.TYPE(#GETTING_DATA=#N/A)`, `=ERROR.TYPE(#GETTING_DATA+#DIV/0!)`, `=ERROR.TYPE(#DIV/0!+#GETTING_DATA)`, `=ERROR.TYPE(#GETTING_DATA&"a")`, and also `*`, `-`, `SUM`. Second part: spec2xlsx writes error literals as formulas; if it can be extended (a harness change, own task if so) to emit `<c t="e"><v>#BLOCKED!</v></c>`, probe whether stored `#BLOCKED!`, `#CONNECT!`, `#FIELD!`, `#BUSY!`, `#UNKNOWN!` cells load. Otherwise record the gap and stop.

## Acceptance criteria
- [ ] Probe case and excel-graph sidecar committed; results in syntax.md (precedence and comparison results with ERROR.TYPE evidence).
- [ ] The stored-error-constant question is answered, or the harness gap is filed as its own task.
- [ ] New-case engine mismatches are baselined; no engine change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine support for the new kinds (T-0177).

## Outcome
Probe `error-literals-new2` (15 `ERROR.TYPE` cells, excel-graph) shows the leftmost error operand wins for comparison, `&`, `+`, `*`, `-` and `SUM`; `#GETTING_DATA` has no special precedence. Recorded in syntax.md. The stored `t="e"` question needs a harness change, filed as T-0999. No engine change; probes known failures 339 -> 354 (15 new baselined cells).
