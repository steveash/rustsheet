---
id: T-2462
title: "Re-fill gen-m7/0624b684889e9228 now that undecoded errors decode (T-2417)"
milestone: M7
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-2417]
---

## Goal
The gen-m7/0624b684889e9228 sidecar holds decoded Excel errors instead of `#UNDECODED!` placeholders, and its remaining Excel `<empty>` spill members are present.

## Context
T-2418 re-filled the other sidecars with dropped spill members but left this one out: a `--force` re-fill replaced many
`#UNDECODED!` cells with decoded `#VALUE!` (more than an added-cells change). The same applies to
probes/t2414-gen-m7-unexplained (input hash changed on re-fill). Re-fill with
`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite gen-m7 --case gen-m7/0624b684889e9228.case.json --force`.

## Acceptance criteria
- [ ] Every changed cell in the sidecar is listed in the Outcome with old and new value; each decoded value is checked against `make eval-show`.
- [ ] `make eval-baseline` shows 0 regressions.
- [ ] `make check` green; reviewers approve.

Also see T-2415 (arrays.md says this re-fill lands with it).

## Out of scope
- Engine fixes for any mismatch the decoded values expose (file a task).
