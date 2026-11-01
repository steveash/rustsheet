---
id: T-3423
title: "Fuzz target external_diffs: crafted diff bytes through UserModel::apply_external_diffs never panic"
milestone: M12
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2896]
---

## Goal
A cargo-fuzz target `external_diffs` applies arbitrary bytes to a small model through `UserModel::apply_external_diffs`
and then runs `evaluate()`, a few edits and a `to_bytes`/`from_bytes` round trip, without panicking. A refused list
leaves the workbook equal to what it was before.

## Context
T-2896 found many more failure shapes than its task named: unchecked indices, `i32` overflow in `row + count` in the
structural actions, and undo matrices shorter than their area. Those were fixed one probe case at a time
(crates/rustsheet-engine/src/test/user_model/test_external_diffs_untrusted.rs), so a fuzzer is the way to find the
ones still left. Follow fuzz/fuzz_targets/xlsx_import.rs and T-2895's bitcode_workbook target for layout and
registration. Seed the corpus with real `flush_send_queue()` output covering edits, undo, sheet operations, styles and
merges, because random bytes rarely decode to a `Vec<QueueDiffs>`. `Diff` is `pub(crate)`, so the target only sees the
bytes, which is what a peer sends.

## Acceptance criteria
- [ ] `fuzz/fuzz_targets/external_diffs.rs` exists and is registered, with a seed corpus of real send queues.
- [ ] A local run of at least 10 minutes finds no crash, or each crash found is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
