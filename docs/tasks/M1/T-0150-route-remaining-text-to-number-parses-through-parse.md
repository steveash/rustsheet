---
id: T-0150
title: "Route remaining text-to-number parses through parse_excel_number (VALUE, criteria, COUNT, utils)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every place that reads user text as a number keeps only 15 significant digits, like T-0143's lexer and `cast_number`.

## Context
T-0143 added `parse_excel_number` (number_format.rs). Sites still using raw `str::parse::<f64>` on user text: utils.rs:330, functions/statistical/count_and_average.rs:263 and :319, functions/criteria.rs:19 and :317, functions/text/string_format.rs:188 (NUMBERVALUE), formatter/lexer.rs:195. Each needs Excel evidence (docs/excel-semantics/numbers.md) before switching; use the excel-probe skill where uncited.

## Acceptance criteria
- [ ] Each site above either uses `parse_excel_number` (with a test or eval case) or has a documented reason in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
