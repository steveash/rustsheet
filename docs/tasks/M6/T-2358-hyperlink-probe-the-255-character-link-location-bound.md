---
id: T-2358
title: "HYPERLINK: probe the 255-character link_location bound for characters outside the BMP"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know (oracle evidence) whether HYPERLINK's 255-character link_location bound counts UTF-16 units or code points for characters outside the BMP, and match it.

## Context
T-2277 added the bound (`hyperlink_location`, crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs; docs/excel-semantics/lookup.md "HYPERLINK: a link_location past 255 characters is #VALUE!", which says "Unprobed: characters outside the BMP"). Ours counts UTF-16 units. Probe e.g. `HYPERLINK(REPT("😀",127)&"a",1)`, `REPT("😀",128)`, `REPT("😀",200)` and `LEN` of each (excel-probe skill).

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values for at least the three formulas above.
- [ ] lookup.md's entry drops "Unprobed" and states the rule; ours matches (fix if not).
- [ ] `make check` green; reviewers approve.

## Out of scope
- HYPERLINK's displayed text and link storage.

## Outcome
Probed on excel-graph (probes/t2358-hyperlink-location-non-bmp): the bound counts UTF-16 units (127 emoji + "a" passes, 128 emoji is #VALUE!). Our implementation already matched; lookup.md updated. No eval baseline change.
