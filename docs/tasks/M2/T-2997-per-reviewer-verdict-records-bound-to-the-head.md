---
id: T-2997
title: "Per-reviewer verdict records bound to the head sha for merge-train readiness"
milestone: M2
lane: harness
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2989]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A merge-train readiness marker cites one verdict record per reviewer, each bound to the PR's head
sha. A fabricated or stale verdict can then be detected mechanically, rather than trusted as the
marking session's attestation.

## Context
- SPEC-0027 section 2, "Trust (owner decision, v1)": the marker's verdicts are an attestation, not
  a verification. Fabricated readiness by a buggy or prompt-injected worker session is an accepted
  residual risk, unchanged from today's flow. This task is the named v2 follow-up.
- Directions to weigh in a short design note first:
  - each reviewer subagent posts its own verdict comment, naming the head sha and a digest of the
    reviewed diff;
  - a signed or otherwise tamper-evident record;
  - the merger checks that each record's sha equals the marker's head and that the diff digest
    matches `git diff M0...H`.
- If the design needs more than size M, file a spec task instead.

## Acceptance criteria
- [ ] A design note, or a SPEC-0027 amendment, chooses the record format and says what it does and
      does not prevent, given that all sessions share one GitHub identity.
- [ ] `merge_ready.py verify` checks the records. V4 becomes a verification, with tests.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Changing the reviewer agents' review criteria.
