---
id: T-1897
title: "REGEXREPLACE replacement: PCRE2 extended escapes and case forcing"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
REGEXREPLACE reads every PCRE2 extended replacement escape as Excel does, case forcing included.

## Context
Found by T-1688 (probes/regex-pcre2-features). Excel's replacement follows PCRE2's extended substitution: `\U$1` on `(a)` gives `Ab` (B25), `\n` and `\t` are LF and TAB (B19, B20), and `\x` with no hex digits, `\q` and `\0` are `#VALUE!` (B17, B21, B24). T-1688 does `\n`, `\t`, the group references and the errors; every other letter escape is `#VALUE!` in ours for now (`Replacement::new` in functions/text/regex_functions.rs). Probe and implement `\U`, `\L`, `\u`, `\l`, `\E` (do they apply to group text as well as literals? does `\u\L` combine?), `\r`, `\a`, `\e`, `\f`, `\xhh`, `\x{hhh}`, `\o{...}`, `\ddd`, `\cX`. Case mapping uses the one-to-one mapping UPPER and LOWER use (text.md, T-1747); keep `Replacement::result_len` an upper bound of the result's bytes (the size check).

## Acceptance criteria
- [x] B25 passes and a probe case with oracle values covers the escapes above; text.md "REGEXREPLACE replacement syntax" records them.
- [x] `make check` green; reviewers approve.

## Out of scope
- The pattern side (T-1900).

## Outcome
`Replacement::new` now parses every PCRE2 extended escape as Excel does, case forcing (`CaseState`) included. A 244-cell
oracle probe (probes/regexreplace-extended-escapes, three rounds) fixed the details: Excel's `\l` acts as `\L` plus a
one-character lowering, `\101` is octal but `\20` is an error, `\b` is backspace, spaces inside `\x{...}` are ignored,
`\Q...\E` is supported. Recorded in text.md "REGEXREPLACE replacement syntax". Eval: probes B25, B26 of
regex-pcre2-features newly pass; the new probe case is 244/244.
