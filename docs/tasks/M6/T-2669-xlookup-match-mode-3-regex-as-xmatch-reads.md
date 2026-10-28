---
id: T-2669
title: "XLOOKUP match_mode 3 (regex) as XMATCH reads it"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
XLOOKUP accepts `match_mode` 3 (regular-expression match), as Excel does, instead of rejecting it as `#VALUE!`.

## Context
Found by T-2656. Excel reads a match_mode of 3, or anything that floors to 3, as the regex mode in XLOOKUP as well as
XMATCH: `XLOOKUP(2.5,{1,2,3},{10,20,30},"nf",3)` is "nf" (no match, so if_not_found), and so are 3.5 and 3-2^-51. In the
same probes the XMATCH twins give `#N/A`, and both reject an invalid pattern (`XMATCH("?",{"z","a"},3)` and
`XLOOKUP("?",...,3)` are `#VALUE!`).
Failing cells (probes, excel-graph, 2026-10-25): probes/t2656-xmatch-mode-near-integer G7 (mode 3), G21 (3-2^-51);
probes/t2656-xmatch-mode-rounding G15 (3.5); probes/t2656-xlookup-mode-snap-edge C8 (3 - 2^-22*(1+2^-11), which snaps to 3). H7 and H21 (`"?"`, `#VALUE!`) already pass for the wrong reason (ours
rejects the mode).
Code: `xlookup_modes` in crates/rustsheet-engine/src/functions/xlookup.rs has no regex arm. XMATCH's regex scan
(`MatchMode::Regex` in functions/lookup_and_reference/xmatch.rs) is the one to reuse, never copy. XLOOKUP maps its
modes onto `XmatchMode` for `linear_scan`, so it likely needs the regex arm there too. Probe the binary search_modes
with regex, and XLOOKUP's regex over numbers and booleans, before implementing either.

## Acceptance criteria
- [x] The four cells above pass. No regressions. The review probe's two PCRE2-only cells are new-coverage lines, owned by
      T-2680.
- [x] A probe pins XLOOKUP's regex mode for text, numbers and the binary search modes, recorded in lookup.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- XMATCH's regex mode (implemented).

## Outcome
- Two excel-graph probes (probes/t2669-xlookup-regex-mode, 60 cells, XLOOKUP beside XMATCH; -blanks, 14 cells;
  2026-10-25) pin the regex mode. It is a case-sensitive find anywhere in the text, with numbers, booleans and blanks
  read as text on both sides. An invalid pattern or a binary search_mode is `#VALUE!`, and an error candidate never
  matches. Recorded in lookup.md "XMATCH and XLOOKUP regex mode".
- XLOOKUP accepts match_mode 3 through two helpers now shared with XMATCH, `regex_pattern` and `regex_scan` in xmatch.rs.
  They read values with `calc_scalar_to_string`, which cast.rs now exports instead of keeping it private. XMATCH's own
  regex scan was rewritten onto them, which fixed two XMATCH bugs the probe found: a boolean read as "true" (Excel
  "TRUE"), and a number or boolean lookup value matching nothing (Excel uses its text as the pattern).
- Eval: the 4 T-2656 known failures pass (near-integer G7, G21; rounding G15; xlookup-mode-snap-edge C8). So do 3 cells
  of a regex-mode XMATCH with a number as lookup value, `XMATCH(COUNTA(..),{3;1;4;1;5},<a mode that reads 3>)`:
  gen-m9/eccd03df42e40006 Calc!A21 and probes/t2475-stat-subexpressions P!A545, P!A549. Of the 86 new cells, all pass
  except two PCRE2-only patterns (lookbehind, backreference: t2669-xlookup-regex-lift G1, G2). Ours compiles with the
  `regex` crate, as XMATCH always has, which refuses them. Moving both functions onto the REGEX* engine is follow-up
  T-2680. The third probe (-lift) was added at review.
- Unit test `regex_mode_reads_values_as_text_in_xmatch_and_xlookup`.
