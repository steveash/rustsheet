---
id: T-1853
title: "Probe whether a legacy cell stays legacy after a row insert, copy and fill"
milestone: M2
lane: research
status: todo
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
- [ ] references.md records the outcome for row insert, move, copy and fill, with the evidence, or
  the reason no oracle can produce it.
- [ ] Each mismatch with SPEC-0012 rule 6 is filed as a spec amendment task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the outcome.
