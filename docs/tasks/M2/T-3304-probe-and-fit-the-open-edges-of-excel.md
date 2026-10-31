---
id: T-3304
title: "Probe and fit the open edges of Excel's pairwise accent rule: marks on several letters, equal-sum units that share a prefix, and the accent pass against the case and skipped-character passes"
milestone: M2
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
An excel-graph probe pins how Excel's pairwise accent rule (coercion.md "Shared leading units are
skipped before the sum", T-3201) behaves on texts it was not verified for, and coercion.md states
the rule for them precisely enough to implement. Today the rule is "VERIFIED for one accented
letter, and for it followed by `"b"`", and the section ends with three open, UNVERIFIED edges.
`cmp_text_pairwise` serves `=`, `<`, `>` and every text criterion (COUNTIF, SUMIF, SUMIFS,
COUNTIFS), so these edges decide M2 criteria results on real accented text (names, places).

## Context
- coercion.md, end of "Shared leading units are skipped before the sum": "Open, UNVERIFIED: units
  shared across letters (`"áb"` against `"a"&UNICHAR(769)&"c"`), two units with equal sums that
  share a prefix, and marks after `æ`/`ß`". The last one is T-3203's item 2; leave it there.
- Ours: `cmp_text_pairwise` and `accent_units` in
  crates/rustsheet-engine/src/functions/util/collation.rs apply the rule per letter. What happens
  when **two or more letters carry accents** is an engine extrapolation: whether Excel decides on the
  first letter whose accent differs (per-letter, as ours), or compares one accent stream for the
  whole text (Windows' classic diacritic pass is string-wide), and whether the shared-unit skip can
  run across a letter boundary.
- Hypotheses to separate, each with cells (suggested strings; the worker designs the final set):
  1. Several accented letters: `"áé"` against `"aé"`, `"áe"`, `"a"&UNICHAR(769)&"e"&UNICHAR(769)`;
     a first letter that wins on accent against a second that loses (`"áe"` vs `"aé"`, both orders);
     wrapped sums on letter 1 with a difference on letter 2 (`"a"`&16 U+0302&`"é"` vs `"aé"`).
  2. Skip across a boundary: `"a"&UNICHAR(769)&"b"&UNICHAR(769)` against `"áb́"` and `"ab"&UNICHAR(769)`;
     the task's example `"áb"` vs `"a"&UNICHAR(769)&"c"` (expected to be decided by `b < c`; confirm).
  3. Equal sums sharing a prefix: `"a"&UNICHAR(769)&UNICHAR(768)` against
     `"a"&UNICHAR(769)&UNICHAR(771)&UNICHAR(...)` built to the same sum, and U+0341 / U+0301
     (equal weight) prefixes of different lengths.
  4. Order of passes on several letters: accent against case (`"Áb"` vs `"aB"`, `"áB"` vs `"Ab"`)
     and against a skipped character (`"á-b"` vs `"a-b"`, `"a-b"` vs `"áb"`).
- Method: the `excel-probe` skill, laid out like probes/t3201-collation-wrapped-run-pairs (Data
  strings built with UNICHAR, a Calc grid of `(x<y)*1+(x=y)*2+(x>y)*4` codes over every pair, and
  `SUMPRODUCT(--(range<x))` ranks), plus `COUNTIF(range,"="&x)` for a few rows so the criteria
  path is checked directly. Fit a model to the cells (as T-3201 did) before writing the rule.

## Acceptance criteria
- [ ] Probe case `probes/t3304-collation-multi-letter-accents` (milestone M2) committed with its
      excel-graph sidecar, holding cells for each of the four hypothesis groups above.
- [ ] coercion.md replaces the "Open, UNVERIFIED" sentence with the rule for several accented
      letters, the cross-letter skip and equal-sum prefixes, each citing case and cells; T-3203's
      `æ`/`ß` item is left to T-3203.
- [ ] Every cell passes, or the failing cells are known failures recorded by `make eval-baseline`
      (0 regressions) and an engine task is filed that lists them (case stem and A1 names in one
      bullet, decision 0159) with the fitted rule in its Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (the follow-up engine task).
- Sorts and lookups under the rule (T-3210, T-3211); marks after an expansion (T-3203); other
  scripts, width and kana (T-3275, T-3276); the Windows-table-only weights (T-3305).
