---
id: T-3202
title: "Collation: long runs of one combining mark weigh as Excel does (t3154-collation-accent-weights A105:A112 and the ranks they shift)"
milestone: M2
lane: engine
status: done
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
  - probes/t3201-collation-long-mark-runs Calc!B22, B72, B83, B86, B89, B92, B95, B98, B101, B104, B107, B110, B113, B116, B119, B163, B164, B173, B174, B179, B180, B202, B242, C72, C173, C174, D22, D51, D62, D72, D83, D86, D89, D92, D95, D98, D101, D104, D107, D110, D113, D116, D119, D122, D124, D126, D128, D130, D131, D133, D135, D137, D139, D140, D142, D144, D146, D148, D150, D151, D153, D155, D157, D159, D160, D202, D242
  - probes/t3201-collation-wrapped-run-pairs Calc!B13, B14, B15, I17, J17, K17, K27, L14, L18, L23, M21, M30, N1, N24, N32, O1, O11, O18, O23, O31, O35, P1, R8, R9, R10, S11, S14, S23, S35, V12, X11, X14, X18, Y13, AB10, AE12, AF14, AF35, AG13, AJ14, AJ18, AJ31
- T-3201 found the rule (coercion.md "Shared leading units are skipped before the sum"): there
  is no separate long-run rule. Per letter, drop the leading units (letter, then marks) both texts
  share with equal weights; a side with nothing left is less; otherwise compare the sums left mod
  256. It is the same mechanism as T-3181's prefix rule and is not transitive (554 intransitive
  triples in t3201-collation-wrapped-run-pairs), so it cannot replace `cmp_text`'s total order in
  sorts: settle it together with T-3181's choice (pairwise only, or a DEVIATION). The rule
  reproduces all 3,391 cells of the four t3154/t3201 accent cases (checked in T-3201's PR).
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
- [x] Every cell named above passes. If a rank cell also counts a prefix-rule pair and cannot pass
      before T-3181, the PR names the cell and the pair, and T-3181's cell list gains it.
- [x] Every other cell of the t3154 probes, probes/t3114-collation-beyond-latin1,
      text-collation-* and the criteria and lookup collation probes keeps passing (`make
      eval-check`: 0 regressions; `make eval-baseline` run).
- [x] A unit test in collation.rs checks that `cmp_text` is a total order (antisymmetric and
      transitive) over every Data string of the two t3154 weight/pair cases and of
      t3201-collation-long-mark-runs, and that `CollationKey` equality matches `Ordering::Equal`
      on them.
- [x] coercion.md's "Ours" text for long runs says the engine matches.
- [x] `make check` green; reviewers approve.

## Out of scope
- The prefix rule (T-3181).
- Marks outside U+0300-U+0362 and scripts other than Latin, Greek and Cyrillic.

## Outcome
- Delivered by T-3181's PR, which implemented the shared-units rule for the operators and
  criteria (`cmp_text_pairwise`), the only callers these cells use: every cell named above passes,
  and the unit test `cmp_text_is_a_total_order_on_the_accent_probes` checks `cmp_text`'s total
  order and `CollationKey` equality over the Data strings of all four accent cases. Sorts keep the
  total order; where Excel's sorts differ is T-3211.
