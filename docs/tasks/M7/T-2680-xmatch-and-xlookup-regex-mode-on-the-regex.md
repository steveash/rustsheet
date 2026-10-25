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
- The lookups now match on the REGEX* engine. `LookupRegexes` (functions/text/regex_functions.rs) holds one
  `PatternCache` per XMATCH or XLOOKUP call, created in `fn_xmatch` and `xlookup_impl` and shared through the lift by
  every element, so the call has one REGEX* call's compile budget, held-memory limit and search-work budget, and
  repeated patterns compile once (SPEC-0013). `LookupPattern` is one element's pattern on it. Once the work is spent,
  the search is `#ERROR!`, and so is the whole call (`finish`, as for REGEX*). `regex_scan` and both `linear_search`s
  now return a `Result` for that error. This is a DEVIATION, recorded in lookup.md and in text.md's REGEX* cap entry
  (decision 0014). Excel's lookup regex is case-sensitive like REGEXTEST's default, so no case argument is
  needed. Invalid patterns stay `#VALUE!` (checked when the pattern is built).
- The `regex` and `regex-lite` dependencies of rustsheet-engine are gone: XMATCH was their last user, and Cargo.lock
  drops regex-lite. The PCRE2 unit tests' regex-automata `meta` feature, which `regex` used to switch on through
  feature unification, is a dev-dependency feature now.
- Eval: t2669-xlookup-regex-lift G1 and G2 pass, and all 45 new cells pass. Unit test `regex_mode_reads_pcre2_syntax`.
- Design question from the task, the budget for a lookup that tests many candidates: one budget per call, shared by
  all its lifted elements, exactly as SPEC-0013 scopes it for a REGEX* call. So no addendum is needed. The first
  version gave each element its own budget; the Excel-fidelity review caught that, since N elements could then spend
  N seconds.
- Unit tests `regex_mode_reads_pcre2_syntax` and `lookup_regexes_share_the_call_budget` (the `#ERROR!` and the
  invalid-pattern `#VALUE!`).
