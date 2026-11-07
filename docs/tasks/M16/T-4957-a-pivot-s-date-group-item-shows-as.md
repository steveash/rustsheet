---
id: T-4957
title: "A pivot's date group item shows as a date, not its serial"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4767]
---

## Goal
A pivot item that comes from a date group item (`fieldGroup/groupItems/d`) shows as Excel for the web shows it: the
date text (1/1/2020), not the serial (43831).

## Context
Found by T-4767 (pivots.md "Capability matrix 3"): on probes/t4767-pivot-group-items, a manual group whose item is
`<d v="2020-01-01T00:00:00"/>` renders as the number 43831 shown 1/1/2020; the engine has the value but shows 43831.
The source field is text (Region), so this is not T-4807's rule (date items in the source cells' format): the format
comes from the group item's type alone. Which format Excel applies (m/d/yyyy here) is all the probe shows.

Owner of these visibly-off cells (decision 0245):
- probes/t4767-pivot-group-items GD!A6, GDA!A6

## Acceptance criteria
- [ ] GD!A6 and GDA!A6 of probes/t4767-pivot-group-items pass; pivots.md "Capability matrix 3" names the code.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date items of a date source field (T-4807).
