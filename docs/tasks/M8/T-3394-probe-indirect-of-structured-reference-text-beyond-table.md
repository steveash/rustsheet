---
id: T-3394
title: "Probe INDIRECT of structured-reference text beyond Table[Column]: specifiers, @, unqualified, case, spacing, bare name, a1 FALSE"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's result for INDIRECT of structured-reference text other than `Table[Column]` is probed, and the engine matches it.

## Context
T-2911 resolves only `INDIRECT("Table[Column]")` (`indirect_structured` in functions/lookup_and_reference/mod.rs); every other shape stays `#REF!` until probed: `#All`/`#Headers`/`#Data`/`#Totals`, `[@Col]`/`#This Row`, unqualified `[Col]` inside a table, case and spacing of the text, a bare table name, and `a1` FALSE. Record results in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [ ] Probe cases with Excel values exist for each shape listed in Context, and the engine passes them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
