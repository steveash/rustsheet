---
id: T-1817
title: "REGEXREPLACE bounds its result before it builds it"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
REGEXREPLACE cannot allocate an unbounded result: a result past Excel's 32,767-character text limit is the error Excel
gives (probe it), and the size is checked before the replacement text is built.

## Context
T-1272's security review: `re.replace_all` in functions/text/regex_functions.rs (`fn_regexreplace`) builds its whole
result first. An empty or `a*` pattern over a 32,767-character text with a 32,767-character replacement allocates about
1 GB, and a lift repeats it per element. Already true on main before T-1272. Count matches x replacement length first
(see `substitute_exceeds_limit` and `within_text_budget` in functions/text/util.rs, which SUBSTITUTE and the text
broadcast use), and use the excel-probe skill for what Excel returns past 32,767 characters (`#VALUE!` is the usual text
limit). Record the result in docs/excel-semantics/text.md.

## Acceptance criteria
- [ ] `=LEN(REGEXREPLACE(REPT("a",32767),"","bbbbbbbbbb"))` returns Excel's probed result at once, without building the
  full replacement (a unit test).
- [ ] text.md records the rule, with a DEVIATION entry if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting changes (T-1272).
