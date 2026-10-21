---
id: T-1899
title: "REGEX functions: PCRE2 global iteration after an empty match"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-1900]
---

## Goal
REGEXREPLACE and REGEXEXTRACT iterate matches as PCRE2 does after an empty match.

## Context
Found by T-1626 and T-1688. After an empty match Excel first tries a non-empty match anchored at the same position, then advances one character: `REGEXREPLACE("aaa","a*?","b")` is `bbbbbbb` (t1626-regexreplace C34), `|b` on `abc` is `-a---c-`, `b|` is `-a--c-`, `(?=b)|b` is `a--c`, and REGEXEXTRACT's all-matches mode on `a*?` gives 7 matches (`,a,,a,,a,`); occurrence counts these matches (`a*?` occurrence 4 replaces the second `a`). Cells: probes/regex-pcre2-features D1-D12. An empty pattern over text with a surrogate pair is `#VALUE!` (D11, D13; D14 with `x*` is an undecoded error: add an ERROR.TYPE cell), probably because Excel iterates UTF-16 units. Design: SPEC-0013 section 4 (rollout step 4). `pcre::Pattern::find_iter` is one iterator for every pattern. After an empty match at `p` it runs one anchored "not empty at start" attempt on the backtracker (T-1900's `Program`, built lazily for a regular pattern too), then advances one UTF-16 unit. An advance into a surrogate pair is the `SplitSurrogate` error, `#VALUE!`. REGEXEXTRACT mode 1 and REGEXREPLACE (occurrence included) use it.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 11 cells, 1 numeric past half a unit in the 15th digit and 10 with a wrong type, error or text, blamed on REGEXREPLACE 9, REGEXEXTRACT 2, COUNTA 1, TEXTJOIN 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/regex-pcre2-features Sheet1!D1 | `=REGEXREPLACE("abc","\|b","-")` | "-a---c-" | "-a-b-c-" |
| probes/regex-pcre2-features Sheet1!D12 | `=REGEXREPLACE("abc","(?=b)\|b","-")` | "a--c" | #VALUE! |
| probes/regex-pcre2-features Sheet1!D13 | `=ERROR.TYPE(REGEXREPLACE("a"&UNICHAR(128512)&"b","","-"))` | 3 | #N/A |
| probes/regex-pcre2-features Sheet1!D2 | `=REGEXREPLACE("abc","b\|","-")` | "-a--c-" | "-a-c-" |
| probes/regex-pcre2-features Sheet1!D3 | `=REGEXREPLACE("aaa","a??","b")` | "bbbbbbb" | "bababab" |
| probes/regex-pcre2-features Sheet1!D4 | `=REGEXREPLACE("abab","(?:ab)*?","-")` | "-----" | "-a-b-a-b-" |
| probes/regex-pcre2-features Sheet1!D5 | `=COUNTA(REGEXEXTRACT("aaa","a*?",1))` | 7 | 4 |
| probes/regex-pcre2-features Sheet1!D6 | `=TEXTJOIN(",",FALSE,REGEXEXTRACT("aaa","a*?",1))` | ",a,,a,,a," | ",,," |
| probes/regex-pcre2-features Sheet1!D7 | `=REGEXREPLACE("aaa","a*?","b",2)` | "baa" | "abaa" |
| probes/regex-pcre2-features Sheet1!D9 | `=REGEXREPLACE("aaa","a*?","b",4)` | "aba" | "aaab" |
| probes/t1626-regexreplace Sheet1!C34 | `=REGEXREPLACE(A8,"a*?","b")` | "bbbbbbb" | "bababab" |

## Acceptance criteria
- [ ] Visible parity: the 11 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 11 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] t1626-regexreplace C34 and regex-pcre2-features D1-D6, D12, D13 pass, plus D14 once its ERROR.TYPE cell is added
  (D7-D10 also need T-1360's optional arguments); baseline shrinks.
- [ ] The retry for a regular pattern runs in the memoized mode (SPEC-0013 section 3). Its program is counter-free and
  guard-free, and its visited set persists across the searches of one iteration, with every position at or before a successful match's end invalidated (SPEC-0013 section 3). Unit tests: `a*?` on `aaa` gives 7 matches in memoized mode, and `|a(?:a*b)?` on `REPT("a",32767)` is linear. The differential test covers this
  path too.
- [ ] `(a|a)*b|` over a 32,767-character text returns the same value as before.
- [ ] `a*?` and `|b` iteration over 32,767 characters stays within the call budget, and each runs under 50 ms in the
  release bench, with before and after numbers.
- [ ] A pattern that cannot match empty iterates at today's cost.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pattern features (T-1900).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong REGEXREPLACE and REGEXEXTRACT text after an empty match), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
