---
id: T-1652
title: "Probe R1C1-shaped defined and sheet names, then fold parse_reference_r1c1 into parse_r1c1_reference"
milestone: M9
lane: quality
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One R1C1 reference parser serves both the INDIRECT/R1C1 reader and the "is this name R1C1-shaped" test, with the surviving semantics backed by oracle evidence.

## Context
Split from T-0958 (CLAUDE.md directive 3). `parse_reference_r1c1` (crates/rustsheet-engine/src/expressions/utils/mod.rs) is used only as an R1C1-shape test in `is_valid_a1_identifier` and `name_needs_quoting`; `ParsedReference::parse_r1c1_reference` (crates/rustsheet-engine/src/utils.rs) reads INDIRECT's R1C1 text. They disagree: the old one is case-sensitive (uppercase `R`/`C` only), returns `None` below 4 characters (`RC`, `R1`, `R1C` are not R1C1-shaped to it), rejects bare `R`/`C`, and ignores sheet bounds; the new one is case-insensitive, accepts `R`, `C`, `RC`, `R3`, `C[1]`, wraps offsets and range-checks. Swapping one for the other changes which defined names are valid and which sheet names are quoted, so Excel decides first (excel-probe skill): FORMULATEXT of references to sheets named `RC`, `R1`, `R1C`, `r1c1`, `R0C1`, `R1048577C1`, `R[1]C[1]`, `C3`; and whether those strings are accepted as defined names (if the probe tooling can define names; otherwise record that gap).

## Acceptance criteria
- [ ] A probe with oracle values for the sheet-name quoting cases (and defined names if the tooling supports them); docs/excel-semantics entry recorded.
- [ ] One R1C1 parser (or one shared shape test built on it) with the probed semantics; `parse_reference_r1c1` deleted; eval unchanged except probed fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
