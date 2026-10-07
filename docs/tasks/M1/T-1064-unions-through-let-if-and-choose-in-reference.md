---
id: T-1064
title: "Unions through LET, IF and CHOOSE in reference context (SPEC-0004 step 4)"
milestone: M1
lane: engine
status: todo
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1062]
---

## Goal
A union bound by `LET`, or returned by `IF` or `CHOOSE`, stays a multi-area reference where a reference is accepted (`LET(x,(A1,B2),SUM(x))`, `SUM(IF(TRUE,(A1,B2)))`) and is `#VALUE!` as a value.

## Context
SPEC-0004 step 4. `fn_let` (functions/logical/let.rs) evaluates bindings in value context, so a union binding is `#VALUE!`. Store a binding that can hold `Areas` (value read gives `#VALUE!`, reference read the areas). Give CHOOSE the `as_reference` flag IF and INDEX already have, and make `evaluate_reference_arg` recurse into the chosen branch. Evidence: SPEC-0004 probe rows 9 and 10 (cases from T-1061).

## Acceptance criteria
- [ ] Probe rows 9 and 10 pass.
- [ ] Unit test: nested LET shadowing a union binding; a deep IF chain stays within the parser depth cap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined names (T-1065).
