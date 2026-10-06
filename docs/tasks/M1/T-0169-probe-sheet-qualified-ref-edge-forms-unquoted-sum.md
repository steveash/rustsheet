---
id: T-0169
title: "Probe sheet-qualified #REF! edge forms (unquoted, SUM, missing sheet, SHEET(name), bare error-literal name bodies)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence (excel-probe) for the hand-expectation forms left by T-0152, with any differences fixed.

## Context
T-0152 made `Sheet!#REF!` lex as `#REF!` and defined names with an error-literal body evaluate to that error. Only the quoted cell form and the name body have Excel cells. Open: `=Sheet1!#REF!`, `=SUM(Sheet1!#REF!)`, a nonexistent sheet prefix (`=NoSuchSheet!#REF!`), `SHEET(name)` of such a name (we return the internal #ERROR!), and bare error-literal bodies (`=#N/A`) as defined names. See docs/excel-semantics/references.md "A sheet-qualified #REF! is #REF!".

## Acceptance criteria
- [ ] A probe case with oracle values covers each form above; the engine matches or the difference is fixed; references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
