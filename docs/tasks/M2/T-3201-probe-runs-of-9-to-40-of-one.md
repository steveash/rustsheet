---
id: T-3201
title: "Probe runs of 9 to 40 of one combining mark and state the rule their accent weight follows (t3154 long runs)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An oracle case pins how Excel weighs a letter followed by a long run of one combining mark, and
coercion.md states the rule precisely enough to implement (T-3202): for which run lengths the
accent is the wrapped byte sum T-3154 found, and what it is beyond them.

## Context
- Split out of T-3181 (planner 2026-10-30), which keeps only the prefix rule.
- T-3154 (coercion.md "Accent weights add up and wrap") found the accent is the letter's diacritic
  weight plus each mark's, summed in a wrapping byte: two U+0342 (2 + 282 = 284, so 28) equal one
  cedilla, and three U+0323 (260, so 4) sort between `"a"` and `"á"`. Long runs of one mark do not
  follow that sum: `"a"&REPT(UNICHAR(769),22)` (2 + 264, so 10 by the sum) ranks 43 of 119 in
  Excel, near the weights around 40, and 3 in ours (probes/t3154-collation-accent-weights Calc!A107).
  Also off: 20, 21, 40 and 100 acutes, 10 and 11 rings, 10 dots below (Calc!A105:A112). Because those
  strings land in the wrong place, the ranks of Calc!A2:A68, A76 and A80 are off by one or two.
- The current probe has too few run lengths to fit a rule: it jumps from 8 to 20 acutes.
- Method: the `excel-probe` skill, excel-graph, laid out like probes/t3154-collation-accent-pairs
  (a Data column of strings, a Calc grid of `(x<y)*1+(x=y)*2+(x>y)*4` codes) plus
  `SUMPRODUCT(--(range<x))` ranks against a ladder of references with known weights from T-3154
  (`a` 2, `á` 14, `à` 15, `â` 18, `ä` 19, `ǎ` 20, `ă` 21, `ā` 23, `ã` 25, `å` 26, `ą` 27, `ạ` 88,
  and single marks up to U+0342's 141 + 2). Suggested strings: `"a"&REPT(UNICHAR(m),n)` for
  n = 1..40 with m = 769 (acute, 12), 778 (ring, 24), 803 (dot below, 86) and 834 (U+0342, 141),
  each compared with its n-1 neighbor and ranked against the ladder; plus two-mark mixes at the
  boundary (for example 20 acutes then one grave, and one grave then 20 acutes) to tell a count rule
  from a sum rule, and the same run after `e` and inside a two-letter text (`"a"&run&"b"`) to see
  whether the rule is per letter or per text. Keep the case under about 1,500 formula cells.
- Code to read for the hypothesis, not to change here: `cmp_chars` and the accent byte in
  crates/rustsheet-engine/src/functions/util/collation.rs, `MARK_WEIGHT` in collation_tables.rs.

## Acceptance criteria
- [ ] A probe case `probes/t3201-collation-long-mark-runs` (milestone M2) is committed with its
      excel-graph oracle sidecar, covering run lengths 1 to 40 of at least the four marks above.
- [ ] coercion.md's T-3154 section states the long-run rule (the run length or sum where it starts,
      and what the accent is beyond it) with the case id and cells, Status VERIFIED, and checks it
      against probes/t3154-collation-accent-weights Calc!A105:A112 (each of those eight Excel ranks
      is explained by the rule, shown in the PR).
- [ ] Cells our engine fails are known failures in eval/baseline (`make eval-baseline`, 0
      regressions), and T-3202's Context names them (case stem and A1 names in one bullet,
      decision 0159), so they keep an owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (T-3202).
- The prefix rule (T-3181) and the other unverified T-3154 choices (T-3203).
