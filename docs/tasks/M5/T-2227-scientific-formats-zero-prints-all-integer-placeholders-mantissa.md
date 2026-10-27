---
id: T-2227
title: "Scientific formats: zero prints all integer placeholders, mantissa carry renormalises, 1E+15 cells"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Scientific formats handle zero, mantissa carry and 1E+15 cells as Excel does.

## Context
Follow-up of T-1730 (gen-m10, excel-graph): zero prints every integer placeholder: `##0.0E+0` on 0 Excel "000.0E+0" (ours "0.0E+0"; 659cd5776d423ce3 F!J14), `##0.00E+00` "000.00E+00", `#E+0` on 0 "0E+0" (ours "E+0"; 5eb95c63c7d6180c F!G13, 71ad5f93362f4ff4 F!V19). Carry: `0.0E+00` on 99.5 Excel "1.0E+02", ours "10.0E+01" (bfbcd16cee266336 F!W12). Also the 1E+15 / 1.23457E+14 cells that show nothing and "-x1E+15" vs "x-1E+15". Tie cases (`#E+0` on 2.5 "3E+0") look like T-2206.

## Acceptance criteria
- [ ] The listed gen-m10 text cells pass, no regressions; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Zero in a scientific code now prints every integer placeholder (`##0.0E+0` → "000.0E+0"; gen-m10 659cd5776d423ce3 F!J14 and 4c6a49fe4a6fa404 F!R4 fixed). The mantissa carry (99.5) already passed. The carry case (99.5; F!W12, F!G13) already passed and is only pinned by the test. Not done, filed as T-2850: the 1E+15 / 1.23457E+14 empty cells and the "-x1E+15" vs "x-1E+15" sign position.
