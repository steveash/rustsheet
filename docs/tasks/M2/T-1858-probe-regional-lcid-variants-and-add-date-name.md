---
id: T-1858
title: "Probe regional LCID variants and add date-name tables for more locales"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: M
tier: sonnet
depends_on: []
---

## Goal
Locale tags for regional variants and for locales with no table yet print Excel's names, backed by probes, instead of an inferred language fallback or English.

## Context
T-1716 added Excel's date names for 25 LCIDs (crates/rustsheet-engine/src/locale/excel_date_names.rs) and an inferred fallback: an LCID or BCP-47 tag with no table uses the first table of its language (`formatter/locale_tag.rs` `resolve_lcid`, `resolve_bcp47`). The probes behind that fallback (`[$-80C]`, `[$-C]`, `[$-7]`, `[$-A]`, `[$-11]`, `[$-16]`, `[$-4]`, `fr-BE`, `fr-XX`) only hit variants whose names are identical. Variants that may differ were never probed: `[$-C07]` de-AT ("Jänner"?), `[$-1004]`/`[$-C04]` zh-SG/zh-HK, `[$-80A]` es-MX, `[$-2C0A]` es-AR. Locales with no table at all (`[$-408]` Greek, `[$-41E]` Thai, `[$-41B]` Slovak, `[$-422]` Ukrainian, `[$-401]` Arabic, `[$-40D]` Hebrew, `[$-439]` Hindi) resolve to `Silent` and print English names. Probe them the way probes/text-locale-name-tables does (12 `mmm|mmmm|mmmmm`, 7 `ddd|dddd`, `h AM/PM` at 0.25 and 0.75) and add tables or LCID entries. Arabic, Hebrew and Thai may also switch calendar or digits; leave those parts to T-1859. See docs/excel-semantics/number-formats.md "Locale tags localize date names".

## Acceptance criteria
- [ ] A probe case covers the LCIDs above; its cells pass or are filed, and number-formats.md records the result (including whether the language fallback holds).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
