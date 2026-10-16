---
id: T-1662
title: "Probe the T-1595 single-character collation choices in longer texts and lookups"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence decides each collation choice T-1595 made from single-character probes only.

## Context
T-1595 fitted `cmp_text_excel` (functions/util/collation.rs) to probes/text-collation-unprobed; the choices recorded as UNVERIFIED in docs/excel-semantics/coercion.md "Collation equality in lookups and criteria; unprobed characters" need multi-character and lookup probes: `"œ"="oe"` (and `XMATCH("oe",{"œ"},0)`, `COUNTIF` with `"oe"` against `œ`); control characters inside text (`"a"&CHAR(1)&"b"` against `"ab"`, `"a b"`, `"a'b"`, and CHAR(1) against `"'"`); CHAR(11) and CHAR(12) against CHAR(160) and CHAR(9); `± « » × ÷ § © ¬ ® ° µ … •` against `< = >`; `¼ ½ ¾ ¹ ² ³` against `1 2 3` (`"¹"` vs `"1"`, `"¹a"` vs `"1b"`); `ç đ ð ł ñ þ ı` as accent or letter (`"ða"` vs `"db"`, `"þ"` vs `"th"`). Use the rank layout of probes/text-collation-unprobed (excel-probe skill).

## Acceptance criteria
- [ ] One probe case covering the list above, filled by the oracle (or left pending with the task blocked on the oracle).
- [ ] coercion.md updated: each choice marked VERIFIED or corrected, with a follow-up engine task for any mismatch (`make eval-show` on the new case).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
