---
id: T-0488
title: "SPEC-0020 amendment: tailed whole-column and past-the-used-area operands for UNIQUE, TEXTJOIN, LOOKUP and SMALL (corpus-bench 45896 17.6 s, 33935 8.6 s, 59969 5.7 s)"
milestone: M11
lane: perf
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
SPEC-0020 says, in an accepted amendment, how the tailed (used head plus one repeated tail row) evaluation reaches
three consumer shapes that real workbooks use and that today materialize 1,048,576-row arrays per formula, so that
the follow-up implementation tasks it lists can bring the three corpus-bench files below 1 s each with
bit-identical results.

## Context
Gap triage 2026-10-27 (`rustsheet-eval fetch`, then `make eval` on origin/main 6087413b): the slowest cases of the
whole run are three small corpus-bench files, all passing but slow, and no open task owns them. T-0448's Outcome
measured the same files and found they are not lookup-bound.
- **corpus-bench 45896** (13 KB, 90 formulas, 17.6 s): 27 array formulas
  `TEXTJOIN(",",TRUE,UNIQUE(IF(A2=ZORD!A:A,ZORD!C:C,"")))` over a 75-cell sheet ZORD, about 0.6 s each. UNIQUE and
  TEXTJOIN consume a whole-column IF; SPEC-0020's non-goals exclude every consumer but the aggregate walk.
- **corpus-bench 33935** (12 KB, 58 cells, 8.6 s): 29 array formulas `IF(L3>1,LOOKUP(2,1/(J:J<>""),J:J))` (N2 and down), the "last non-empty
  cell" idiom: LOOKUP's approximate search over a whole-column expression.
- **corpus-bench 59969** (16 KB, 162 cells, 5.7 s): `IFERROR(INDEX(Report!G:G,SMALL(IF((Report!$A$2:$A$70000=Form!$B$3)
  *(Report!$B$2:$B$70000=Form!$B$5),ROW(Report!$A$2:$A$70000)),ROWS($A$1:A1))),"")`: bounded 70,000-row ranges that
  run far past the used area, a position-dependent ROW() operand, and SMALL as the consumer.
Timings are one `rustsheet-eval run --suite corpus-bench --filter <id>` each, release build.

SPEC-0020 (docs/specs/SPEC-0020-whole-column-array-operands.md) built `Tailed`, `tailed_leaves` and `read_tailed`
(T-0469, engine `tailed_array.rs`) for the aggregate walk only; T-0470 (element-wise functions) and T-0471
(SUMPRODUCT) extend the same consumer. The questions the amendment must settle:
1. Which consumers take a tailed argument, and with what contract: UNIQUE (the tail value appears once, at the
   position of its first row), TEXTJOIN with ignore_empty (an empty tail contributes nothing; with ignore_empty FALSE
   it contributes tail_len delimiters, which must still be bounded by the text length limit), LOOKUP's approximate
   search (the tail is one run of equal keys), SMALL/LARGE (tail_len copies of one value, or none for an empty
   tail).
2. Whether a bounded range that extends past the used area (`$A$2:$A$70000`) is a tailed leaf like a whole column.
3. ROW()/COLUMN() over such a range is position-dependent, which "Which expressions are tailed" now refuses:
   can it be tailed as an arithmetic sequence, or does 59969 need another route (for example SMALL over the head
   only when the tail is FALSE in the IF's condition)?
4. The equivalence and error-precedence invariants per consumer (same value, error and reads as the
   materialized path), and the size checks (`#ERROR!` above 1,048,576 cells, arrays.md) on the logical size.

## Acceptance criteria
- [ ] SPEC-0020 has an amendment section (status stays accepted) answering questions 1-4, with the design for
      each consumer and a Rollout list of implementation tasks (S/M each), filed with `make new-task` (lane perf,
      tier opus), each naming the corpus-bench file it targets and its measure.
- [ ] A bench case in the aggregate or lookup bench reproduces each of the three shapes (with 2,000-row data) so
      the follow-ups can measure against it; its current timing is in the Outcome.
- [ ] No behavior change in this task; `make check` green; reviewers (architecture, performance) approve.

## Out of scope
- Implementing the consumers (the Rollout tasks).
- Aggregates (T-0469, T-0470) and SUMPRODUCT (T-0471).
