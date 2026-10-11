---
id: T-1367
title: "Desktop Excel: cell display of a negative under a date or elapsed number format, 1900 and 1904"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what desktop Excel displays in a cell holding a negative number under a date/time or elapsed number format (`#####` or text), in both date systems, and make `Model::get_formatted_cell_value` match it.

## Context
T-1105 (acceptance criterion 2) and T-1268: excel-graph reads cached values, not displayed text, so the display question needs the desktop oracle (decision 0039). The engine's cell display goes through the same `ParsePart::Date` arm as TEXT (formatter/format.rs, `negative_serial_allowed`), so today it shows `#VALUE!`-text where TEXT is #VALUE!. Questions: B1 = -1.5 formatted `[h]:mm`, `d`, `h:mm` and `[h]`, in a 1900 and a 1904 workbook. See dates.md "The 1904 date system" and "Negative serials under an explicit negative section".

## Acceptance criteria
- [ ] Desktop display text recorded in dates.md for each question.
- [ ] If it differs from the engine's formatted value, `get_formatted_cell_value` follows it, with a test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXT results (VERIFIED on the web: T-1268, T-1105).

## Blocked because
Needs the excel-desktop oracle (`rustsheet-oracle status`: excel-desktop n/a). Unblock when it is available.
