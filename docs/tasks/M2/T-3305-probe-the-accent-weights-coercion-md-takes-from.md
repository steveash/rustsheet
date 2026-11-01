---
id: T-3305
title: "Probe the accent weights coercion.md takes from the Windows table alone: the unprobed combining marks of U+0300-U+0362 and the inferred decomposed letters of Latin Extended, Greek and Cyrillic"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every `MARK_WEIGHT` entry and a sample of the `DECOMPOSED` letters that no oracle cell checks today
are checked against Excel by a probe, and coercion.md marks them VERIFIED or records Excel's weight
where it differs.

## Context
- coercion.md "Accent weights add up and wrap" (T-3154): the marks probed are U+0300-U+0304,
  U+0306-U+030C, U+030F, U+0311, U+0314, U+0315, U+031B, U+0323-U+0328, U+032C, U+032D, U+0330,
  U+0331, U+0334, U+0342, U+0345 (probes/t3154-collation-accent-weights Data!A10:A39); "the other
  weights of U+0300-U+0362 come from the Windows table alone, UNVERIFIED".
- coercion.md "Letters beyond Latin-1 and combining marks": "Inferred, UNVERIFIED: the engine gives
  every other lower- or upper-case letter of Latin Extended-A and -B, Greek, Cyrillic and Latin
  Extended Additional that has a canonical decomposition (`ĉ ĝ ĥ ĵ ŝ ŵ ŷ ĩ ĭ ŭ ŏ ō ĕ ġ ċ ļ ŗ`,
  `ǐ ǒ ǔ ǖ ȁ ȩ`, `ϋ ΰ`, `ѐ ѓ ї ќ ѝ ў`, `ấ ặ ẽ ỹ`) the same base-plus-marks rule".
- Ours: `MARK_WEIGHT` and `DECOMPOSED` in
  crates/rustsheet-engine/src/functions/util/collation_tables.rs.
- No probe needs new design: copy the layout of probes/t3154-collation-accent-weights (Data
  strings built with UNICHAR, ranked by `SUMPRODUCT(--(range<x))`) for the marks, each as `"a"`
  plus the mark, ranked in a ladder that includes the probed marks as anchors; and the
  t3154-collation-accent-pairs layout (`(x<y)*1+(x=y)*2+(x>y)*4`) for each listed letter against
  its decomposition (`"ĉ"` vs `"c"&UNICHAR(770)`), against the base letter and against the base
  letter of the next code point.
- Leave U+0363-U+036F to T-3203 (item 1), and the multi-letter rule to T-3304.

## Acceptance criteria
- [ ] Probe case `probes/t3305-collation-table-weights` (milestone M2) committed with its
      excel-graph sidecar: one ranked text per unprobed mark of U+0300-U+0362 (U+034F included, as
      the "no weight" claim), and one pair row per letter listed in the Context.
- [ ] coercion.md drops both UNVERIFIED sentences quoted above, citing the case and cells, or
      states Excel's weight for each mark or letter that differs.
- [ ] Every cell passes, or the failing cells are known failures recorded by `make eval-baseline`
      (0 regressions) and an engine task is filed that lists them (case stem and A1 names in one
      bullet, decision 0159).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation_tables.rs (the follow-up task).
- U+0363-U+036F and marks after `æ`/`ß` (T-3203); several accented letters (T-3304); other scripts
  (T-3276); width and kana (T-3275).
