# 0173 — Lift the depth bound for runs of directly nested parentheses

- **Status:** accepted (2026-10-30, T-3191)
- **Context:** Excel loads 128, 255 and 256 directly nested parentheses around an operand and
  refuses 257 (probes/t3097-nesting-paren-*, probes/t3172-paren-256; docs/excel-semantics/syntax.md
  "Formula nesting depth cap"). RustSheet refused 128 or more as "Formula is nested too deeply":
  every `(` recursed through the whole precedence chain (about 3.5 KB of release stack each,
  decision 0168), and the depth bound charged it an operand's two levels although `((x))` is one
  `Group` node. Decision 0168 pinned the T-3097 cases whole until a parser change (T-3191) lands.
- **Decision:** A run of `(` with nothing between them is parsed in a loop
  (`Parser::parse_parenthesis_run`): the innermost group first, then each enclosing one with the
  group inside handed to its content as its first operand. A group whose content is exactly the
  group inside merges with it and adds no depth; any other costs an operand's levels, as before.
  More than 256 groups open at once (`MAX_PARENTHESES`, Excel's own limit for a run) is too deep,
  whether in one run or split by whitespace, an operand or a call; Excel was probed with an
  unbroken run only, so counting split runs together is the conservative reading. Parentheses separated
  by whitespace, calls and every other shape keep the recursive path and its bound.
- **Consequences:** The pinned lines for t3097-nesting-paren-128 and -255 (stored and typed) and
  t3172-paren-256 leave `eval/suites/probes/deviations`; those cases now match Excel. `Group.levels`
  widens to `u16`. The `(ABS(` cases (t3097-nesting-paren-abs-64 and -65) stay pinned: each level is
  a call inside a group, which still recurses; T-3197 owns them. The at-limit tests on a 1 MiB
  release stack and a 2 MiB debug stack cover a run of 256 and the refusal of 257.
