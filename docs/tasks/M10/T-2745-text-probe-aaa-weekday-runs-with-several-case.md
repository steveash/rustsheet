---
id: T-2745
title: "TEXT: probe aaa weekday runs with several case changes and aaam/pm (follow-up of T-1880)"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Pin what Excel does with `a` runs that change case more than once, and with `aaa` right before `m/pm`, and make TEXT match.

## Context
T-1880 found that three or more of one `a` (one case) are the weekday code, and that a run of three
or more of the other case right after it joins the code (`TEXT(1,"aaaAAA")` = "Sunday",
probes/text-format-aaa-weekday-c A6, A9). The lexer (formatter/lexer.rs, the `'A' | 'a'` arm) keeps
joining any further such runs (`aaaAAAaaa` is one code), which is inferred, not probed. `aaam/pm`
is also unprobed. Probe in a new case: `TEXT(1,"aaaAAAaaa")`, `"AAAaaaAAA"`, `"aaaAAAa"`,
`"aaaAAAaaaa"`, `TEXT(0.75,"aaam/pm")`, `"aaaAM/PM"`, `"aaaa/p"`. See
docs/excel-semantics/number-formats.md "`aaa` is the weekday" (its Unprobed line).

## Acceptance criteria
- [ ] A probe case with the cells above has excel-graph values; the rule (or its limits) is in number-formats.md.
- [ ] Every cell of the new case passes, or the remainder is listed with evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
