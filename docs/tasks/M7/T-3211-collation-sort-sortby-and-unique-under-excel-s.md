---
id: T-3211
title: "Collation: SORT, SORTBY and UNIQUE under Excel's non-transitive pairwise accent rule, without a panicking sort (t3181 array cells)"
milestone: M7
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: [T-3181]
campaign: semantics
---

## Goal
SORT, SORTBY and UNIQUE order and deduplicate texts the way Excel's oracle shows, using Excel's
pairwise accent rule, and no sort can panic on its non-transitive order; or, where matching
Excel's output on an intransitive cycle is out of reach, a recorded DEVIATION (decision 0014,
directive 5: no panics) with accepted-deviation entries.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P2).
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
- [x] Every cell named above passes or is an accepted deviation with a decision-0014 entry;
      `make eval-check`: 0 regressions; `make eval-baseline` run.
- [x] A unit test sorts the Data strings of probes/t3201-collation-wrapped-run-pairs (554
      intransitive triples) with the new sort, in several input orders, without panicking.
- [x] coercion.md's "Ours (T-3181)" text says what the sorts do now.
- [x] `make check` green; reviewers approve.

## Out of scope
- Lookups (T-3210).

## Outcome
- Probe: probes/t3211-collation-sort-cycles (361 formula cells, 377 cells checked with Data; excel-graph 2026-10-31): every order of the two three-text
  cycles and 32 eight-text shuffles under SORT/SORTBY ascending and descending, UNIQUE of collation-equal and
  prefix pairs, and the 9x9 comparison matrix. Findings in coercion.md "Not a total order".
- `functions::util::sort_indices_pairwise`: a stable natural merge sort (runs, descending ones reversed, pairwise
  bottom-up merges) that ends on any comparator. SORT and SORTBY use it with `sort_key_cmp`, which now compares
  texts with `compare_values_pairwise`. It reproduces every three-text cycle order and 68 of 88 eight-text rows;
  no simple algorithm reproduced the rest (tried: see T-3418).
- UNIQUE keys texts by `CollationKey` (`CellKey::Collated`) and joins a line to the first kept line of its bucket
  that `cmp_text_pairwise` finds equal; rows and columns share one path.
- Eval delta: probes known failures 2302 -> 2335: fixed 10 (all t3181-collation-prefix-arrays cells: A1-A9, B2),
  new case t3211-collation-sort-cycles with 43 known failures owned by T-3418 (40 eight-text sort rows, UNIQUE
  G4:I4); 0 regressions.
- Unit test `the_pairwise_sort_ends_on_intransitive_texts` (test_collation.rs): the t3201 wrapped-run texts in five
  input orders, both directions.
- Follow-ups: T-3418 (fit the eight-text orders and UNIQUE's sort-order grouping), T-3419 (bound UNIQUE's pairwise bucket scan; reviewers).
