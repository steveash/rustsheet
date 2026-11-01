---
id: T-3203
title: "Probe the UNVERIFIED T-3154 collation choices: U+0363-U+036F as letters, a mark after an expansion, the variant pass before the skipped pass"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The three collation choices T-3154 made from the Windows sorting weight table alone are backed by
an oracle case, and coercion.md marks each VERIFIED or records Excel's actual rule.

## Context
- Split out of T-3181 (planner 2026-10-30). coercion.md "Accent weights add up and wrap" lists
  these engine choices as UNVERIFIED:
  1. U+0363-U+036F ("combining a" to "combining x") are the letters `a e i o u c d h m r t v x`
     with diacritic weight 3, not marks.
  2. A mark after an expansion (`æ`, `ß`) adds its weight to the expansion's second letter (the
     Windows table gives `ǣ`, `æ` plus macron, a row of its own).
  3. The superscript (variant) pass comes before the skipped-character pass.
- Method: the `excel-probe` skill, excel-graph, laid out like probes/t3154-collation-accent-pairs
  (Data strings, a Calc grid of `(x<y)*1+(x=y)*2+(x>y)*4` codes, `SUMPRODUCT(--(range<x))` ranks).
  Suggested strings: for 1, `"b"&UNICHAR(867)` against `"b"&"a"`, `"bá"` and `"b"&UNICHAR(769)`, for a
  few of U+0363-U+036F; for 2, `"æ"&UNICHAR(772)`, `"ǣ"`, `"ae"&UNICHAR(772)`, `"ā"&"e"`,
  `"ß"&UNICHAR(769)`, `"ss"&UNICHAR(769)`; for 3, pairs that differ in both a superscript (`ª`,
  `²`) and a skipped character (`-`, `'`), in both orders (`"ª-b"` against `"a-b"`, `"ªb"`,
  `"a'b"`).

## Acceptance criteria
- [ ] A probe case `probes/t3203-collation-unverified-choices` (milestone M2) is committed with its
      excel-graph oracle sidecar, covering the three choices.
- [ ] coercion.md marks each of the three choices VERIFIED with the case id and cells, or states
      Excel's rule and drops the UNVERIFIED wording.
- [ ] Every cell passes, or the failing cells are known failures in eval/baseline (`make
      eval-baseline`, 0 regressions) and a fix task is filed that names them (case stem and A1 names
      in one bullet, decision 0159).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (a follow-up task if the evidence requires it).
- Long mark runs (T-3201) and the prefix rule (T-3181).
