---
id: T-1105
title: "Negative serials with elapsed tokens are #VALUE! in the 1900 system; 1904 workbooks keep the minus"
milestone: M1
lane: functions-date
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1267, T-1268]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
TEXT and cell display of a negative serial under a format with an elapsed token match Excel: #VALUE! in the 1900 date system, the leading minus kept in a 1904 workbook.

## Context
probes/text-negative-serial-elapsed-mixed A1, A3, A5, A6 expect #VALUE! (dates.md, T-1079); the engine exempts elapsed sections in `ParsePart::Date` (formatter/format.rs) because poi ElapsedFormatTests A40-A47 is a 1904 workbook. The engine has no date-system awareness, so the fix needs the importer's date1904 flag plumbed to the formatter (design question, hence opus). Cell display (#####) under a date format is still unprobed: the case format cannot set a cell number format, so that needs a harness extension first.

Design: SPEC-0005 (rollout step 3), split out by T-1266.

## Acceptance criteria
- [x] probes/text-negative-serial-elapsed-mixed passes without regressing poi ElapsedFormatTests A40-A47.
- [ ] (moved to T-1367, desktop oracle) Cell display of a negative under a date format is probed (xlsx case with a number format) and recorded in dates.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

Note (T-1079 review): poi A40-A47 only evidence elapsed/time formats in 1904, never `d`/`mmm`/`yyyy` tokens; probe a 1904 mixed-token format before generalizing, and add `TEXT(-1E-10,"[h]:mm")` and `TEXT(-0,"[h]:mm")` to the 1900 probes.

## Outcome
- `formatter::format::FormatContext { locale, date_system }` replaces the bare `&Locale` of `format_number`/`format_text` (SPEC-0005 step 3). Model call sites (TEXT, `get_formatted_cell_value`, the date-guess path) pass `Model::format_context()`, built from `WorkbookSettings::date_system`; locale-only callers (`number_format::format_number`, `get_fmt_settings` examples, tests) use `FormatContext::new` (1900).
- `formatter::dates::negative_serial_allowed(tokens, date_system)` decides negatives, on the value's original sign: 1904 always formats; 1900 only a section whose one date/time token is a single elapsed token (literals, text and colors allowed). A negative in a one-section format prints its magnitude after one leading minus in front of everything (`"x"[h]` → "-x36"), so `elapsed_text` no longer handles the sign.
- New probes probes/negative-date-sections-1900/-1904 (23 questions each, excel-graph): an explicit negative date section is still #VALUE! in 1900 (`TEXT(-1,"d;d")`), a lone elapsed token keeps its literals, two elapsed tokens are #VALUE! for any value. Recorded in dates.md "Negative serials under an explicit negative section" and number-formats.md "Elapsed-time tokens".
- Criterion 2 (cell display) is not probeable through excel-graph (T-1268); moved to T-1367 (M14, blocked on the desktop oracle).
- Eval delta: probes 11 fixed (text-negative-serial-elapsed-mixed A1/A3/A5/A6, date-system-1900-negatives A2/A4/A5/A13/A15, date-system-1904-negatives A2/A7), poi 1 fixed (ElapsedFormatTests A40), +2 cases with 12 new-coverage failures (1904 epoch A1/A2/A12/A17/A20: T-1330; two elapsed tokens and `[ss] AM/PM` A9/A21/A22/A23: T-1366; 1904 `[s].00` A10: T-0934), no regressions.
