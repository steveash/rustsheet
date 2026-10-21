---
id: T-2303
title: "Probe TEXT's code check in an unused section and a calendar prefix after a color, condition or currency"
milestone: M10
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2157]
---

## Goal
Excel oracle values settle the two TEXT code-check rules T-2157 built without evidence.

## Context
From T-2157 (number-formats.md "TEXT checks the code before it looks at a text or boolean value"). T-2157 rejects a text or
boolean value when any number section of the code does not parse, and rejects a `B1`/`B2` calendar prefix after a printing
token in its section; probes/t2202-text-code-check-on-text covers only one-section and all-sections-bad codes. Probe with the
excel-probe skill, each on 2.5, -2.5, 0, "a" and TRUE:
1. A bad section a value does not use: `"0;ab1"`, `"ab1;0"`, `"0;0;ab1"`, `"0;0;ab1;@"`, `"0;n"`, `"0;0;0;n@"`.
2. A calendar prefix after a bracket or in a later section: `"[Red]b1"`, `"[>0]b1"`, `"[$€]b1"`, `"[$USD]b1"`, `"0;b1"`,
   `"[$-409]b1"`, `"\"x\"b1"`, `" b1"`, `"0b1"`, `"yyyyb1"`.
Record the answers in number-formats.md and, where Excel differs from T-2157's choice, file the engine task.

## Acceptance criteria
- [ ] A probe case with excel-graph oracle values answers 1 and 2; number-formats.md records them and drops the UNVERIFIED marks.
- [ ] Every new failing cell is in eval/baseline with an owning task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (a follow-up task).
