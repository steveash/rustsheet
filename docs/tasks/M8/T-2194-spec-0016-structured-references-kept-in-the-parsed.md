---
id: T-2194
title: "SPEC-0016: structured references kept in the parsed formula"
milestone: M8
lane: engine
status: blocked
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
- [ ] `make check` green; the architecture reviewer approves.

## Out of scope
- Implementation (T-2195, T-1699).

## Blocked because
The review-round limit (work-loop step 5, 3 rounds) ran out on 2026-10-20. Security and quality approved round 1. Architecture approved round 1 and requested changes in round 2. Excel fidelity requested changes in all three rounds, with fewer findings each time:
- Round 1: export without table parts, and the bare name in a file. Fixed: export stays lowered until T-2196.
- Round 2: values on structural edits. Fixed: the "Structural edits" section.
- Round 3: moves, cut and paste, and CF and defined-name formulas left out; the evidence for ISREF of `#This Row` overstated. Fixed in the last commit, which has not been re-reviewed.

Architecture round 2 also asked for the mechanism for passing tables into the displace printer, the edge-case table, and the `ThisRowKind` printing rule; those are fixed and not re-reviewed either. Next: re-run `reviewer-architecture` and `reviewer-excel-fidelity` on the branch head (`review` skill). If both approve, set the status to `done`, set the spec status to `accepted`, and merge. `make check` passed on the first commit, and every later change is docs only.

## Outcome (draft, for when it merges)
SPEC-0016: a `StructuredReferenceKind { reference, lowered }` wrapper made transparent through `ungrouped()`, plus a late-bound `ThisRowKind` for `#This Row` (shared formulas parse at (1, 1), so the row has to be bound at evaluation). The xlsx export keeps lowered ranges until the exporter writes table parts (T-2196), because a structured reference in a file without its table, or a bare table name, breaks the file in Excel (xlsx-storage.md). Filed T-2195, T-2196 and T-2197; T-1699 now depends on T-2195. Added the UNVERIFIED references.md entry "Structured references when the table changes". Eval delta: none (docs only).
