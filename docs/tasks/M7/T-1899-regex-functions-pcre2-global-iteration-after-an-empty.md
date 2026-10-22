---
id: T-1899
title: "REGEX functions: PCRE2 global iteration after an empty match"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-1900, T-2355]
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

## Handoff (resumed)
Opus run of 2026-10-22: no code yet. This run landed T-2355 (#1530), which this task needed, then
stopped at its session budget. Notes for the next run, from reading the code on main after #1530:
- Memoized mode exists: `pcre::program::compile_memoized(ast, limit)` builds a guard-free
  program for a regular AST, and `Backtracker` (memoized when `Program::memoized`) keeps a visited
  set across one operation, forgetting positions up to a match's end in `find_from`.
- What is missing:
  - an anchored "not empty at start" attempt on `Backtracker`: one `attempt` at `p` whose `Match`
    is refused when `pos == p` (PCRE2's `NOTEMPTY_ATSTART | ANCHORED`). Under memoization a refused
    `Match` at `(pc, p)` is safe to keep marked, since later attempts start after `p`
    (SPEC-0013 section 3);
  - `Matcher::Automata` has no AST or program, so the retry needs the regular AST kept (or
    re-parsed) and a lazily compiled memoized `Program` plus `BacktrackCache` in the matcher,
    charged to the compile budget and counted in `memory_usage`;
  - one iterator replacing both `regex_search::each_match` and `EngineSearch::advance`'s
    `Searcher` rule:
    - after a non-empty match, search from its end;
    - after an empty match at `p`, run the anchored not-empty attempt at `p`, then advance one
      UTF-16 unit;
    - advancing into a surrogate pair is `SplitSurrogate`, which maps to `#VALUE!`.
  - `captures_at` replays the iteration, so it must use the same iterator.
- Tests to pin: the 11 cells in Context, `a*?` on `aaa` with 7 matches in memoized mode, and
  `|a(?:a*b)?` on `REPT("a",32767)` linear (the forget-after-`e` rule).

## Outcome
`regex_search::each_match` is now the one iterator of every REGEX* pattern, by PCRE2's global rule: after an
empty match at `p`, one attempt anchored at `p` that refuses the empty match (`Backtracker::retry`), then a search
one UTF-16 unit on, which inside a surrogate pair is `SearchError::SplitSurrogate` (`#VALUE!`). A regular pattern
keeps its AST (`Form::Automata { hir, retry }`) and compiles its memoized retry program the first time an
iteration needs it, within the call's compile budget (`within_size_limits`, shared with the pattern compile); the
visited set persists across the retries of one iteration. `captures_at` replays the iteration, and an n-th
occurrence stops at its match (new probe case probes/regex-iteration-surrogates A1-A11, excel-graph: a split after
the n-th match is no error, A4, A6, A8). The differential test now compares PCRE2 iteration with groups on the
automata (with retries) against the memoized mode.

Eval: probes known failures 2047 -> 2037 (regex-pcre2-features D1-D7, D9, D13, t1626-regexreplace C34), plus 11
new passing cells; no regressions. D12 needs lookahead (added to T-2354's criteria); D14's question is answered by
the new case's A1 (`x*` is ERROR.TYPE 3). Release timings of `LEN(REGEXREPLACE(REPT("a",32767),p,"-"))`, before ->
after: `a*?` 3.4 -> 4.3 ms, `|b` 3.2 -> 3.7 ms, `a` (cannot match empty) 2.0 -> 2.1 ms (noise), `(a|a)*b|` 427 ->
431 ms, `#ERROR!` both (its automata searches already run out of work).
