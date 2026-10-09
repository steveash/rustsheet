---
id: T-1065
title: "Multi-area defined-name bodies such as Print_Area (SPEC-0004 step 5)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1064, T-0158]
---

## Goal
A defined name whose body is a union, such as `_xlnm.Print_Area` = `Sheet1!$A$1,Sheet1!$B$2`, loads and stands for its areas: `SUM(mu)` 6, `COUNT(mu)` 2, `ISREF(mu)` TRUE, `mu A1` 1, `=mu` `#VALUE!`.

## Context
SPEC-0004 step 5. probes/union-defined-name H1:H5 (T-0187) fails today: `parse_defined_names` (new_empty.rs) refuses the body. Add a name-body parser entry point that reads a top-level comma as the union (only a name body; a cell's `=A1,B2` is `#VALUE!` in Excel when stored in a file, probes/multiarea-bare, T-1073 decides its load behaviour; do not make it a load error here), `ParsedDefinedName::Areas` for unions of absolute references, and the general-formula path of T-0158 for anything else. A body that fails to parse, has a cross-sheet or 3-D operand, or more than `MAX_AREAS` areas becomes `InvalidDefinedNameFormula` (never a load failure), and the areas of all names of a workbook together are capped at `MAX_AREAS` (DEVIATION, references.md "Union area cap"). Evidence: references.md "Union: parsing and evaluation" (multi-area defined names) and SPEC-0004 probe rows 14 and 14b: probes/multiarea-printarea and multiarea-xname (a cross-sheet name body loads; `SUM(xs)` and `=xs` are `#VALUE!`, `ISREF(xs)` FALSE; both cases must pass).

## Note from T-1064
Also move AGGREGATE's array form (`aggregate_array_form`, functions/aggregate.rs) off its `Node::OpUnionKind` syntax match onto `evaluate_reference_arg` (keep its `Array`, number and error arms and `#VALUE!` for a 3-D reference), so it sees unions behind names, LET variables and IF/CHOOSE.

## Note from T-1073
The parser now reads a top-level comma as the bare union (`Parser::parse_bare_union`, cells and name bodies alike), so `mu` = `Sheet1!$A$1,Sheet1!$B$2` parses: probes/union-defined-name loads (H2, H3 pass; H1 `SUM(mu)`, H4 `mu A1`, H5 `COUNT(mu)` still fail, the name is not yet evaluated as its areas) and probes/multiarea-xname passes. No separate name-body entry point is needed.

## Acceptance criteria
- [x] AGGREGATE's array form (`aggregate_array_form`, functions/aggregate.rs) reads its reference through `evaluate_reference_arg` instead of matching `Node::OpUnionKind`, so a union behind a LET variable, IF, CHOOSE, LET or a name reaches it (moved from T-1064); its `Array`, number and error arms and the `#VALUE!` for a 3-D reference stay.
- [x] probes/union-defined-name H1:H5 (including `mu A1`) pass; baseline updated. (Probe row 14, the sheet-local Print_Area, moved to T-1193: it needs sheet-scoped names in the case loader and `_xlnm.` names, not multi-area work.)
- [x] references.md "Union area cap" DEVIATION covers the name-body cap (and why no per-workbook cap is needed: names store no area list); unit tests.
- [x] A Print_Area body round-trips through xlsx export unchanged (unit test).
- [x] `make check` green; reviewers approve.

## Out of scope
- Names whose body is not a union (T-0158).

## Outcome
A name whose body is a union now stands for its areas wherever a reference is accepted:
`Model::defined_name_reference` evaluates a `Formula` body through `evaluate_reference_arg` under
the name depth and budget limits, and `is_multi_area` became a `Model` method that is true for a
name whose formula body may itself stand for several areas (looked through one level; cell,
range, array and scalar names keep their old paths, so FREQUENCY and root snapping are unchanged). AGGREGATE's
array form reads its argument through `evaluate_reference_arg`, so it takes a union behind a name,
LET, IF or CHOOSE. Unlike SPEC-0004's plan there is no stored `ParsedDefinedName::Areas` and no
per-workbook cap (names store no area list; spec "As built" note and references.md "Union area cap"
updated). Unit tests: names in SUM/COUNT/AREAS/INDEX/AGGREGATE/LET/intersection, a name of a name,
a cross-sheet body, the area cap through a name, self-referential union names, and an xlsx
round-trip of a union body. New probe probes/union-defined-name-uses (22 cells, excel-graph) backs
AREAS, INDEX, AGGREGATE, name-of-name, CHOOSE and LET uses and FREQUENCY over names, LET and IF arrays (T-1196, fixed here); 21 pass, COUNTBLANK(bad) is a
pre-existing gap (T-1195). Eval delta: probes 360 -> 358 known failures (union-defined-name H1, H4,
H5 fixed; one new-coverage failure), 0 regressions. Follow-ups: T-1193, T-1194, T-1195 (T-1196 filed and fixed in this PR). Probe row 14 (sheet-local Print_Area) split to T-1193.
