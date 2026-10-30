---
id: T-3211
title: "Collation: SORT, SORTBY and UNIQUE under Excel's non-transitive pairwise accent rule, without a panicking sort (t3181 array cells)"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3181]
---

## Goal
SORT, SORTBY and UNIQUE order and deduplicate texts the way Excel's oracle shows, using Excel's
pairwise accent rule, and no sort can panic on its non-transitive order; or, where matching
Excel's output on an intransitive cycle is out of reach, a recorded DEVIATION (decision 0014,
directive 5: no panics) with accepted-deviation entries.

## Context
- Filed by T-3181. Evidence: coercion.md "Not a total order", the "Every caller uses it" part:
  SORT of `"a"&UNICHAR(834)` and `"a"&UNICHAR(834)&UNICHAR(834)` puts the shorter first in either
  input order, so Excel sorts with the pairwise comparison (`cmp_text_pairwise`), not the
  wrapped-sum total order (`cmp_text`) the engine sorts with.
- Cells owned (decision 0096):
  - probes/t3181-collation-prefix-arrays Calc!A1, A2, A3, A4, A5, A6, A7, A8, A9, B2
- `slice::sort_by` may panic on a non-total order since Rust 1.81, so a sort that uses the
  pairwise comparison needs an algorithm that cannot panic (for example a hand-written stable
  merge or insertion sort over indices, bounded allocation). The SORT callers are in
  functions/spill_functions.rs and lookup_and_reference/grouping.rs (GROUPBY, PIVOTBY);
  TEXTSPLIT and the sort helpers in functions/util.rs also reach `cmp_text`.
- On a three-text cycle (Calc!A2, A8) Excel's output depends on its algorithm. Probe more cycles
  (excel-probe skill: three- and four-text cycles in every input order, ascending and descending,
  SORT against SORTBY) before choosing an algorithm; if no simple algorithm reproduces them, match
  every non-cycle input and record the cycle outputs as a DEVIATION.
- UNIQUE (Calc!B2): two U+0342 and U+0327 are one row in Excel but two in ours, which may be
  UNIQUE not using collation equality at all: check before tying it to the prefix rule.
- `CollationKey` (XMATCH, XLOOKUP, criteria indexes) must keep meaning "equal wrapped sums".
- Tier opus: algorithm choice from oracle evidence, panic safety.

## Acceptance criteria
- [ ] Every cell named above passes or is an accepted deviation with a decision-0014 entry;
      `make eval-check`: 0 regressions; `make eval-baseline` run.
- [ ] A unit test sorts the Data strings of probes/t3201-collation-wrapped-run-pairs (554
      intransitive triples) with the new sort, in several input orders, without panicking.
- [ ] coercion.md's "Ours (T-3181)" text says what the sorts do now.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lookups (T-3210).
