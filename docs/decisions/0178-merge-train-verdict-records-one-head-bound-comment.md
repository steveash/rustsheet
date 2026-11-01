# 0178 — Merge-train verdict records: one head-bound comment per reviewer, rolled out warn then require

- **Status:** accepted (2026-10-30, T-2997). It amends SPEC-0027 section 2 ("Verdict records (v2)").
- **Context:** SPEC-0027 v1 treats the marker's verdicts as the marking session's attestation (V4). T-2997 asked
  for verdict records that make a fabricated or stale verdict detectable. All sessions and their reviewer
  subagents share the owner's GitHub account and machine, and reviewers have no GitHub write tools, so the marking
  session posts every record and could forge one; a signing key would be readable by the same session.
- **Decision:**
  - Each reviewer's verdict becomes one PR comment, posted by the marking session right after the reviewer returns:
    the report verbatim, then a `merge-train-verdict v1` block naming the head the reviewer read, the reviewer and
    the verdict. No diff digest: the head sha already fixes the reviewed diff.
  - `mark` derives the marker's `reviewers:` from the newest writer records for the head and refuses without them;
    V4 checks the same records (for `prev` under a verified carry-over).
  - Only unedited writer records count, newest by `created_at`, re-read at every V4 check; only the block that ends a
    body counts, and `verdict-record` neutralises comment markup in the report and never truncates the block.
  - Once in `require` mode, records stop accidental stale panels (against a buggy session; `head:` is self-declared), verdicts with no review behind
    them, and silent fabrication (the report is a visible, auditable comment, not a tamper-proof one). They do not
    stop a deliberately fabricating session; that residual risk stays accepted.
  - Rollout through `verdict_records = "warn" | "require"` in `scripts/merge_train.toml`, `warn` first.
- **Consequences:** the implementation needs more than one size-M change, so it is split: T-3207 (the record
  format, `make verdict-record`, `mark` deriving verdicts, V4 in `warn` mode, tests) and T-3208 (the review and
  work-loop skills post records; flip to `require` after a week of clean `warn` data). Until T-3208 flips the key,
  V4 stays an attestation.
- **Implementation (T-3207, 2026-10-30):** `scripts/merge_ready.py` holds the record format (`Record`,
  `record_body`, `parse_record`), the lookup (`recorded_verdicts`, `record_gaps`) and the rollout key
  (`records_mode`); `make verdict-record` posts a record (`record`, direct or `STEP=1`), `mark` derives its
  verdicts from the records (`REVIEW=` and `CARRY=1` stand in for a missing record only in `warn`), and V4
  checks them, re-running the carry check under `prev:`. Choices this record did not fix: a missing
  `verdict_records` key is `warn` and an unparseable or unknown value `require`; a comment without a
  `created_at` counts as edited; a record body leaves room for the session's attribution footer within
  GitHub's 65,536-character limit. Real-payload check (PR #2419, comment 6521609074, 2026-10-30): an
  `eyes` reaction left `updated_at` at `created_at` (11:32:02Z); a body edit moved it to 11:32:36Z. So only an
  edit drops a record; if GitHub ever moves `updated_at` for anything else, that record stops counting
  (fails closed: a missing record, never a wrong verdict). `scripts/merge_flip.py` now reads the switch path from `merge_ready.SWITCH`
  (one definition).
- **Skills (T-3208, 2026-10-30):** each reviewer agent's report starts with `HEAD: <git rev-parse HEAD>`; the
  `review` skill posts each report with `make verdict-record` as the reviewer returns (a `BLOCK` recorded as
  `REQUEST_CHANGES`; nothing posted when HEAD moved past the reviewed commit), so the work-loop opens its PR as a
  draft before the first review round; the work-loop and steward skills mark with `make mark-ready PR=<n>`, with
  `REVIEW=` only as the `warn`-mode stand-in. The flip to `require` moved to its own task, T-3243 (blocked until
  2026-11-07, a week after the skills land), because the week of `warn` data cannot fit in one session.
