---
id: T-1899
title: "REGEX functions: PCRE2 global iteration after an empty match"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1898]
---

## Goal
REGEXREPLACE and REGEXEXTRACT iterate matches as PCRE2 does after an empty match.

## Context
Found by T-1626 and T-1688. After an empty match Excel first tries a non-empty match anchored at the same position, then advances one character: `REGEXREPLACE("aaa","a*?","b")` is `bbbbbbb` (t1626-regexreplace C34), `|b` on `abc` is `-a---c-`, `b|` is `-a--c-`, `(?=b)|b` is `a--c`, and REGEXEXTRACT's all-matches mode on `a*?` gives 7 matches (`,a,,a,,a,`); occurrence counts these matches (`a*?` occurrence 4 replaces the second `a`). Cells: probes/regex-pcre2-features D1-D12. An empty pattern over text with a surrogate pair is `#VALUE!` (D11, D13; D14 with `x*`), probably because Excel iterates UTF-16 units. Follows the matcher chosen by T-1898.

## Acceptance criteria
- [ ] t1626-regexreplace C34 and regex-pcre2-features D1-D6, D12, D13 pass (D7-D10 also need T-1360's optional arguments); baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pattern features (T-1900).
