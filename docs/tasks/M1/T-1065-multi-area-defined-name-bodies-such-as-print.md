---
id: T-1065
title: "Multi-area defined-name bodies such as Print_Area (SPEC-0004 step 5)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1062, T-0158]
---

## Goal
A defined name whose body is a union, such as `_xlnm.Print_Area` = `Sheet1!$A$1,Sheet1!$B$2`, loads and stands for its areas: `SUM(mu)` 6, `COUNT(mu)` 2, `ISREF(mu)` TRUE, `mu A1` 1, `=mu` `#VALUE!`.

## Context
SPEC-0004 step 5. probes/union-defined-name H1:H5 (T-0187) fails today: `parse_defined_names` (new_empty.rs) refuses the body. Add a name-body parser entry point that reads a top-level comma as the union (only a name body; a cell's `=A1,B2` stays a syntax error), `ParsedDefinedName::Areas` for unions of absolute references, and the general-formula path of T-0158 for anything else. A body over `MAX_AREAS` areas is refused at load (DEVIATION, references.md "Union area cap"). Evidence: references.md "Union: parsing and evaluation" (multi-area defined names) and SPEC-0004 probe row 14.

## Acceptance criteria
- [ ] probes/union-defined-name H1:H5 and probe row 14 pass; baseline updated.
- [ ] A Print_Area body round-trips through xlsx export unchanged (unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Names whose body is not a union (T-0158).
