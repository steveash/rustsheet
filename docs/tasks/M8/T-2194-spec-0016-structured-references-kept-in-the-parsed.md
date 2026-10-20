---
id: T-2194
title: "SPEC-0016: structured references kept in the parsed formula"
milestone: M8
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A reviewed spec says how structured references stay in the parsed formula: kept in the engine's storage (the export keeps lowered ranges until the exporter writes tables), re-resolved when the table list changes, kept through structural edits with today's values (moved tables, or a lowered fallback), and `#This Row` decided at evaluation. With it, T-1699 and T-1683 can be implemented.

## Context
T-1699 found that the parser lowers every structured reference to a range at parse time (`structured_reference_operand`, parser/operands.rs). `to_rc_format` then stores that range, so `=SUM(Sales[Qty])` is stored and exported as `=SUM($A$2:$A$3)`, and `=Sales[@Qty]` off the data rows as `=#VALUE!`. Re-adding the table over a larger range changes neither. Shared formulas are parsed with the context (1, 1), so a preserved `#This Row` cannot be resolved at parse time. Fixing this changes the evaluation model, so it needs a spec first (CLAUDE.md directive 4).

## Acceptance criteria
- [x] `docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md` covers the node design, transparency (`ungrouped()`), printers, shared formulas, alternatives, test plan and rollout.
- [x] Rollout tasks filed: T-2195 (step 1), T-1699 re-scoped as step 2 with `depends_on: [T-2195]`, T-2196 (step 3, table parts in the export) and T-2197 (probe).
- [x] `make check` green; the architecture reviewer approves.

## Out of scope
- Implementation (T-2195, T-1699).

## Outcome
SPEC-0016, accepted on merge of this task: a `StructuredReferenceKind { reference, lowered }` wrapper made transparent through `ungrouped()`, plus a late-bound `ThisRowKind` for `#This Row` (shared formulas parse at (1, 1), so the row has to be bound at evaluation). The xlsx export keeps lowered ranges until the exporter writes table parts (T-2196), because a structured reference in a file without its table, or a bare table name, breaks the file in Excel (xlsx-storage.md). Filed T-2195, T-2196 and T-2197; T-1699 now depends on T-2195. Added the UNVERIFIED references.md entry "Structured references when the table changes". Eval delta: none (docs only).
