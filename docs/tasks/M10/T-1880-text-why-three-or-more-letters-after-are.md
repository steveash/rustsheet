---
id: T-1880
title: "TEXT: why three or more letters after a quote-taking _ are #VALUE! (follow-up of T-1486)"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Find out why Excel rejects `@_"aaa` while it accepts `@_"a` and `@_"aa`, and make TEXT match.

## Context
`_` and `*` take the next `"` as their character, so the letters that follow are unquoted. Excel
gives `TEXT("x","@_""a")` "x a" and `"@_""aa"` "x aa", but `"@_""aaa"` is `#VALUE!`, and so are
runs of 20, 40 and 80 `a` (probes/text-format-length-limit-n A1, A6 pass; n A2, A7:A9 are known
failures in eval/baseline/probes.txt; excel-graph, T-1486). We print the letters ("x aaa").
Hypothesis: `aaa` reads as a weekday code (`ddd`'s alias), so T-1656's rule rejecting `@` with a
date token in the same section applies (formatter/complexity.rs, `conflicts_with_text`). Probe to
tell this apart from a letter-run limit: `TEXT("x","@aaa")`, `TEXT(1,"aaa")`, `TEXT(1,"aaaa")`,
`TEXT(7,"0_""aaa")`, `TEXT("x","@_""aab")`, `TEXT("x","@_""bbb")` and `TEXT("x","@""a""aa")`.
Semantics: docs/excel-semantics/number-formats.md, TEXT format code complexity limit,
"Not explained or not implemented".

## Acceptance criteria
- [x] A probe round pins what makes the run fail; the rule and its evidence are in number-formats.md.
- [x] n A2 and A7:A9 leave eval/baseline/probes.txt, or the remainder is listed with evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fill characters printing a space (T-1735), conditions (T-0936).

## Outcome
`aaa` is Excel's weekday code, not a letter-run limit: three or more of one `a` (all lowercase or
all uppercase) print the short (`aaa`) or full (`aaaa`+) weekday, localized like `ddd`/`dddd`, so
`@` beside it is `#VALUE!` by T-1656's rule. Mixed case and one or two letters stay literal; an
adjacent other-case run of three or more joins the code. Three probe rounds
(probes/text-format-aaa-weekday, -b, -c; 90 cells, excel-graph) pin it; the lexer now
reads such runs as `DayNameShort`/`DayName`. Rule in docs/excel-semantics/number-formats.md,
"`aaa` is the weekday". Eval: probes -4 known failures (text-format-length-limit-n A2, A7:A9), and
the 90 new probe cells pass.
