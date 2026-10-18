---
id: T-1853
title: "Probe whether a legacy cell stays legacy after a row insert, copy and fill"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0012-legacy-n-t-marker.md
---

## Goal
Oracle evidence for SPEC-0012's rule 6. A legacy `=N(IF(1,{2;3}))+A5` is `#VALUE!` (a plain
`<f>`). Is it still `#VALUE!`, and does it still save as a plain `<f>`, after a row is inserted
above it, after it is moved, and after it is copied and filled to another cell? Or is it re-entered
as a dynamic formula that spills?

## Context
SPEC-0012 assumes the cell stays legacy, as Excel adjusts a stored formula without re-entering it,
and T-1852 implements that. The probe harness only writes static cells (rustsheet-case/v1), so
first find out whether the oracle can run an edit sequence. Check the Graph workbook range
`insert` / `delete` and any copy endpoint, and `rustsheet-oracle` (tools/). Then use the
excel-probe skill. If no oracle can do it, record that in references.md and keep the assumption,
marked unprobed. If only desktop Excel can do it, add an item to T-1075 (decision 0039).
references.md "Legacy N and T over a reference-returning call".

## Acceptance criteria
- [x] references.md records the outcome for row insert, move, copy and fill, with the evidence, or
  the reason no oracle can produce it.
- [x] Each mismatch with SPEC-0012 rule 6 is filed as a spec amendment task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the outcome.

## Outcome
- **Row insert: probed.** An excel-graph session inserted a row above a legacy `=N(IF(1,{2;3}))+A5`. The formula became `+A6`, stayed a
  plain `<f>`, and stayed `#VALUE!`. The C2 reference variant stayed legacy too. This matches SPEC-0012 rule 6, so no amendment task is needed.
- **Copy, move and fill: not probeable on excel-graph.** Graph v1.0 refuses `range/copyFrom`, `range/moveTo` and `range/autoFill` with
  400 BadRequest. They stay rule 6's assumption, and the desktop check is T-1075 item 12 (decision 0039).
- **Evidence:** docs/research/t1853-legacy-cell-edits-2026-10-18.md and references.md "Legacy N and T over a reference-returning call".
- **Side finding:** a probe of a legacy cell must list it under `legacy_formulas`. Otherwise spec2xlsx writes a dynamic `t="array"` formula.
- **Follow-up:** the row-insert result came from a one-off session and is not kept as an eval case. T-1933 adds a row-insert op to `sheet_ops`
  and makes it an oracle-filled probe.
- **Eval delta:** none (docs only).
