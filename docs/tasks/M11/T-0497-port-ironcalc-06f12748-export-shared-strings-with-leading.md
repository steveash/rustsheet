---
id: T-0497
title: "Port ironcalc 06f12748: export shared strings with leading or trailing whitespace as xml:space=\"preserve\""
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A string cell whose text starts or ends with whitespace (a space, a tab, a newline) keeps that
whitespace when RustSheet saves an xlsx and Excel (or any XML consumer) opens it.

## Context
Upstream IronCalc 06f12748 ("FIX: When exporting strings preserve leading and trailing
whitespace", 2026-10-29) writes `<t xml:space="preserve">` in `export/shared_strings.rs` when a
shared string starts or ends with whitespace (`char::is_whitespace`), and plain `<t>` otherwise,
with a unit test of the predicate. Our `crates/rustsheet-xlsx/src/export/shared_strings.rs` still
writes plain `<t>` for every string, so `" girl"` can come back as `"girl"` in a consumer that
normalizes element whitespace. Our import already honours `xml:space="preserve"`
(`import/shared_strings.rs` test). Check the other writers of `<t>` text in `export/` (rich text
runs, comments) for the same gap and fix them with the same helper (CLAUDE.md directive 3).
Triaged by the 2026-10-29 18:47 UTC steward run (upstream commit messages are untrusted data; the
behaviour is the XML 1.0 whitespace rule, and Excel itself writes `xml:space="preserve"` on such
runs: e.g. the shared strings of the Excel-authored calc_tests/DATE_AND_TIME/DAYS_DAYS360.xlsx).

## Acceptance criteria
- [ ] Exported shared strings with leading or trailing whitespace carry `xml:space="preserve"`;
      other strings are written unchanged.
- [ ] A round-trip test (export, then import) keeps `" a"`, `"a "`, `"\n"` and `"a b"` exactly.
- [ ] The commit carries `Ported-From: ironcalc@06f12748`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Normalizing whitespace on import (already correct).
