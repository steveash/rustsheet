---
id: T-2944
title: "Keep whitespace beside a missing argument (SPEC-0023)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Whitespace beside a missing argument prints and saves as Excel does.

## Context
T-2786 found Excel keeps a run between commas or after the last comma (`=IF(1, ,2)`, `=SUM(1, )`, `=NOW( )`, `=SUM(, 1)`), where SPEC-0023 drops a run before a comma or `)`. A saved `<f>` writes a line feed as raw CR LF bytes, which read back as LF. See syntax.md "Whitespace beside a missing argument is kept" and xlsx-storage.md "Typed whitespace is saved"; probes/t2786-whitespace-structured-refs-runs F/I cells. `=SUM (1)` and `=SUM( )` are refused by Graph (not a parse requirement).

## Acceptance criteria
- [x] The I1:I13 FORMULATEXT cells of probes/t2786-whitespace-structured-refs-runs pass, except I1 and I11, which moved to T-2966 (see Outcome).
- [x] SPEC-0023 "Where whitespace attaches" updated; a kept line feed round-trips as LF.
- [x] `make check` green; reviewers approve.

## Out of scope
- Structured references (T-2801).

## Outcome
- A missing argument after a comma now holds the run written in its place, as a `Spaced` around
  `EmptyArgKind` (`parser/lists.rs` `missing_argument`). `=IF(1, ,2)`, `=IF(1,  ,2)`, `=IF(1,` LF `,2)`,
  `=SUM(1, )`, `=SUM(1,  )`, `=SUM(1,` LF `)` and `=IF(1,2, )` print back as typed. Evaluation reads the stripped
  tree, so values are unchanged. The LF prints as LF, through the same `Run` the other kept runs use.
- I1 and I11 (`=NOW( )`, `=TODAY(  )`) are split off to T-2966. An empty call has no argument node for the run,
  and adding one would change every arg-count check. So of I1:I13, these two still fail and I12/I13 already
  passed.
- Eval: probes known failures 2620 -> 2613 (t2786 I2, I3, I4, I5, I8, I9, I10), no regressions.
