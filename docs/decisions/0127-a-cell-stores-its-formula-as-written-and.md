# 0127 — A cell stores its formula as written and stripped of whitespace

- **Status:** accepted (2026-10-26, T-2781)
- **Context:** SPEC-0023 keeps typed whitespace as a transparent `Node::Spaced`. About 300 sites in the evaluator
  and the functions match an argument's shape without peeling wrappers (LET name positions, `ISOMITTED`, reference
  argument tests), so a `Spaced` reaching them changed values (`=LET(x, A2, y, B5, OFFSET(x, 0, 0)+y)` was
  `#VALUE!` in T-2781's first attempt). See SPEC-0023 "Implementation note".
- **Decision:** a cell's shared formula is stored as `formula_cells::ParsedFormula`: the tree with every `Spaced`
  removed, which evaluation, units and static analysis read and nothing else; its static result; and, only when the
  formula holds whitespace, the tree as written, which only printers and edits read (`written_formula`). Other
  evaluated trees (defined names, conditional formats, outside-cell formulas) are stripped where they are parsed.
- **Consequences:** new evaluation code never needs to peel `Spaced`; new printing or edit code must read
  `written_formula`, or the formula loses its spaces. A whitespace-written formula costs a second tree. The xlsx
  export reads the stripped tree until T-2786 decides what Excel saves. T-2799 names the tuple's fields.
