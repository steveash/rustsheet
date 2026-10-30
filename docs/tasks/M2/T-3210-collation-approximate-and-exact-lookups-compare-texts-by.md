---
id: T-3210
title: "Collation: approximate and exact lookups compare texts by Excel's pairwise accent rule (t3181 lookup cells)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3181]
---

## Goal
MATCH, VLOOKUP, HLOOKUP, XMATCH, XLOOKUP and SWITCH find texts the way Excel's oracle shows: by
the pairwise accent rule that the comparison operators and criteria use since T-3181, so the
lookup cells of the t3181 probes pass.

## Context
- Filed by T-3181. Evidence and rule: coercion.md "Not a total order" (the "Every caller uses it"
  part) and "Shared leading units are skipped before the sum". `cmp_text_pairwise` in
  crates/rustsheet-engine/src/functions/util/collation.rs is the comparison; reuse it, do not
  write another.
- Cells owned (decision 0096):
  - probes/t3181-collation-prefix-callers Calc!C3, C5, C6, C7, C10, C11, C12, C14
  - probes/t3181-collation-prefix-arrays Calc!B3, B4, B5, B7, B8, B11, B12
- Approximate lookups (MATCH 1/-1, VLOOKUP/HLOOKUP TRUE, XMATCH -1/1 and binary search) compare
  one candidate at a time and never sort, so they can use `cmp_text_pairwise` for two texts
  directly (find where they reach `compare_values`/`compare_searched`; a binary search with a
  non-transitive comparator cannot panic, it only picks some position, as Excel's does).
- Exact XMATCH and XLOOKUP hash `CollationKey` (lookup_table.rs, T-0448). Every pair
  `cmp_text_pairwise` finds equal shares a key, but not every pair with one key is equal (`"a"`
  and `"a"` plus 16 U+0302), so keep the key and test the candidates with `cmp_text_pairwise`, as
  T-3181 did for the criteria index (`for_each_match` in statistical/ifs_cache.rs). SWITCH goes
  through `compare_values` (logical/switch.rs).
- Calc!B11 (`XLOOKUP(Data!A1,Data!A2,"hit","miss")`) gives #VALUE! in ours: the return array is a
  scalar text. Check first whether that is the prefix rule or an XLOOKUP argument gap; if the
  latter, file it as its own task with that cell and leave this task the rest.
- MATCH(...,0) is not the collation's equality (coercion.md T-1590 section); its cells already pass.
- Do not change `cmp_text` or `compare_values` for sorts (T-3211).

## Acceptance criteria
- [ ] Every cell named above passes (or B11 moves to its own task); `make eval-check`: 0
      regressions; `make eval-baseline` run.
- [ ] A unit test shows an exact XMATCH over a range holding both `"a"` and `"a"` plus 16 U+0302
      finds only the equal one, through the hashed index.
- [ ] coercion.md's "Ours (T-3181)" text says what the lookups do now.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT, SORTBY, UNIQUE and GROUPBY (T-3211).
