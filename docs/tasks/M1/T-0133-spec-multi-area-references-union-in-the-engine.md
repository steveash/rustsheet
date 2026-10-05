---
id: T-0133
title: "Spec: multi-area references (union) in the engine"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0135]
---

## Goal
A spec in `docs/specs/` that settles how the engine represents a multi-area reference, so unions
can reach functions that take them (`SUM((A1,B2))`, `AREAS`, `INDEX(…,area_num)`) without a
redesign.

## Context
T-0135 implements only what gen-m1 measures: a union in a value context gives `#VALUE!`, and an
intersection that cuts it down to one cell gives that cell. It deliberately leaves out a general
multi-area value. CLAUDE.md directive 4 asks for a spec before that architectural step. The spec
should decide:
- the `CalcResult` (or reference) representation of several areas, possibly on different rows,
  columns or sheets;
- how functions declare that they accept multi-area references;
- the lexing rule for a union comma versus an argument separator, including comma-decimal locales;
- stringify round-trip;
- which Excel behaviour needs an excel-probe first (AREAS of a nested union, a cross-sheet union).

## Acceptance criteria
- [ ] `docs/specs/SPEC-XXXX-multi-area-references.md` is merged, with the representation, function
  contract, lexing rule, probes needed and follow-up implementation tasks filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing it (the follow-up tasks do that).
