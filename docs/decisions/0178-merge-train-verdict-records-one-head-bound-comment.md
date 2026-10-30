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
  GitHub's 65,536-character limit. `scripts/merge_flip.py` now reads the switch path from `merge_ready.SWITCH`
  (one definition).
