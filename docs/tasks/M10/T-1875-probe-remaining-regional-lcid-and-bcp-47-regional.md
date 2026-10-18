---
id: T-1875
title: "Probe remaining regional LCID and BCP-47 regional date-name variants"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Other regional LCIDs and BCP-47 regional tags print Excel's names, backed by probes, instead of the inferred language fallback.

## Context
T-1858 showed the language fallback is wrong for de-AT, zh-SG, es-MX and es-AR (docs/excel-semantics/number-formats.md "Locale tags localize date names"). Unprobed: es-CO `[$-240A]`, es-CL `[$-340A]`, es-419, de-LU `[$-1007]`, de-LI `[$-1407]`, zh-MO `[$-1404]`, en-AU/CA/IN, pt-AO, and BCP-47 regional tags such as `[$-de-AT]`, `[$-es-419]`. Probe each as probes/text-locale-regional-lcids does (12 `mmm|mmmm|mmmmm`, 7 `ddd|dddd`, `h AM/PM` at 0.25 and 0.75) and add tables or entries in locale/regional_date_names.rs and `LOCALES`.

## Acceptance criteria
- [ ] A probe case covers the variants above; its cells pass; number-formats.md records which differ from the language fallback.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
