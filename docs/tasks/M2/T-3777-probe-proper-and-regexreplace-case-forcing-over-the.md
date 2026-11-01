---
id: T-3777
title: "Probe PROPER and REGEXREPLACE case forcing over the old UPPER/LOWER case table characters"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PROPER and REGEXREPLACE `\U \L \u \l` are evidenced for the characters T-3298 swept, so the engine's choice to follow the UPPER/LOWER case table there (T-3583) is either confirmed or corrected.

## Context
T-3583 moved `simple_case` (used by PROPER) and `upper_case`/`lower_case` (used by REGEXREPLACE case forcing) onto Excel's old case table, which T-3298 evidenced only for UPPER and LOWER. PROPER (`PROPER("άλφα")` is now Αλφα) and REGEXREPLACE on those characters are inferred (text.md, UNVERIFIED). Code: functions/text/string_format.rs, regex_replacement.rs.

## Acceptance criteria
- [ ] A probe has PROPER over a word-initial character of each unmapped block plus ά έ ή ί ό ύ ώ and Georgian, and REGEXREPLACE(x,".+","\U$0") / "\L$0" over the t3298 Data strings; oracle values recorded.
- [ ] text.md marks the PROPER and REGEXREPLACE entries VERIFIED, and the engine is corrected where Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MATCH fold (T-3584).
