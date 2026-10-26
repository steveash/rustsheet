---
id: T-2782
title: "SPEC-0023 step 2: normalise a whitespace run to its line feeds and last spaces, wrapping the space count at 256"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2781]
---

## Goal
A typed whitespace run prints as Excel stores it: only its line feeds and the spaces after the last
one, the space count wrapping at 256 (300 spaces print as 44); the line-feed count as a probe shows.

## Context
- From T-2781: a run holding a line feed prints nothing yet (`Run::of`); probes/t2769-formula-text-whitespace
  C17 and t2776-formula-text-whitespace-shapes C13 (a line feed after `=`) are this task's too.
SPEC-0023 step 2 ("A run is a line-feed count plus a space count"). Evidence:
probes/t2776-formula-text-whitespace-shapes C19:C22, F21 (excel-graph, 2026-10-26). Exactly 256 spaces is
unprobed: probe it first (excel-probe skill) before relying on the byte reading.

## Acceptance criteria
- [ ] t2776-formula-text-whitespace-shapes C19:C22, F21 pass, plus any t2769 line-feed cell T-2781 handed over.
- [ ] Probes, before any code: 255, 256 and 257 typed spaces, and 255, 256, 257 and 300 typed line feeds (the
      line-feed wrap is unprobed; the BIFF layout alone is not evidence). The parser matches them; no panic on any count.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Stored-text runs (T-2784).

## Handoff (resumed)
Claimed by a sonnet run, then escalated at once (work-loop step 5b: unclear evidence) without code changes. The task needs
oracle probes first (255/256/257 spaces; 255/256/257/300 line feeds; the line-feed wrap is unprobed), then a parser change in
`Run::of` driven by those results. `rustsheet-oracle status` showed excel-graph OK in this session, so the probes can be filled.
Next: write the probe case with the excel-probe skill, fill it, read the wrap behaviour from the oracle values, then implement
SPEC-0023 step 2 against t2776-formula-text-whitespace-shapes C19:C22, F21 and the t2769 C17 / t2776 C13 cells.
