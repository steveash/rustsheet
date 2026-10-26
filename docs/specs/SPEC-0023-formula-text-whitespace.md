# SPEC-0023 — Typed whitespace in the parsed formula

- **Status:** accepted (on merge of T-2776; architecture reviewer approved)
- **Milestone / lane:** M2 / engine
- **Tasks:** T-2776 (this spec); T-2781 (step 1), T-2782 (2), T-2783 (3), T-2784 (4); T-2785 (a parse
  bug the probes found, independent); T-2786 (probes of what is still open)

## Problem
Excel keeps most of the whitespace a formula is typed with. FORMULATEXT prints it back, and so does
the formula bar. The engine's `Node` tree keeps none, and the printer (`stringify.rs`) writes
canonical, space-free text. Every value matches, but the formula text does not:

- probes/t2769-formula-text-whitespace: 18 FORMULATEXT cells fail (C1:C5, C7, C8, C10:C15, C17:C20,
  C22). Examples: `=1 + 2`, `=SUM( 1 )` and `=- 5` print as `=1+2`, `=SUM(1)` and `=-5`.
- probes/t2723-unary-plus-formula-text C24 and C25 fail: `=+ 5` and `=+ "a"` print as `=+5` and `=+"a"`.
- probes/t2776-formula-text-whitespace-shapes: 24 cells fail (C1:C3, C5, C7:C13, C16:C22, E1, E2,
  E6, E9, E10, F21).
- probes/t2776-ws-after-at: C1 fails.
- probes/t2776-ws-after-bang: A1 and C1 fail. The value cell fails too: `=S! B1` is a parse error in
  ours, while Excel reads it as `S!B1`. T-2785 fixes that one separately.

SPEC-0007 (written parentheses) listed whitespace as a non-goal. This spec picks it up, using the
same approach: a transparent wrapper node.

## Goals / non-goals
Goals:
1. Whitespace that Excel keeps survives parse → print, in every printer:
   - `to_english_string`, `to_localized_string`, `to_rc_format`, `to_string_displaced` and
     `to_string_moved`;
   - the xlsx export text, Excel saves kept whitespace, a line feed as CR LF (T-2786, xlsx-storage.md).
2. Whitespace that Excel drops is dropped at parse time, so the tree only ever holds what prints.
3. No value changes. Whitespace is transparent to evaluation and to every shape test, as a `Group`
   is.
4. `size_of::<Node>()` does not grow (`NODE_SIZE`). A formula with no whitespace allocates nothing
   new.

Non-goals:
- Whitespace that is itself an operator: the single space of an intersection. The parser already
  reads it (`whitespace_follows`, `separator`). Step 3 keeps only the extra spaces beside it.
- Separators Excel refuses: CR, TAB and NBSP (references.md "Intersection operator: separators").
  They stay parse errors.
- Localised printers: the kept whitespace prints the same in every locale.

## Excel behavior to match
Source: docs/excel-semantics/syntax.md, "Formula text keeps typed whitespace, except at the end,
before a comma and in an array constant". The evidence is probes/t2769-formula-text-whitespace and
probes/t2776-* (excel-graph, 2026-10-26).

- **Kept**, typed: whitespace in these positions.
  - around a binary operator, including `&`, `=` and `*`;
  - after `=`;
  - after a unary sign (`=- 5`, `=-  -5`);
  - before `%`;
  - after `(` and before `)` of a call or a group;
  - after a comma, and inside `LET`;
  - after `@` (`=@ B1:B2`);
  - the spaces beyond the operator in an intersection (`=B1:B2  B2`);
  - a line feed after `=`.
- **Dropped**: whitespace in these positions.
  - at the end of the formula, whether spaces or line feeds;
  - before a comma;
  - inside an array constant;
  - around `:` (`=SUM(B1 :B2)` prints `=SUM(B1:B2)`);
  - after a sheet prefix's `!`.
- **Folded plus**: a plus that folds into a number literal leaves its spaces before it in place. So
  `=  +5` prints `=  5`, and `=1+  +5` prints `=1+  5`.
- **A run is a line-feed count plus a space count.** Only the line feeds and the spaces after the
  last line feed print. `=1+ ` LF ` 2` prints `=1+` LF ` 2`, and 10 spaces, 3 line feeds and 2 spaces
  print as the 3 line feeds and the 2 spaces. The space count is one byte and wraps: 300 typed
  spaces print as 44. This matches the BIFF `PtgAttrSpace` record, which stores one count per kind.
  The behaviour comes from the probes; the record only explains it.
- **Stored text**: a formula read from a file's `<f>` keeps its interior whitespace (`1 + 2`,
  `SUM( 1 )`, `1+` LF `2`, `(1 )+2`) and applies the typed drops. Whitespace at the very start is
  also dropped: `<f>  +5</f>` prints `=5`, and `<f> 1+2</f>` prints `=1+2`.
- **Refused** (Graph answers 400 InvalidArgument, T-2786): `=SUM (1)`, a space between a
  function name and its `(`. What ours does with it is untested
  (`operands.rs` reads a call only when no whitespace precedes the `(`); probes/t2786-whitespace-structured-refs-runs.
- **Values**: every probed value equals the value of the formula without spaces.
- **Answered** (T-2786, syntax.md "Whitespace beside a missing argument is kept"): a run beside a missing operand is
  kept as typed (`=NOW( )`, `=IF(1, ,2)`, `=SUM(1, )`, `=SUM(, 1)`), unlike a run before a comma that follows an
  operand, so the rule above "a run followed by `,` ... is dropped" holds only after an operand (T-2799). `=SUM (1)` and
  `=SUM( )` are refused. Structured references normalize their whitespace (T-2801). Excel saves kept whitespace in the
  `<f>` text, a line feed as CR LF (xlsx-storage.md), so export prints the tree's whitespace and does not strip it.

## Design
### Where whitespace attaches
Every kept run sits between two tokens. The parser gives it to the nearest **primary**, which is
what `parse_primary` returns: a literal, reference, call, group, array, name, or a unary or `@`
applied to one.
- A run before a primary's first token is that primary's `before`. This covers the run after `=`,
  after a binary operator, after a unary sign, after `(`, after a comma, and after `@`.
- A run after a primary's last token, followed by a binary operator, `%` or `)`, is that primary's
  `after`.
- A run followed by `,`, by the end of the formula, or found inside `{…}` is dropped. (A run beside a missing argument is kept: see Unknown/Answered below, T-2799.) These are
  exactly the places where Excel drops it (for the probed shapes; see Unknown).

Attaching at the primary level, and never to a chain, leaves the value-operator chains of SPEC-0007
untouched. Take `=1*2 + 3`. The run before `+` becomes `after` of the leaf `2`, not of the chain
`1*2`, so `value_operands`, `left_chain` and `grow_chain` see the same spine as today.

### The node
```rust
/// Whitespace written around `child`, printed back as typed: `before` ahead of its first token,
/// `after` behind its last (docs/excel-semantics/syntax.md, SPEC-0023). Transparent to
/// evaluation and analysis, as a `Group` is (see `Node::ungrouped`).
Spaced { space: Box<Spacing>, child: Box<Node> },

/// One run before and one after an operand, in Excel's stored shape.
pub(crate) struct Spacing { pub before: Run, pub after: Run }

/// Excel's whitespace run: `line_feeds` line feeds, then `spaces` spaces.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Run { pub line_feeds: u8, pub spaces: u8 }
```
- **Size.** Two boxes, 16 B, is smaller than the largest variant, so `NODE_SIZE` holds. A `Spaced`
  node exists only where whitespace was written.
- **`Run` is Excel's shape.** Parsing a run into `Run` performs the normalisation the probes show:
  count the line feeds, keep the spaces after the last one, and wrap the space count at 256 (the
  count mod 256, never a panic). Whether the line-feed count wraps too is unprobed: step 2 probes
  it first and does whatever that shows. Printing a `Run` is `"\n" × line_feeds + " " × spaces`. Step 1
  keeps only the space count of a run without line feeds; step 2 adds the line-feed rules.
- **Parser.** `parse_primary` reads the run before it from the lexer. The binary-operator, `%` and
  `)` sites read the run after the operand they just finished. The lexer already sees the
  whitespace (`whitespace_follows`, `separator`), so it only needs one accessor that returns the run
  it skipped before the current token. Nesting is unchanged: a `Spaced` adds one node and no
  `PRIMARY_DEPTH` charge, so the depth accounting (T-1169) holds.

### Seeing through it
`Node::ungrouped` and `Node::lowered` peel `Spaced` in the same loop that peels `Group` and
`UnaryPlus`, so it adds no evaluator frame. Every shape-matching site that peels a `Group` peels a
`Spaced` with it. The SPEC-0007 checklist (parser predicates, the reference-context dispatch, the
`ReferenceKind`/`RangeKind` argument tests) is the same list, behind the same accessor. The walkers
(static analysis, renames, structural edits, `prefix_bound_variables`) recurse into `child`.

### Printing
`Spaced` prints `before`, then the child, then `after`, in the shared `stringify` walker and in
`to_string_moved`. In the precedence tables it is transparent: the printer decides wrapping on the
peeled child. A tree the parser built never needs a new wrap, because whitespace never changes a
parse.

### Sharing, equality and storage
- Formulas are pooled by their R1C1 text (`to_rc_format`). Two cells that differ only in whitespace
  therefore get two pool entries, as two parenthesisings do under SPEC-0007, and they print
  differently. A fill copies the text, so a filled column still shares one tree.
- `Node: PartialEq` distinguishes `1+2` from `1 + 2`. That is correct for the same reason.
- Structural edits (row or column insert and delete, sheet rename, cut/paste) rewrite references
  inside a `Spaced` child like anywhere else. They never add or remove whitespace.
- **Stored text (step 4).** Import parses `<f>` with the same parser, so interior whitespace comes
  for free from step 1. Step 4 drops the leading run of a stored formula and keeps the typed leading
  run (`= 1+2`), as the probe shows.

### What does not change
Public API, the depth and operator caps, every parse error, and every value.

## Alternatives considered
- **A trivia field on every node** (`before`/`after` on each variant). This grows `Node` by at least
  16 B and touches every constructor and pattern. Rejected for the reason SPEC-0007 rejected a
  `written` count.
- **Keep the typed text and re-print from it.** Structural edits rewrite references, so the text
  would need token-level patching, a second source of truth. Rejected, as in SPEC-0007.
- **A side table of runs by token index**, like Excel's own RPN with `PtgAttrSpace` records. Our
  tree has no token stream. Every printer would have to count tokens in step with the parser, and a
  `#REF!` rewrite changes token counts. Rejected.
- **Whitespace fields on `Group` and the call node only.** This covers `SUM( 1 )` and `(1 )` but not
  `1 + 2`, `- 5` or `@ B1`. Every probe shape would need another field. Rejected.
- **Attaching runs to chain nodes.** This is closer to "around the operator", but it makes the
  SPEC-0007 chain spine non-uniform, so every chain walker would need to peel. Rejected in favour of
  attaching at the primary.

## Test plan
- **Probes**: every failing cell listed under Problem leaves the baseline in the step that owns it,
  with 0 regressions.
- **Unit tests**, in the parser tests:
  - `print(parse(f)) == f` for each probed typed shape;
  - `eval(parse(f)) == eval(parse(strip_ws(f)))`;
  - `clone()` round-trips a `Spaced` operand inside a chain (`=1*2 + 3`).
- **Structural edits**: a row insert, a sheet rename and a cut/paste on `=Sheet1!A1 + 1` keep the
  spaces.
- **Depth**: the T-1169 tests (125, 126 and 127 nested parentheses) stay green, and they gain a
  `( ( … ) )` shape with spaces at every level.
- **Size**: the `NODE_SIZE` const assertion stays unchanged.
- **Run normalisation (step 2)**: a unit test per probe cell (C19:C22), plus 255, 256 and 257
  spaces. The wrap must not panic. Step 2 probes exactly 256 spaces first, because the byte reading
  predicts none would print, and that is unverified.

## Rollout
Each step is one S/M task. Each leaves `make check` green and moves only the cells named in it.

1. **T-2781 — the `Spaced` node** (engine, opus, M). Covers the node, `Spacing`/`Run` (spaces only),
   the parser attach sites, the peel in `ungrouped`/`lowered` and the shape sites, the printers and
   the walkers. Fixes:
   - t2769 C1:C5, C7, C8, C10:C15, C17:C20 and C22, except the line-feed cells step 2 takes;
   - t2723 C24 and C25;
   - t2776-shapes C1, C2, C7:C13 and C16:C18;
   - t2776-ws-after-at C1.
   Runs that contain a line feed may print approximately until step 2.
2. **T-2782 — run normalisation** (engine, sonnet, S). Line feeds, then the spaces after the last
   one, the space count wrapping at 256 (the line-feed count as step 2's probe shows). Fixes t2776-shapes C19:C22 and F21, plus any t2769 line-feed
   cell left over.
3. **T-2783 — intersection spaces** (engine, sonnet, S). Keeps the spaces beyond the operator
   (t2776-shapes C3 and C5, and E9 together with step 4). The intersection node prints its
   `separator()` run instead of one space.
4. **T-2784 — stored text** (engine, sonnet, S). Import drops the leading run and keeps the
   interior runs. Fixes t2776-shapes E1, E2, E6 and E10.
5. **T-2785 — `=S! B1`** (engine, sonnet, S). Independent of the steps above: whitespace after a
   sheet prefix's `!` is skipped (t2776-ws-after-bang A1, C1).
6. **T-2786 — probes (done)** (research, sonnet, S). Whitespace inside structured references, and the
   `<f>` text Excel saves. Files the export or structured-reference task if the answers differ from
   what step 1 prints.
