---
id: T-2235
title: "Probe and match the unprobed criteria edges: D-function ~~, array criteria over blank, empty-text and 0 cells, single-cell AVERAGEIF/COUNTIFS/MAXIFS ranges"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The criteria functions (COUNTIF/SUMIF and their *IFS forms, AVERAGEIF(S), MAXIFS/MINIFS and the
D-functions) give Excel's results on three edge groups that docs/excel-semantics/criteria.md marks
unprobed. Each group gets oracle evidence, and the engine changes wherever Excel differs.

## Context
Gap triage 2026-10-21. No criteria cell fails in `make eval`, and no open task covers these
groups. Only the SPEC-0014 array-of-references cells fail, and T-2064 owns them. These are M2
functions with high priors, and the engine's behavior in each group below is inferred, not
probed:
1. **Tildes in D-function criteria.** criteria.md "Wildcard escapes, case and trailing tilde"
   says D-functions are unprobed for `~~`. D-functions match plain text by prefix ("D-function
   plain-text equality matches by prefix"; `CriteriaFlavor::Database` in `functions/criteria.rs`).
   So it is open whether `~` before a non-wildcard, `~~` and a trailing `~` stay literal in a
   prefix match, as they do in COUNTIF's plain-text rule, or read as in wildcard patterns
   ("Tildes inside wildcard patterns"). Probe DSUM/DCOUNTA criterion cells `a~c`, `a~~c`, `abc~`,
   `~`, `~~`, `=a~c` and `a~*` over the probes/criteria-wildcard-tilde data (a~c, ac, abc~, abc, ~,
   a~~c, a~b, a~bx, weights 1..128).
2. **Array criteria over blank, `""` and 0 cells in the searched range.** criteria.md "*IF(S)
   lift array criteria", dynamic edge cases (probes/ifs-array-criteria-edges): "unprobed: blank,
   `""` or 0 cells in the searched range". Probe COUNTIF, SUMIF, SUMIFS, AVERAGEIFS and MAXIFS with
   `{"","=","<>",0,"0","=0"}` criteria over a range holding a blank cell, a formula `=""`, a typed
   0 and the text "0". Compare each with the scalar criterion (criteria.md '"=" matches blank
   cells, not an empty-string cell'). Read the shape with ROWS/COLUMNS and the elements with INDEX.
3. **Single-cell ranges in AVERAGEIF, COUNTIFS and MAXIFS.** criteria.md "SUMIF/AVERAGEIF resize
   clamps at the sheet edge; single-cell ranges are 1x1": these share code paths with the probed
   functions, but they are not probed themselves. Probe `AVERAGEIF(A1,">0",D1)`,
   `AVERAGEIF(A1:B2,">0",XFD1048576)`, `COUNTIFS(A1,">0",B1,"<5")`, `MAXIFS(D1,A1,">0")` and
   `MINIFS(D1:D1,A1,">0")` in the style of probes/sumif-edge-single-cell F1-F15.

Write one excel-probe case (excel-graph oracle), about 60 cells. Code: `functions/criteria.rs`
(`wildcard_text_matcher`, `WildcardPattern::new`, the `Database` flavor),
`functions/statistical/if_ifs.rs` (`apply_ifs`, `resize_to`, `fn_countifs`) and
`functions/database.rs`.

## Acceptance criteria
- [ ] A new probes case, oracle-filled (excel-graph), covers the three groups. Every cell passes,
      or its known failure is recorded with `make eval-baseline` and has a filed task named in the
      Outcome.
- [ ] criteria.md replaces the three "unprobed" notes with rules that cite their cells. Where
      Excel looks inconsistent, it is recorded as found, not tidied.
- [ ] Any engine change has unit tests next to the existing criteria tests; no gated baseline gains
      a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A `NA()` or non-range first argument with an array criterion. excel-graph refuses those
  workbooks (criteria.md); they are desktop-only (M14).
- An array of references as the range (T-2064).
- Conditional-format value operators (T-1051).
