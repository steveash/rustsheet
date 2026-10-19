---
id: T-1899
title: "REGEX functions: PCRE2 global iteration after an empty match"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1900]
---

## Goal
REGEXREPLACE and REGEXEXTRACT iterate matches as PCRE2 does after an empty match.

## Context
Found by T-1626 and T-1688. After an empty match Excel first tries a non-empty match anchored at the same position, then advances one character: `REGEXREPLACE("aaa","a*?","b")` is `bbbbbbb` (t1626-regexreplace C34), `|b` on `abc` is `-a---c-`, `b|` is `-a--c-`, `(?=b)|b` is `a--c`, and REGEXEXTRACT's all-matches mode on `a*?` gives 7 matches (`,a,,a,,a,`); occurrence counts these matches (`a*?` occurrence 4 replaces the second `a`). Cells: probes/regex-pcre2-features D1-D12. An empty pattern over text with a surrogate pair is `#VALUE!` (D11, D13; D14 with `x*` is an undecoded error: add an ERROR.TYPE cell), probably because Excel iterates UTF-16 units. Design: SPEC-0013 section 4 (rollout step 4). `pcre::Pattern::find_iter` is one iterator for every pattern. After an empty match at `p` it runs one anchored "not empty at start" attempt on the backtracker (T-1900's `Program`, built lazily for a regular pattern too), then advances one UTF-16 unit. An advance into a surrogate pair is the `SplitSurrogate` error, `#VALUE!`. REGEXEXTRACT mode 1 and REGEXREPLACE (occurrence included) use it.

## Acceptance criteria
- [ ] t1626-regexreplace C34 and regex-pcre2-features D1-D6, D12, D13 pass, plus D14 once its ERROR.TYPE cell is added
  (D7-D10 also need T-1360's optional arguments); baseline shrinks.
- [ ] The retry for a regular pattern runs in the memoized mode (SPEC-0013 section 3). Its program is counter-free and
  guard-free, and its visited set persists across the retries of one iteration. The differential test covers this
  path too.
- [ ] `(a|a)*b|` over a 32,767-character text returns the same value as before.
- [ ] `a*?` and `|b` iteration over 32,767 characters stays within the call budget, and each runs under 50 ms in the
  release bench, with before and after numbers.
- [ ] A pattern that cannot match empty iterates at today's cost.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pattern features (T-1900).
