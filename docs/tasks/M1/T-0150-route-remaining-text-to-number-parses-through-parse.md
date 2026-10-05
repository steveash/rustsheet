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
T-0143 added `parse_excel_number` (number_format.rs). Done in T-0143: lexer, `cast_number`, `parse_formatted_number` (VALUE). The `"NaN"`/`"inf"`/`1e400` part is done (T-0920: those sites now use `parse_finite_number`, probes/text-number-words); still open: `"<17 digits>"+0` has no oracle case. Sites still using raw `str::parse::<f64>` on user text: utils.rs:330, functions/statistical/count_and_average.rs:263 and :319, functions/criteria.rs:19 and :317, functions/text/string_format.rs:188 (NUMBERVALUE), formatter/lexer.rs:195, user_model/sequence_detector.rs:218, conditional_formatting.rs:118. Also check autofill output for very small/large steps: `to_excel_precision_str` may print exponent notation where `to_string` printed plain decimals (T-0143). Each needs Excel evidence (docs/excel-semantics/numbers.md) before switching; use the excel-probe skill where uncited.

## Acceptance criteria
- [ ] Each site above either uses `parse_excel_number` (with a test or eval case) or has a documented reason in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
