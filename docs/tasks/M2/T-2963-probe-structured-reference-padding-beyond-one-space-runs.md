---
id: T-2963
title: "Probe structured-reference padding beyond one space: runs, line feeds, one side, lone specifiers, two commas"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how Excel prints the structured-reference whitespace shapes T-2801 implemented by guess.

## Context
T-2801 matched probes/t2786-whitespace-structured-refs-runs, whose padded shapes have one space each
side. Beyond that, ours keeps the runs verbatim. The other guessed shapes are a one-sided run, a lone
padded item specifier and a two-specifier form, which keeps only the second comma's space. MS-XLSB's
PtgList stores only one-bit `squareBracketSpace` and `commaSpace` flags, which hints that Excel may
normalize these differently. See syntax.md "Whitespace beside a missing argument is kept; a structured
reference keeps some and normalizes the rest" (Ours) and the `TableSpacing` lexer/printer in
crates/rustsheet-engine/src/expressions/{lexer/structured_references.rs,parser/structured_reference.rs}.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) has excel-graph values for FORMULATEXT of these shapes against a
      table `T` (Col, Other): `T[  [Col]  ]`, `T[` LF `[Col]` LF `]`, `T[[Col] ]`, `T[ [Col]]`,
      `T[ [#Data] ]`, `T[[#Headers], [#Data],[Col]]`, `T[[#Headers],[#Data], [Col]]`, `T[[#Data],` LF `[Col]]`.
- [ ] syntax.md records the results. Any mismatch with ours gets an engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
