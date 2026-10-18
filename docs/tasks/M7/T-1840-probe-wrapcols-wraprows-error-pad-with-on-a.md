---
id: T-1840
title: "Probe WRAPCOLS/WRAPROWS error pad_with on a multi-element vector and short vectors"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WRAPCOLS and WRAPROWS match Excel for an error pad_with on a vector of more than one element, and for a vector shorter than wrap_count.

## Context
T-1348 returns an error pad_with as the whole result when nothing is padded, on evidence for a single value only (gen-m9/675763979c1defb6 A6); WRAPCOLS_ROWS.xlsx E10 shows it fills padded cells. Use the excel-probe skill for `WRAPCOLS({1,2},4)`, `WRAPROWS({1,2},4)`, `WRAPCOLS(B5:H5,7,NA())`, `WRAPCOLS(B5:H5,100,NA())` and `WRAPCOLS({1,2,3,4},2,NA())`, add them as eval cases and fix wrapcols_wraprows.rs (`get_pad_with`) to match; update arrays.md "WRAPCOLS of a single value is not padded".

## Acceptance criteria
- [ ] The probe cells are in an eval case with oracle values and pass; arrays.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
