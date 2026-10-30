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

From T-1924 (2026-10-26): tables reached by the extended hex reading of a tag. They are 0x21 Indonesian (`[$-x]`), 0x24
Slovenian (`[$-{]`), 0x27 Lithuanian (`[$-~]`), 0x28 Tajik (`[$-28]`, `[$-428]`: "март|чоршанбе", "AM"), 0x2F Macedonian
(`[$-0_]`, `[$-_]`) and 0x3F Kazakh (`[$-1_]`). Cells: probes/t1924-locale-tag-extended-hex A1-A4, A10, A28, A29, B1-B4,
B10, B28, B29, and text-locale-calendars A140. Also open there: B22, where `[$-_409]` (0xF409) prints "6 PM" while
`[$-x_y]` (0x2412) prints English, so find where Excel falls back to the language. Probe the plain hex tags `[$-2412]` and `[$-2F409]` too, to learn
whether the extended path (`resolve_extended`, neutral LCIDs only) really differs from `resolve_lcid`'s fallback.

Exit owners (decision 0096; the 2026-10-27 planner run names each cell the ranges above hold):
- probes/t1924-locale-tag-extended-hex S!A2, S!A3, S!B2, S!B3
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - probes/t1924-locale-tag-extended-hex.case.json: S!B10, B22, B28, B29

## Acceptance criteria
- [ ] A probe case covers the variants above; its cells pass; number-formats.md records which differ from the language fallback.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
