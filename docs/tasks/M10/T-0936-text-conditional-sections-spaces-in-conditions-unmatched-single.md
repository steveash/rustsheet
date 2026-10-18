---
id: T-0936
title: "TEXT conditional sections: [<>], spaces in conditions, unmatched single condition (15 known failures)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Number formats with a `[cond]` section choose the section the way Excel does.

## Context
Evidence: `poi/FormatChoiceTests.xlsx` (excel-graph, T-0019), formula `=TEXT(Cn, Bn)`. The
condition lexer and section choice are in `crates/rustsheet-engine/src/formatter/`
(`Token::Condition` in `lexer.rs`).

1. **A single conditional section that does not match prints the plain number** (7 cells, TextMismatch):
   A3 `[<10]#" Wow"` on 10 expected "10" (actual "10 Wow"); A4 same on 11 expected "11"; A5 `[>10]#" Big"`
   on 10 expected "10"; A10 `[<=10]#" Wow"` on 11 expected "11"; A14 `[=10]#" Wow"` on 1 expected "1";
   A16 same on 11 expected "11"; A22 `[<10]#" Wow"` on 10 expected "10". The evidence cannot tell whether the
   fallback is General or the section without its literals (both give these values). Pick the
   simplest rule that fits and say in the semantics entry that the evidence does not decide it.
2. **`[<>10]` is rejected** (3 cells, #VALUE!): A17 on 1 expected "1 Wow", A18 on 10 expected
   "10", A19 on 11 expected "11 Wow".
3. **Spaces inside the condition are rejected** (4 cells, #VALUE!): A23 `[< 10]#" Wow"` on 1 expected
   "1 Wow", A24 on 10 expected "10", A25 `[<      10]#" Wow"` on 1 expected "1 Wow", A26 on 10 expected "10".
4. **The same unmatched single condition in the probes suite** (1 cell, TextMismatch, T-1486):
   probes/text-format-length-limit-q A16, `TEXT(1,"d"&<119 quoted a>&"[>5]d")`, expected "1" (we print
   the whole section). It fits rule 1 with General as the fallback.

## Acceptance criteria
- [ ] The 14 lines (A3, A4, A5, A10, A14, A16, A17, A18, A19, A22, A23, A24, A25, A26)
      disappear from eval/baseline/poi.txt, and probes/text-format-length-limit-q A16 from
      eval/baseline/probes.txt, with no new F lines.
- [ ] Rules recorded in `docs/excel-semantics/number-formats.md` citing these cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
FormatConditionTests (excel-cached) unless it is fixed by the same change. Empty formats (T-0933).
