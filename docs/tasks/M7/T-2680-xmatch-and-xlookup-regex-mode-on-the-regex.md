---
id: T-2680
title: "XMATCH and XLOOKUP regex mode on the REGEX* PCRE2 engine (lookaround, backreferences)"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
XMATCH and XLOOKUP's regex match_mode (3) accepts the PCRE2 syntax Excel accepts, such as lookbehind and backreferences,
by matching on the engine REGEXTEST, REGEXEXTRACT and REGEXREPLACE already use, instead of the `regex` crate.

## Context
Found by T-2669's review. XMATCH's regex mode has compiled its pattern with the `regex` crate since before T-2669, and T-2669
shares that with XLOOKUP (`regex_pattern` / `regex_scan` in functions/lookup_and_reference/xmatch.rs). That crate refuses
PCRE2-only syntax with `#VALUE!`, while Excel accepts it:
- probes/t2669-xlookup-regex-lift G1 `XMATCH("(?<=D)og",A1:A3,3)` is 2 in Excel, ours `#VALUE!`.
- G2 `XMATCH("(d)\1",{"ab";"dd"},3)` is 2 in Excel, ours `#VALUE!`.
The REGEX* functions run on `functions/text/regex_search.rs` (`Matcher`: automata for regular patterns, the `pcre`
backtracker otherwise, one work budget per call; SPEC-0013). Reuse it, never copy it. The design questions are:
- how the per-call work budget and `#ERROR!` on spent work (SPEC-0013) apply to a lookup that tests many candidates;
- whether Excel's lookup regex mode honours the REGEX* functions' case and other rules (probe first);
- UTF-16 spans (a lookup only needs is-match).
Size M, and SPEC-0013 may need a short addendum.

## Acceptance criteria
- [x] t2669-xlookup-regex-lift G1 and G2 pass; no baseline gains a line.
- [x] A probe covers PCRE2 features (lookahead, lookbehind, backreference, `\K`, a possessive quantifier) in XMATCH and
      XLOOKUP regex mode, and records the result in lookup.md "XMATCH and XLOOKUP regex mode".
- [x] `make check` green; reviewers approve.

## Out of scope
- The REGEX* functions themselves.

## Outcome
- probes/t2680-lookup-regex-pcre2 (excel-graph, 45 cells: XMATCH, XLOOKUP and XMATCH from the last) shows Excel's
  lookup regex mode is full PCRE2, the same as REGEXTEST: lookbehind and lookahead (both polarities), backreferences,
  `\K`, possessive and atomic groups, `(?i)`, POSIX classes and `\p{..}`.
- The lookups now match on the REGEX* engine. `LookupPattern` (functions/text/regex_functions.rs) wraps one
  `PatternCache` per search, so the compile budget, the held-memory limit and the search-work budget are a REGEX*
  call's, and `#ERROR!` comes once the work is spent (SPEC-0013). `regex_scan` and both `linear_search`s now return a
  `Result` for that error. Excel's lookup regex is case-sensitive like REGEXTEST's default, so no case argument is
  needed. Invalid patterns stay `#VALUE!` (checked when the pattern is built).
- The `regex` and `regex-lite` dependencies of rustsheet-engine are gone: XMATCH was their last user, and Cargo.lock
  drops regex-lite. The PCRE2 unit tests' regex-automata `meta` feature, which `regex` used to switch on through
  feature unification, is a dev-dependency feature now.
- Eval: t2669-xlookup-regex-lift G1 and G2 pass, and all 45 new cells pass. Unit test `regex_mode_reads_pcre2_syntax`.
- Design question from the task, the budget for a lookup that tests many candidates: one budget per search (one
  element of a lifted lookup), the same as one REGEX* call over many texts. No SPEC-0013 addendum was needed, as the
  spec already scopes the budget to a call.
