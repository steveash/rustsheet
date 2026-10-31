---
id: T-3440
title: "Move the parser's Node, ArrayNode and completion types out of expressions/parser/mod.rs into parser/node.rs (the file is at 991 of its 1,000 lines)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/expressions/parser/mod.rs` has room under the 1,000-line file limit again
(`scripts/check_file_size.py`). The parse-tree type definitions move, unchanged, into a new `parser/node.rs`. Then
the next parser fix (T-3338's sign cap, T-3102 and T-3212's R1C1 entry, T-3262's typed ANCHORARRAY) does not first
have to split the file.

## Context
Planner run 2026-10-31 (`make eval` at origin/main 4c8a1a20). mod.rs is at 991 lines (`wc -l`), 9 under the limit.
It is not in quality/oversized-files.json, so the gate fails any PR that takes it past 1,000. T-3377 and T-3313 set
the pattern for the same problem in other files.

What moves (line numbers at 4c8a1a20):
- `ArrayNode` (l. 114) and `NamedVariable` (l. 135);
- `ExpectedTokens` (l. 142) and `CompletionContext` (l. 155);
- `Node` (l. 166-362), the `NODE_SIZE` constant and its size assertion (l. 363-366), and `impl Node` (l. 368 to the
  line before `pub struct Parser`).
All of them keep their doc and line comments (the SPEC-0007 and SPEC-0016 notes).

`Parser`, `new_parser_english`, `Reference`, `DefinedNameS`, `get_table_column_by_name` and the module list stay in
mod.rs. mod.rs re-exports the moved public types (`pub use node::{ArrayNode, CompletionContext, ExpectedTokens,
NamedVariable, Node};`), so `crate::expressions::parser::Node` and every other existing path, inside the engine, in
`crates/rustsheet` and in `bindings/python`, still resolves with no edit at the use sites. Items that `impl Node`
calls from mod.rs's sibling modules are imported, never copied (directive 3).

## Acceptance criteria
- [ ] The items above live in `expressions/parser/node.rs`, with bodies unchanged (a reviewer can diff them line by
      line); mod.rs declares `mod node;` and re-exports them.
- [ ] expressions/parser/mod.rs is at most 600 lines; node.rs is under 1,000.
- [ ] No use site outside `expressions/parser/` changes (`git diff --stat` shows only mod.rs and node.rs, plus imports
      inside `expressions/parser/` if a sibling module named a moved item by a private path).
- [ ] `make eval-check` shows no cell changed (0 fixed, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to a `Node` variant, its size or its methods.
- Splitting `impl Parser` (its methods are already spread over the sibling modules).
