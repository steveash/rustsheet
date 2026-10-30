---
id: T-2997
title: "Per-reviewer verdict records bound to the head sha for merge-train readiness"
milestone: M2
lane: harness
status: done
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
- [x] A design note, or a SPEC-0027 amendment, chooses the record format and says what it does and
      does not prevent, given that all sessions share one GitHub identity.
- [ ] ~~`merge_ready.py verify` checks the records. V4 becomes a verification, with tests.~~ Moved to T-3207
      (implementation) and T-3208 (enforcement): the work is more than size M (see Outcome).
- [x] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Changing the reviewer agents' review criteria.

## Outcome
- The design is a SPEC-0027 section 2 amendment, "Verdict records (v2)", recorded in decision 0178.
  - Each reviewer gets one PR comment: the reviewer's report plus a `merge-train-verdict v1` block naming the head
    it read and its verdict.
  - `mark` derives the marker's verdicts from those records, and V4 checks the same records.
  - A diff digest was weighed and dropped, because the head sha already fixes the reviewed diff.
  - It is rolled out through `verdict_records = "warn" | "require"`.
- What it does and does not prevent, given that every session shares one GitHub identity: it stops stale panels,
  verdicts with no review behind them, and silent fabrication (the report becomes a visible, auditable comment). It does
  not stop a session that deliberately fabricates, which stays the accepted residual risk.
- The implementation needs more than one size-M change, so this PR is the spec the task's rule asks for, and the
  work is split. T-3207 covers the format,
  `make verdict-record`, `mark` and V4 in `warn` mode, with tests. T-3208 covers the skills and the flip to
  `require`. The acceptance criterion "verify checks the records" moves to T-3207, and V4 stays an attestation
  until T-3208.
- Eval delta: none (docs only).
