# 0168 — A deviations line can pin a whole case the engine refuses

- **Status:** accepted (2026-10-30, T-3165)
- **Context:**
  - T-3097 (#2312) probed Excel's nesting limits. Excel accepts 128 and 255 bare nested parentheses
    and 64 and 65 levels of `(ABS(`; RustSheet refuses all eight cases (`probes/t3097-nesting-paren-128`,
    `-255`, `-abs-64`, `-abs-65`, stored and typed) with "Formula is nested too deeply", its
    directive-5 depth bound (`MAX_FORMULA_DEPTH`, docs/excel-semantics/syntax.md "Formula nesting
    depth cap", Status: DEVIATION, decision 0014).
  - Measured for T-3165 (release build, 1 MiB thread, the size the bound is set for): the parser
    alone overflows the stack at about 292 nested parentheses (about 3.5 KB of stack per level, every
    parenthesis passing through the whole precedence chain), 180 nested `ABS(` and 112 nested
    `(ABS(`. Excel's formula length limit admits about 4,000 nested parentheses, so no constant
    bound over today's recursive parser matches Excel, and raising the bound to 255 would leave
    almost no stack for evaluation.
  - The spec adapter enters each formula through the API, so a refused formula fails the case as
    a whole (`EngineError`, no cell scored). A decision-0036 `deviations` line pins a scored
    cell, so it cannot pin these cases: they stay unowned known failures on the M2 exit row.
- **Options considered:**
  - *(a) A whole-case line*: the cell field is `*` and the pinned value names the refusal. Chosen:
    the refusal stays loud for every other case, and a pinned case that starts loading reads stale.
  - *(b) The spec adapter leaves a refused cell empty* (as the API does) so the cells score and
    per-cell lines pin them. Rejected: every refused input of every case would then score quietly
    instead of failing the case, a harness-wide policy change for eight cases.
  - *(c) Lift the bound* by making a parenthesis cost less parser stack. Not rejected, but a parser
    restructure (a nested group would have to skip the precedence chain) that this decision does not
    wait for (T-3191); lifting it makes the lines stale, and the stale report says so.
- **Decision:** In a suite's `deviations` file (decision 0036) a line whose cell field is `*` pins
  the whole case; its pinned value reads `refused "<message>"`. When the engine refuses the case
  with an error containing `<message>`, every scored cell of the case is a deviation (its actual
  value reads `<refused>`); a refusal with any other message is an `EngineError` as before; a case
  that loads scores as usual, with every cell listed, so a cell that matches Excel shows as a stale
  line. The line must still cite decision 0014 and an accepted `Status: DEVIATION` entry (decision
  0052).
- **Consequences:** `crates/rustsheet-eval/src/deviations.rs` parses the new line and
  `runner.rs` scores it; reports, milestone lists and `scripts/exit_owners.py` see ordinary
  deviation cells and need no change. The eight T-3097 cases leave the probes baseline as passes
  with deviations. A whole-case line is for a resource-limit refusal only; any other refusal stays a
  known failure. The match is on the case's error text, not on a cell: a case pinned this way
  should hold one formula that meets the limit, as the eight T-3097 cases do.
