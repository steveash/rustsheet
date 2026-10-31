---
id: T-3465
title: "Collation hold-out sweep: random mixed-category text pairs compared in Excel (<, =) to measure whether the fitted cmp_text_excel rules generalize past their probes"
milestone: M2
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3324, T-3329]
---

## Goal
A committed, excel-graph-filled probe of randomly drawn text pairs, built from every character category the
engine's collation models, answers one question with a number: on pairs nobody picked, how often does
`cmp_text_excel` order two texts as Excel does? Every disagreement is owned by a task, and the rate is recorded in
coercion.md "Text comparison uses Excel's collation, not code-point order".

## Context
- Excel's text order (`=`, `<`, the criteria, MATCH/XMATCH/XLOOKUP, SORT) is reproduced by a fitted model:
  crates/rustsheet-engine/src/functions/util/collation.rs (`cmp_text_excel`, `cmp_text_pairwise`, `CollationKey`,
  `accent_units`), collation_tables.rs and collation/kana.rs. Its rules came from 18 probe cases, each aimed at one
  rule (`ls eval/suites/probes/cases | grep collation`: t3114, t3115, t3154 x3, t3181 x2, t3201 x2, t3275 x2, t3276,
  t3304, text-collation-*), and several are marked as single-probe fits (coercion.md "Fitted in `cmp_text_excel`
  (T-1595...) with these choices where the probe is single-character only (each UNVERIFIED)"; T-3329's "Only U+0301
  was probed, so keep the choice minimal"). Each probe passing says the rule fits its cells; nothing measures the
  rules together on texts that mix them: a skipped hyphen inside a kana run, an expansion (`æ`, `ß`, `þ`, `œ`, `ĳ`)
  next to a combining mark, full-width letters against accented ones, a Thai prefix vowel after a Latin letter.
  Decision 0188 R4 asks the same of numeric fits (a hold-out the fit never saw); this is that check for the
  largest fitted text rule in M2.
- Depends on T-3324 (other scripts) and T-3329 (marks after a hyphen), so the sweep measures the model they
  complete rather than reporting their known cells again. If either is still open when this starts, draw only
  from the categories already implemented and say so.
- Design (decide and record in the case description and the task Outcome):
  - Categories: ASCII letters and digits, case pairs, space/NBSP/tab/LF, ASCII punctuation, the skipped characters
    (hyphen, apostrophe, en and em dash, control characters), Latin-1 symbols and digit-like symbols
    (`¹ ² ³ ¼ ½ ¾`), precomposed accented Latin letters, a letter plus combining marks from U+0300-U+0362, the
    expansions, Greek, Cyrillic, kana and the long-vowel and repeat marks, full-width and half-width forms, and the
    T-3276 scripts (Hebrew, Arabic, Devanagari, Thai, Lao, Hangul, CJK, astral).
  - Pairs: about 600, drawn with a fixed seed; texts of 1 to 6 characters from 1 to 3 categories; a third of the
    pairs made "near" (one edit apart: a substitution within a category, an inserted skipped character or mark), since
    random far pairs mostly differ at the first letter and test little.
  - Readers per pair: `=A<n><B<n>` and `=A<n>=B<n>`, both orders of `<`, so the sweep also shows any pair where
    Excel itself is not antisymmetric (coercion.md records non-transitive cycles; record any such pair, never fit it).
  - Keep the generator script out of the engine; commit it under the probe (or in tools/ if a reviewer asks), and
    keep the draw deterministic.
- Cells this sweep finds failing go to the open collation task whose rule covers them, or to a new engine task
  (milestone M2), case file stem and A1 names in one bullet (decision 0159), so `make exit-owners MILESTONE=M2`
  stays at 0. A failure that only says a single-probe choice was wrong is evidence for the existing UNVERIFIED
  entry: update that entry's status in the same PR.
- Not a duplicate of T-3309 (gen-m2 data and criteria pools of accented Latin texts, one category), T-3203,
  T-3305, T-3381 or T-3180 (each probes named unprobed points of one rule). This task measures the composition of
  all rules on random pairs, which none of them does; overlap of a few drawn pairs with their cells is harmless.

## Acceptance criteria
- [ ] Probe case `probes/t3465-collation-holdout` (milestone M2) committed with its excel-graph oracle sidecar,
      at least 500 pairs and their `<` and `=` readers, with the seed and the category of each text recorded.
- [ ] coercion.md "Text comparison uses Excel's collation, not code-point order" records the agreement rate (pairs
      the engine orders as Excel, out of all, before any fix) and lists each new rule or corrected UNVERIFIED choice
      with its cells.
- [ ] Every failing cell is a known failure in eval/baseline/probes.txt (`make eval-baseline`, 0 regressions) and
      is owned: `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the pairs that fail (the owning tasks).
- SORT, SORTBY and UNIQUE orders (T-3211) and lookup folds (T-3133, T-3298).
- Locales other than en-US.
