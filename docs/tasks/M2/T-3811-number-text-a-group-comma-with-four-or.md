---
id: T-3811
title: "Number text: a group comma with four or more digits after it, a trailing comma and a leading percent sign"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text that number coercion reads (VALUE, `0+`, and through them the IM* part reader) reads group commas and a
leading percent sign as Excel does.

## Context
Found by T-3644 (probes/eng-inumber-grammar-value, excel-graph, 2026-11-02; coercion.md "Group commas and a
leading percent sign"). Excel reads "1,0000" 10000, "1,000,0000" 10000000, "1,0000.5" 10000.5 and "%1" 0.01 through
VALUE, `A+0` and IMREAL alike, and refuses "1," (#VALUE!). Ours does the opposite on all five; "1,00", ",1", "2,0" and
"2%%" are refused by both. The failing cells are probes/eng-inumber-grammar-value B1:D3, B8:D9 and
probes/eng-inumber-grammar-parts B18:D18, B20:D20, B32:D32 (baselined by T-3644). The grammar is
`parse_formatted_number` (formatter/format.rs) through `number_text_in` (cast.rs). A group of fewer than three digits
after a comma is refused; the rule for more than three is not probed past "1,0000" and "1,000,0000": probe the edges
(a comma group of 5 or 6 digits, a comma after the point, "1,000,00") before generalising, and a percent sign on
both ends ("%1%").

## Acceptance criteria
- [ ] The cells above pass; any new probe cells are filled from excel-graph and pass.
- [ ] coercion.md "Group commas and a leading percent sign" updated with the rule and the new cells.
- [ ] `make check` green; eval known failures only shrink; reviewers approve.

## Out of scope
- Typed cell input formats for these texts.
