---
id: T-0435
title: "Allocation-free month-name lookup and a stack buffer for date-text words"
milestone: M3
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Parsing text that is not a date allocates nothing in the month-name lookup, and `parse_date` keeps its words and glued parts on the stack.

## Context
Found by the T-1789 performance review. `parse_month_name` (crates/rustsheet-engine/src/formatter/date_text.rs)
allocates `name.to_lowercase()` and then lowercases each locale month name it compares, about 25
short-lived Strings per miss. Since T-1789, letters-then-digits values such as IDs and codes (`"A1"`,
`"ID123"`, `"Q12026"`) reach it through `split_glued_month` on every text-to-number coercion.
`parse_date` also collects `words` and `parts` into two heap Vecs, bounded at 5 and 15 entries.

## Acceptance criteria
- [ ] `parse_month_name` compares case-insensitively without allocating (lowercased names stored once on `Locale`, or a char-wise comparison), with a cheap length or first-character check first.
- [ ] `words` and `parts` in `parse_date` are fixed-size stack buffers (or one), with no behavior change: the date_text tests and `make eval-check` are unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which texts parse as dates.
