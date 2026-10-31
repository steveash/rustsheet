---
id: T-3446
title: "Probe script digits in typed input, percent, currency, parentheses, groups and exponents (T-3325 follow-up)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's reading of Arabic-Indic, Devanagari and Thai digits is probed and recorded for typed cell input, percent, currency, accounting parentheses, group separators, exponents, dates and times, and other digit blocks (Bengali, Lao, Tibetan, Myanmar).

## Context
T-3325 maps the three blocks in `cast::number_text_in` (coercion, VALUE, criteria) from VALUE/`+0`/COUNTIF evidence only. Typed input is deliberately not mapped; the other forms follow the mapping by inference. See docs/excel-semantics/coercion.md "Other scripts ... (T-3276)". Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers each form above; coercion.md records results; engine changed (or follow-up filed) where the probe disagrees.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Collation of these digits (T-3324).
