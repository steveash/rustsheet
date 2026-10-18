---
id: T-1652
title: "Probe R1C1-shaped defined and sheet names, then fold parse_reference_r1c1 into parse_r1c1_reference"
milestone: M9
lane: quality
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One R1C1 reference parser serves both the INDIRECT/R1C1 reader and the "is this name R1C1-shaped" test, with the surviving semantics backed by oracle evidence.

## Context
Split from T-0958 (CLAUDE.md directive 3). `parse_reference_r1c1` (crates/rustsheet-engine/src/expressions/utils/mod.rs) is used only as an R1C1-shape test in `is_valid_a1_identifier` and `name_needs_quoting`; `ParsedReference::parse_r1c1_reference` (crates/rustsheet-engine/src/utils.rs) reads INDIRECT's R1C1 text. They disagree: the old one is case-sensitive (uppercase `R`/`C` only), returns `None` below 4 characters (`RC`, `R1`, `R1C` are not R1C1-shaped to it, but the 4-character `R12C` and `R[1]C` are), rejects every row-only and column-only form at any length (`R`, `C`, `R1234`, `C12`, `C[1]`: it needs a leading `R` and a `C`), and ignores sheet bounds (`R0C0` passes); the new one is case-insensitive, accepts `R`, `C`, `RC`, `R3`, `C[1]`, wraps offsets and range-checks. Swapping one for the other changes which defined names are valid and which sheet names are quoted, so Excel decides first (excel-probe skill): FORMULATEXT of references to sheets named `RC`, `R1`, `R1C`, `r1c1`, `R0C1`, `R1048577C1`, `R[1]C[1]`, `R12C`, `R`, `C`, `r`, `c`, `R3`, `C3`, `R1234`, `C[1]`; and whether those strings are accepted as defined names (if the probe tooling can define names; otherwise record that gap). The new parser also takes a cell context, a sheet-lookup closure, `Sheet!` prefixes and `:` ranges; the fold needs a shape-only entry point.

## Acceptance criteria
- [x] A probe with oracle values for the sheet-name quoting cases (and defined names if the tooling supports them); docs/excel-semantics entry recorded.
- [x] One R1C1 parser (or one shared shape test built on it) with the probed semantics; `parse_reference_r1c1` deleted; eval unchanged except probed fixes.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 44 cases on excel-graph (sheet-name quoting via FORMULATEXT for about 160 names, defined
names one per case, LET names, Lotus dotted ranges); recorded in references.md "R1C1-shaped sheet
and defined names". Excel's test is neither old parser's: it is a case-insensitive prefix test
(`R1CD`, `R1_`, `C1R1` quoted; `RCR`, `R0`, `R1.5` not), plus a separate Lotus-range test (`A1.B2`,
`R1.C1`). `parse_reference_r1c1` is deleted; `is_r1c1_shaped` (quoting and defined/table names) and
`is_dotted_a1_range` (quoting) replace it. The shape test is deliberately not built on
`ParsedReference::parse_r1c1_reference` (INDIRECT's reader): it has a different grammar (prefixes,
no brackets, no column bound after `R<n>C`), so do not try to fold them again. LET/LAMBDA names skip
the shape test, as in Excel, except uppercase `R<n>C<m>`, which the R1C1 storage would read back as a
reference (T-1658). Eval: 0 regressions; probes +44 cases, 36 new-coverage failures, all tracked:
T-1657 (out-of-range R1C1 defined names are #NAME? in Excel), T-1658 (LET `R1C1` names), T-1659
(Lotus dotted ranges in formulas).
