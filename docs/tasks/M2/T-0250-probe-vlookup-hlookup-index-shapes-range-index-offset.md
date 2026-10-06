---
id: T-0250
title: "Probe VLOOKUP/HLOOKUP index shapes: range/INDEX/OFFSET indexes and the negative #VALUE!/#REF! cut-off"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the VLOOKUP/HLOOKUP index cases T-0211 left unprobed, and the code matching it.

## Context
From the T-0211 fidelity review. lookup.md "VLOOKUP / HLOOKUP search before validating the index"
(probes/lookup-index-order) covers single-cell references and literals. `resolve_lookup_index`
(functions/lookup_and_reference/index_arg.rs) maps *every* failed conversion of a reference index to
`#REF!`, and treats an index below -2^31 as `#REF!` and anything else below 1 as `#VALUE!`; neither
is evidenced beyond -2147483649 and -1E+15.
Probe (excel-probe skill, add to a new probes case): an index that is a multi-cell range in and out
of implicit intersection (`VLOOKUP(2,A1:C3,E1:E2)` from rows 1, 2 and 5), indexes from
`INDEX(E1:E3,2)`, `OFFSET(E1,1,0)` and `INDIRECT("E1")` holding text/number/error, and the negative
cut-off (-2147483648, -2147483647, -65536, -32769, -32768), each for a found and a not-found value.

## Acceptance criteria
- [ ] Probe case with oracle values; lookup.md updated (the "Open:" line resolved).
- [ ] `resolve_lookup_index` changed if Excel differs; unit tests for each new rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe case probes/lookup-index-shapes (oracle-filled, excel-graph). Found: the negative cut-off is the 32-bit minimum itself (-2147483648 is #REF!, -2147483647 is #VALUE!), fixed in `resolve_lookup_index`; OFFSET/INDIRECT indexes and implicit-intersected range indexes already matched; INDEX(...) as an index is a reference in Excel, not implemented (4 new-coverage failures baselined, follow-up T-0251). lookup.md updated.
