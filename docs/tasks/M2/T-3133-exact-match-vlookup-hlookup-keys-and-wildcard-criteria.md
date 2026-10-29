---
id: T-3133
title: "Exact MATCH/VLOOKUP/HLOOKUP keys and wildcard criteria fold case one character to one, as Excel does (İ, final sigma, Kelvin sign, ẞ, titlecase digraphs)"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Exact-match lookups and wildcard criteria fold case beyond ASCII the way Excel does. Exact match covers MATCH(..,0),
VLOOKUP/HLOOKUP(..,FALSE) and the indexed exact search. Wildcard criteria cover COUNTIF/SUMIF/*IFS patterns with
`*`/`?` and wildcard MATCH. Excel's behaviour is backed by a committed oracle case. Today both fold with Rust's full
`str::to_lowercase`, which can change a text's length and depends on context. Excel's SEARCH and TEXTSPLIT do
neither.

## Context
- `folded_units` (crates/rustsheet-engine/src/functions/criteria.rs) lower-cases each run with `str::to_lowercase`.
  `WildcardPattern::new` folds the pattern the same way, and `ExactKey::of`
  (functions/lookup_and_reference/lookup_table.rs) folds every exact-match key through `folded_units`. On main
  97318e68a the engine gives:
  - `COUNTIF(A1,"?")` with A1 = `İ` (U+0130) is 0: the fold turns it into `i` + U+0307, two units. `COUNTIF(A1,"i*")`
    is 1, and `MATCH("i"&UNICHAR(775),A1:A1,0)` is 1.
  - `MATCH("οσ",A4:A4,0)` with A4 = `ΟΣ` is `#N/A`, but `MATCH("ος",A4:A4,0)` is 1. Rust lower-cases a word-final
    capital sigma to `ς`.
  - `MATCH("k",..,0)` matches the Kelvin sign U+212A. `COUNTIF(..,"ß*")` matches `ẞ`. `VLOOKUP("ǆ",..,FALSE)` finds
    `ǅ`.
- None of these has oracle evidence. Excel's other case-insensitive matchers each fold one character to one, but
  not all in the same way:
  - SEARCH (text.md "SEARCH lowercases per character", `fold_case` in functions/text/casefold.rs): `İ` stays
    itself and does not match `i`, `ς` matches `σ` and `Σ`, Kelvin matches `k`, `ǆ` matches `ǅ`, and `ſ`, `µ`, `ϐ`
    and `ﬀ` do not match.
  - TEXTSPLIT/TEXTBEFORE (`fold_match_char`): `İ` matches `i`, Kelvin does not match `k`, and `ẞ` does not match
    `ß`.
  So the lookup and criteria fold cannot be inferred from either. It needs its own probe.
- Exact equality in a criterion without wildcards goes through the collation (`cmp_text`, coercion.md
  "Collation equality in lookups and criteria"). It is not this task's fold. T-3114 (open PR #2287) probes that
  collation's case pairs for Latin Extended-A, Greek and Cyrillic, and leaves "the exact MATCH/VLOOKUP key beyond
  the confirming cells" out of scope. That key is this task.
- Why opus: the probe has to separate at least three candidate folds: Rust's full lowercase, SEARCH's
  per-character fold and TEXTSPLIT's fold. It has to do that for both consumers. `?` counts UTF-16 units
  (probes/criteria-lone-surrogates), so a length change shows up in the cell value. The fold then has to replace
  `folded_units` without breaking the probes that pass today: criteria-lone-surrogates, text-collation-lookups,
  criteria-wildcard-tilde and criteria-wildcard-newline. It must also keep `literal_units` and `ExactKey` agreeing,
  so that the indexed and linear exact searches give the same answer.
- Method: the `excel-probe` skill, excel-graph, one case `probes/t3133-lookup-criteria-case-fold` (milestone M2).
  Put the characters `İ ı I i`, `ΟΣ οσ ος`, U+212A `K k`, `ẞ ß`, `ǅ ǆ Ǆ`, U+212B `Å å`, U+2126 `Ω ω`, `ſ s`, `µ μ`
  and fullwidth `Ａ ａ` in cells. For each pair, test:
  - `MATCH(x,range,0)` and `VLOOKUP(x,..,FALSE)` both ways;
  - `COUNTIF` with `x&"*"`, `"?"`, `"??"` and `"*"&x`;
  - `XMATCH(x,..,2)`;
  - one `COUNTIFS` cell per pair, so that both criteria entry points are seen.

## Acceptance criteria
- [ ] `probes/t3133-lookup-criteria-case-fold` is committed with its excel-graph oracle sidecar. It covers the
      characters above in exact MATCH, VLOOKUP(..,FALSE), HLOOKUP(..,FALSE), wildcard COUNTIF/COUNTIFS (with `?`,
      `*` and a literal suffix) and wildcard MATCH/XMATCH, both ways round for each pair.
- [ ] Every cell of that case passes. The cells of probes/criteria-lone-surrogates, text-collation-lookups,
      criteria-wildcard-tilde, criteria-wildcard-newline and the gen-m2 suite still pass (`make eval-check`: 0
      regressions; `make eval-baseline` run).
- [ ] The fold the case fits is defined once and shared by `WildcardPattern`, `folded_units` and `ExactKey`. If it
      equals `fold_case` or `fold_match_char`, the existing function in functions/text/casefold.rs is reused
      rather than copied: `fold_case` is `pub(super)` and `fold_match_char` is private, so move the shared fold
      to one home that both functions/text and functions/criteria.rs import (no ad hoc re-export, no copy). A unit test shows that an indexed exact search (`ExactIndex`) and a linear one agree on
      `İ`, `ΟΣ` and the Kelvin sign.
- [ ] criteria.md "Wildcard escapes, case and trailing tilde" and lookup.md, at the exact-match key, record the
      probed fold as VERIFIED with the case id and cells. Characters the probe did not cover are named as
      UNVERIFIED.
- [ ] `make check` green; reviewers approve.

- [ ] If T-3114 (open PR #2287, collation) has merged first, `origin/main` is merged in and the fold sits beside
      its collation changes in functions/criteria.rs, criteria.md and lookup.md without undoing them.

## Out of scope
- The collation used by `=`, `<`, `>` and by plain-text criteria equality (T-3114) and approximate-match text
  order (T-3115).
- SEARCH, TEXTSPLIT, TEXTBEFORE and TEXTAFTER folds, which are already probed. Do not change them.
- XLOOKUP/XMATCH binary search modes (M6).
