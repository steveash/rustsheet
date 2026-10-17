---
id: T-1825
title: "Probe call-name folds for Greek symbol variants, other ligatures and capital sharp s"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's call-name spelling folds for characters T-1818 left unprobed are probed and `spelling_key` matches them.

## Context
T-1818 found ı, ſ and µ do not fold though `to_uppercase` merges them; ϐ/β, ϑ/θ, ϕ/φ, ϖ/π, ϰ/κ, ϱ/ρ, ϵ/ε, ẛ, U+0345, ligatures ﬀ ﬃ ﬄ ﬅ ﬆ, ŉ/ǰ/ΐ, ẞ vs ß/SS and İ are still merged unprobed. Code: `spelling_key` in expressions/parser/call_names.rs; semantics: references.md "Unknown function call names".

## Acceptance criteria
- [ ] Probe case (pairs such as `=ϐx(1)` then `=βx(1)`) with oracle values; `spelling_key` matches them; references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 16 pairs (excel-graph). ϵ, ﬀ ﬃ ﬄ ﬆ and ẞ fold; ϐ ϑ ϕ ϖ ϰ ϱ, ﬅ and ŉ do not. `spelling_key` updated; U+0345, ǰ, ΐ stay unprobed (excel-graph rejects the workbook). Recorded in references.md.
