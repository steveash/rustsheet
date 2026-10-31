---
id: T-3417
title: "Probe combining marks after skipped characters beyond T-3304's one case: other and stacked marks, en and em dash, full-width forms, control characters, leading and trailing positions"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph probe shows how Excel weighs a combining mark that follows a skipped character in
the shapes T-3304 left open, and docs/excel-semantics/coercion.md states the rule for each shape
(VERIFIED with case and cells), so T-3329's minimal fit is either confirmed beyond U+0301 after `-`
and `'`, or the cells that contradict it are owned by an engine task.

## Context
- coercion.md, "Shared leading units are skipped before the sum", bullet "A mark after a hyphen or
  apostrophe goes with that skipped character" (T-3304, probes/t3304-collation-multi-letter-accents
  G4 rows 11 and 20, Rank!H11): only U+0301, only after `-` and `'`, only between two letters, was
  probed; the bullet says "how other marks or several marks weigh there is open".
- T-3329 (engine, M2) implements the fit "skipped character's weight plus the weights of the marks
  after it, position unchanged" and lists other marks, several marks, and marks after the en/em dash
  or control characters as out of scope, "a probe task if needed". This is that task.
- The skipped characters (collation.rs module docs, pass 5): the controls other than CHAR(9) to
  CHAR(13), `'`, `-`, en dash U+2013, em dash U+2014, full-width apostrophe U+FF07 and hyphen-minus
  U+FF0D.
- Questions, each answered by at least two ranked cells, with `"a-b"`, `"a'b"`, `"a-"&UNICHAR(769)&"b"`
  (T-3304's probed points) and `"áb"` as anchors in every ladder:
  1. Other marks after `-`: U+0300, U+0302, U+0308, U+030C, U+0327 (do they order by their
     `MARK_WEIGHT` among themselves as on a letter, and all below `"áb"`?).
  2. Two marks after `-`: U+0301+U+0300 and U+0300+U+0301 (do weights add, and does order matter?),
     and a mark after `-` plus a mark on the next letter (`"a-"&UNICHAR(769)&"b"&UNICHAR(769)`).
  3. U+0301 after the en dash, the em dash, U+FF07, U+FF0D, CHAR(1) and CHAR(127): does each rise
     above its own unmarked form and stay below the next skipped character's unmarked form?
  4. Positions: a mark after a leading hyphen (`"-"&UNICHAR(769)&"ab"` against `"-ab"` and `"áb"`),
     after a trailing one (`"ab-"&UNICHAR(769)` against `"ab-"`, `"ab'"`), and after two hyphens
     (`"a--"&UNICHAR(769)&"b"`).
  5. Equality: `=` and `COUNTIF(r,x)` of each marked text against its unmarked form (expected not
     equal, as T-3304's COUNTIF 1 for the exact text implies; confirm).
- Method: the `excel-probe` skill on excel-graph, laid out like probes/t3304-collation-multi-letter-
  accents (a Data column of strings built with UNICHAR so the workbook stays ASCII,
  `SUMPRODUCT(--(range<x))` ranks, chosen pairwise `<` cells and a COUNTIF column). Under about 400
  formula cells. Case `probes/t3417-collation-marks-after-skipped`, milestone tag M2 (decision 0051).

## Acceptance criteria
- [ ] probes/t3417-collation-marks-after-skipped is committed with its excel-graph oracle sidecar and
      answers questions 1-5.
- [ ] coercion.md's T-3304 bullet is extended (or a sub-bullet added) with the rule per shape, the
      case id and the cells, VERIFIED; a shape whose result is ambiguous stays UNVERIFIED with the
      reason.
- [ ] Every cell the engine fails is a known failure in eval/baseline/probes.txt (`make
      eval-baseline`, 0 regressions). If T-3329 is still open, its cells go into T-3329 when they
      follow its rule; any that contradict its rule, or if T-3329 is done, go into a new engine task
      (`make new-task`, M2, lane engine) filed in this PR (case stem and A1 names in one bullet,
      decision 0159), so `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (T-3329 or the engine task this files).
- Marks after `æ`/`ß` and U+0363-U+036F (T-3203); the mark weights themselves on a letter (T-3305);
  ignorable format characters (T-3398 on agent/engine/tasks-m2-opus-plan-2026-10-31).
