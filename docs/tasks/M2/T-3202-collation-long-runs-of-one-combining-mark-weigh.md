---
id: T-3202
title: "Collation: long runs of one combining mark weigh as Excel does (t3154-collation-accent-weights A105:A112 and the ranks they shift)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3201]
---

## Goal
`cmp_text` weighs a letter followed by a long run of combining marks by the rule T-3201 states, so
the long-run strings of probes/t3154-collation-accent-weights rank where Excel ranks them, and the
strings ranked with them stop being off by one.

## Context
- Split out of T-3181 (planner 2026-10-30). Rule and evidence: coercion.md T-3154 section, the
  long-run rule T-3201 adds, and its case probes/t3201-collation-long-mark-runs.
- Cells owned (decision 0096; gap triage 2026-10-30, report run at 83cf65c2):
  - probes/t3154-collation-accent-weights Calc!A2, A3, A4, A5, A6, A7, A8, A9, A11, A18, A22, A36, A37, A40:A68, A76, A80, A105, A106, A107, A108, A109, A110, A111, A112
  - The cells of probes/t3201-collation-long-mark-runs that T-3201 leaves as known failures.
- The off-by-one ranks (Calc!A2:A68, A76, A80) come from the long-run strings landing below their
  Excel place: ours ranks 22 acutes (Calc!A107) 3rd of 119, Excel 43rd, so one extra string
  sits below every weight above 10. Calc!A94, A95 and A104 are the prefix rule's (T-3181).
- Code: the accent byte in `cmp_chars` and `CollationKey` (crates/rustsheet-engine/src/functions/
  util/collation.rs). The order must stay total: GROUPBY (`grouping.rs`), TEXTSPLIT and the sort
  helpers in `functions/util.rs` sort with `cmp_text`, and `slice::sort_by` may panic on a
  non-total order. `CollationKey` must stay equal exactly when `cmp_text` says Equal (XMATCH and
  XLOOKUP hash it, T-0448). One accent computation for both: do not write a second one.
- Tier opus: the rule is not known at filing, and it may need fitting to Excel's bytes.

## Acceptance criteria
- [ ] Every cell named above passes. If a rank cell also counts a prefix-rule pair and cannot pass
      before T-3181, the PR names the cell and the pair, and T-3181's cell list gains it.
- [ ] Every other cell of the t3154 probes, probes/t3114-collation-beyond-latin1,
      text-collation-* and the criteria and lookup collation probes keeps passing (`make
      eval-check`: 0 regressions; `make eval-baseline` run).
- [ ] A unit test in collation.rs checks that `cmp_text` is a total order (antisymmetric and
      transitive) over every Data string of the two t3154 weight/pair cases and of
      t3201-collation-long-mark-runs, and that `CollationKey` equality matches `Ordering::Equal`
      on them.
- [ ] coercion.md's "Ours" text for long runs says the engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The prefix rule (T-3181).
- Marks outside U+0300-U+0362 and scripts other than Latin, Greek and Cyrillic.
