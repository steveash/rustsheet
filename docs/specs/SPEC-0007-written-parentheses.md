# SPEC-0007 — Written parentheses in the parsed formula

- **Status:** accepted (on merge of T-1244; architecture and performance reviewers approved)
- **Milestone / lane:** M1 / engine
- **Tasks:** T-1244 (this spec); T-1339 (step 0), T-1340 (1), T-1341 (2), T-1342 (3), T-1343 (4), T-1344 (5); T-1331 (its probe case is on its branch; T-1340 merges it and fixes its cells, then T-1331 closes as done by T-1340)

## Problem
Excel keeps every parenthesis a formula was written with. The engine's `Node` tree drops them
except in five special cases. Its printer then re-derives parentheses from operator precedence.
So the engine's formula text and FORMULATEXT differ from Excel even though every value matches:

- probes/at-operand-nested-groups: 14 FORMULATEXT cells differ (`=A1+(B1)` prints `=A1+B1`,
  `=-(A1)` prints `=-A1`, `=@(A1+(B1))` prints `=@(A1+B1)`). All 18 values match
  (docs/excel-semantics/syntax.md "FORMULATEXT keeps every written parenthesis").
- probes/at-group-leftmost-shapes E34, E35 and probes/at-group-range-corner-whole E34
  (`=@((A1 B1) C1) D1`, `=@(A1):(B1) C1`, `=@(A1):(B1)`). `LeadGroup` records only one group, so these
  print wrong (T-1251).
- probes/power-chain-text I11, I26, I41, I46 (T-1331; the case and its rule are on T-1331's branch, PR draft, and land with T-1340). `=A1^2^3` and `=(A1^2)^3` parse to the same
  `Node`, and Excel prints each as written. No printer-only rule can get both right.
- The T-1167 unit test `at_operand_adds_no_level_for_a_nested_inner_group` (E40, E41, E43, E44) and
  the F50, F51 asserts of `at_operand_group_leftmost_shapes_match_the_probe` pin the engine's
  current, non-Excel text.

The five special cases each have their own representation and parser bookkeeping:

| Case | Today |
|---|---|
| Whole formula in parentheses | `GroupedRootKind`, one node per level |
| A union's parentheses | `OpUnionKind.written: usize` |
| A cut of a written union | `OpIntersectKind.written: usize` (T-1176) |
| Parentheses around a whole `@` operand, its lead operand, or the whole `@` | `ImplicitIntersection { written: u8, lead: LeadGroup, grouped: u8 }` |
| Parser bookkeeping | `at_paren`, `at_group`, `leading_paren`, `grouped_root`, `root_group` |

The cut/paste printer (`to_string_moved`, move_formula.rs) also has its own precedence table that
disagrees with `stringify`. For example, its Power arm never wraps an operand.

## Goals / non-goals
Goals:
1. Every written parenthesis survives parse → print, in `to_english_string`, `to_localized_string`,
   `to_rc_format`, `to_string_displaced`, the moved text (`to_string_moved`) and the xlsx export
   text. So FORMULATEXT and the stored file text match Excel.
2. One representation, a `Group` node, replaces the five special cases and the parser bookkeeping.
3. Evaluation results do not change. A group is transparent to every evaluation and reference
   path. The one existing exception is kept: a parenthesized formula root skips the near-zero snap
   (`evaluate_formula_root`).
4. No new resource risk: tree depth and parse cost stay under the existing caps (T-1169).

Non-goals:
- Whitespace preservation. This is a separate question, and Excel's handling of spaces is
  partly known (references.md).
- Typed-input normalisation. Excel may drop or add parentheses when a user types a formula; we
  model the stored text. The Graph range API returns the typed text unchanged (see below), so no
  difference is known.
- New semantics for groups beyond what the parser already decides: a union comma only inside
  `(`, and the `(A1)(2)` call form. Those stay parse-time decisions.

## Excel behavior to match
- Every redundant parenthesis is printed back, at every level and in every probed shape: plain
  operators, unary minus, intersection, function arguments, `@` operands, unions and cuts.
  Evidence: probes/at-operand-nested-groups (36 cells, excel-graph, 2026-10-09, T-1178);
  syntax.md; references.md (union, cut and `@` group entries).
- A `^` chain prints as written: `=A1^2^3` stays bare and `=(A1^2)^3` keeps its group.
  Evidence: probes/power-chain-text (excel-graph, T-1331; on T-1331's branch until T-1340 merges it).
- **The stored xlsx text keeps them too.** Formulas entered through the Graph range API and
  saved by Excel for the web (excel-graph, 2026-10-11, T-1244) stored these `<f>` texts:

  | Entered | Stored `<f>` |
  |---|---|
  | `=Z1+(Z2)` | `Z1+(Z2)` |
  | `=((Z1))` | `((Z1))` |
  | `=-(Z1)` | `-(Z1)` |
  | `=IF((Z1>0),(Z2),Z3)` | `IF((Z1&gt;0),(Z2),Z3)` |
  | `=(Z1^2)^3` | `(Z1^2)^3` |
  | `=SUM((Z1:Z2))` | `SUM((Z1:Z2))` |
  | `=(1)+((2))` | `(1)+((2))` |
  | `=@(Z1+(Z2))` | `_xlfn.SINGLE((Z1+(Z2)))` |

  The range API read the formulas back as entered. So `@` with a written group exports as
  `_xlfn.SINGLE(` + the operand with its group + `)`. The engine exports `_xlfn.SINGLE(Z1+Z2)` today.
  This is recorded in xlsx-storage.md by this task.
- Values: groups never change a value in any probed cell (all 18 at-operand-nested-groups values
  match). The parse-time effects are already implemented and stay as they are:
  - the union comma (references.md);
  - `(A1)(2)` as a call (references.md "A parenthesized reference called");
  - `(1)` being a constant in an intersection (references.md);
  - the root snap (`GroupedRootKind`, arithmetic.rs).
- Reference context (probes/written-parens-reference-context, excel-graph, 2026-10-11, T-1339):
  groups are transparent in every probed cell. `ISREF((A1))`, `ROWS(((A1:A3)))`, `SUM(((A1,B1)))`,
  `LET(x,(A1:A3),ROWS(x))` and `ANCHORARRAY(@(IF(TRUE,D1)))` equal the forms without the group (the last without `@(...)`), and
  FORMULATEXT prints the groups. The only value a group changes is the root snap (`=(0.1+0.2-0.3)` and
  `=((0.1+0.2-0.3))` are 5.55e-17, `=0.1+0.2-0.3` is 0), which is the parse-time effect listed above. No
  design amendment is needed. T-1364 (probes/written-parens-reference-only) also found groups transparent for
  reference-only checks: `ISREF((A1:A3))`, `ISREF(LET(x,(A1:A3),x))`, `OFFSET((A1:A3),1,0)`, `CELL("address",(A1))`,
  `ANCHORARRAY(@IF(TRUE,D1))` and `ANCHORARRAY((INDEX(D1:D3,1)))` (`)#` shape) match the ungrouped forms; no
  non-transparent cell. T-1340 keeps the existing parse-time handling of a group before `#` and does not change it.

## Design
### The node
```rust
/// Parentheses written around `child`: `levels` directly nested pairs, `((x))` = 2.
/// Transparent to evaluation (see `Node::ungrouped`); printed back as written.
Group { levels: u8, child: Box<Node> },
```
- **One node per run of directly nested parentheses.** `((A1))` is one `Group { levels: 2 }`.
  `((A1)+(B1))` is `Group{1, Sum(Group{1,A1}, Group{1,B1})}`. This is the same "span is exactly
  `open+1..close`" rule `root_group` uses today (reference_operators.rs:79-85).
- **Memory.** `Box<Node>` + `u8` is smaller than the largest variant (`ParseErrorKind`, about
  80 B), so `size_of::<Node>()` does not grow. Rollout step 1 adds a const size assertion, which
  pins today's size so later variants cannot grow it unnoticed. The cost is one allocation per
  run of written parentheses. Formulas without parentheses allocate nothing new.
- **`levels` bound.** The parser charges `PRIMARY_DEPTH` (2) per written `(` as today
  (depth.rs). So at most 127 levels reach a node, and `u8` holds that. The parser saturates rather
  than wraps (`levels.saturating_add(1)`), and the depth cap fires first in any case.
- **Tree depth.** A run of k levels adds one node but was charged 2k levels. So the recursion
  depth of every tree walker stays under `MAX_FORMULA_DEPTH`. The T-1169 tests (125, 126 and 127
  nested parentheses: parse, eval, print, row insert, sheet rename) must keep passing unchanged.

### Seeing through groups
```rust
impl Node {
    /// The node under any written parentheses: `((A1))` → `A1`. Loops, never recurses.
    pub(crate) fn ungrouped(&self) -> &Node;
}
```
A group is transparent to evaluation. Every place that matches on an operand's or argument's
*shape*, not its value, sees through it.

**Peel where operands are dispatched, not per site.** `evaluate_node_in_context`,
`evaluate_node_with_reference`, `evaluate_reference_arg`, `eval_ref_or_index`,
`spill_anchor_operand` and `range_operand` start with `let node = node.ungrouped();`.
- This is a loop, not a recursive arm, so a group adds no evaluator frame. `-(-(…))` and
  `1*(1*(…))` keep today's stack depth.
- Function code that tests argument shapes on `args[i]` gets the peeled node from one accessor
  (for example `Node::arg_shape(&self) -> &Node`, an alias of `ungrouped`). It does not call
  `ungrouped()` ad hoc.

The checklist is **every match on any `Node` variant applied to an operand or argument**, not
only the group-related variants. Step 1 greps `Node::[A-Z]\w*Kind` / `matches!(` over functions/,
model.rs, reference_operators.rs, units.rs and the parser predicates, and lists every site in its
PR with "peeled" or "not an operand". Known sites:
- **Parser predicates:** `is_reference_operand`, `is_union_operand`, `has_negated_operand`,
  `is_constant`, `chain_operands` and `cuts_written_union`.
- **Reference-context dispatch:** the entry points above, plus:
  - `is_reference` (ISREF, information/mod.rs:141-157);
  - `variable_value` / the LET binding (let.rs:77-82);
  - `call_callee`;
  - `contains_function`;
  - `compute_node_units` (units.rs:114, 136, 180-212).
- **Union and intersection matches:** model.rs:634/641, reference_operators.rs, util.rs:216-224,
  variance.rs:97, mode_functions.rs:37-50.
- **`ReferenceKind` / `RangeKind` argument tests**, which today see `(A1)` as a reference only because
  the parser drops the parentheses. Without peeling, `COUNT((A1))` or `AVERAGE((A1))` with text in
  A1 would change value. The sites:
  - count_and_average.rs:33, 38, 117, 262 and count.rs:23, 34;
  - and_or_xor_not.rs:147, concatenate.rs:345, chisq.rs:355, aggregate.rs:172;
  - product.rs:32 (single-cell fast path);
  - model.rs:271, 300.

**A `Group` ends a value-operator chain.** It is a chain operand (the first one, or a right
operand), never part of the spine. `value_operands`, `value_operands_mut`, `left_chain`,
`for_each_chain_operand_mut`, `ChainLeft::clone`'s `relink`, `grow_chain`, and the chain loops in
`stringify` (stringify.rs:642) and `to_string_moved` (move_formula.rs:415) **never peel**.
- Peeling there would print `(A1+B1)+C1` as `A1+B1+C1`, make `clone()` drop groups, and make the
  mutable walkers (row insert, rename) see a different shape from eval and print.
- Chain *evaluation* sees through a group operand through the dispatch peel above.
- Test: `clone()` and print round-trip `=((A1+B1)+C1)*D1`.

The root snap is the one place that reads the *unpeeled* node. `evaluate_formula_root` matches
`OpSumKind` on the root as written, so a `Group` root skips the snap, exactly as `GroupedRootKind`
does today. Static analysis (the three walkers in static_analysis.rs), `prefix_bound_variables`
and the rename walkers recurse into the child. These walkers are not on the evaluation path, so
one frame per group run is within the depth accounting below.

### Printing
- `Group` prints `"(".repeat(levels) + child + ")".repeat(levels)` in `stringify` and in
  `to_string_moved`.
- In the precedence tables (`operator_text`, the unary-minus table, `at_operand`,
  `callee_operand`, `tight_operand`, `intersection_operand`, `spill_operand`), a `Group` is an
  atom and is never wrapped again.
- **Precedence wrapping stays, only as a safety net.** It applies to trees the engine builds
  itself (`add_implicit_intersection` for legacy formulas, `remove_redundant_implicit_intersection`).
  The printer adds parentheses only where the bare text would re-parse to a different tree.
- For `^` this means a left operand that is itself a `^` (or a lower-precedence unary) prints
  bare. `A1^2^3` re-parses left-associatively, which fixes T-1331's I11, I26 and I46. A right `^`
  operand still needs wrapping, but a parsed tree always carries that as a `Group`.
- The rule "wrap only if the re-parse would differ" replaces the hand-tuned per-case tables over
  time. Step 1 changes only what the probes require.
- **Export.** `ImplicitIntersection` exports as `_xlfn.SINGLE(` + `stringify(child)` + `)`. When the
  operand is a `Group`, the group prints inside SINGLE's own parentheses, giving
  `_xlfn.SINGLE((Z1+(Z2)))` as Excel stores it.
- **Import.** Import parses `_xlfn.SINGLE((x))` as `@` applied to `Group{1,x}`, which prints `@(x)`.
  The round trip is the identity.

### Folding the special cases
| Today | Becomes |
|---|---|
| `GroupedRootKind` | `Group` at the root. `parse_cell_formula`'s wrapping disappears because the parser already produced the group. The defined-name body and `evaluate_formula_root` paths keep their behaviour. |
| `OpUnionKind.written` | `Group` around the union. `bare` stays: it records a union with no parentheses at all, which is a parse fact, not a group. |
| `OpIntersectKind.written` | `Group` around the cut's left union. This is the tree as written: `((A1,B1) B1)` is `Group{1, Intersect(Group{1, Union}, B1)}`. |
| `ImplicitIntersection.written` | `@` + `Group` child. |
| `ImplicitIntersection.lead` / `LeadGroup` | A `Group` on the leftmost operand or range corner, where it was written. Shapes with more than one group (E34, E35) come for free. |
| `ImplicitIntersection.grouped` | `Group` around the `@` node. |
| Parser `at_paren`, `at_group`, `leading_paren`, `grouped_root`, `root_group` | Removed. `parse_parenthesized` returns `Group { levels, child }`, merging into the child's `Group` when the inner span is exact. |

`union_text`'s stripping of a left union's inner parentheses and `at_operand`'s `written.max(1)`
become plain `Group` printing. Their special cases are deleted.

### Equality and dedupe
`Node: PartialEq` now distinguishes `A1+(B1)` from `A1+B1`. That is correct: they print
differently and Excel stores them differently. Any cache keyed by `Node` equality or printed text
(shared-formula detection, the formula pool) sees them as different formulas. This costs memory
only for formulas that differ in parentheses, which is the point.

### What does not change
- Public API: none. `Node` is crate-internal to rustsheet-engine.
- Structural edits shift references inside a `Group` like anywhere else; they never add or
  remove groups.
- The depth cap, the operator cap (`MAX_VALUE_OPERATORS`), and every parse error.

## Alternatives considered
- **A `written: u8` count on every variant.** This puts a byte on every node and touches every
  constructor and pattern (~990 occurrences). Neither design gets compiler help at the
  `matches!(…, _)` shape sites; the `Group` node gets it at the exhaustive walkers and keeps
  argument shape tests behind one peel point. Rejected.
- **Keep the original text and re-print from it.** Structural edits (row insert, sheet rename,
  cut/paste) rewrite references, so text would need token-level patching, a second source of
  truth beside the tree. Rejected.
- **A side table of group spans by node id.** Nodes have no ids. It would mean adding ids or
  pointer keys that do not survive `Clone`. Rejected.
- **Keep patching special cases** (T-1200, T-1251, T-1331 each added or would add one). Each case
  added parser state and printer branches, and the probes keep finding new shapes. Rejected; this
  spec is the replacement.

## Test plan
- **Probes** (filled by excel-graph; power-chain-text arrives with T-1331's branch):
  - probes/at-operand-nested-groups: all 36 cells pass;
  - at-group-leftmost-shapes E34, E35;
  - at-group-range-corner-whole E34;
  - power-chain-text I11, I26, I41, I46.
  Each leaves the baseline in the step that fixes it, with 0 regressions in every suite.
- **Step 0 probe** (new): the group-transparency questions listed under "Unknown", plus
  FORMULATEXT of each.
- **Unit tests:**
  - The T-1167 test (E40, E41, E43, E44) and the F50, F51 asserts flip to Excel's text.
  - test_grouped_root, test_union, test_implicit_intersection keep their print and value
    assertions. Only internal shape assertions change.
- **Metamorphic properties** (in a test module, over the probe corpus and gen-m1/gen-m2 formulas):
  - print(parse(f)) == f for every formula whose only difference from canonical form is
    parentheses;
  - eval(parse(f)) == eval(parse(strip_redundant_parens(f))).
- **Round trip:** export → import → print is the identity on the table in "Excel behavior to match".
- **T-1169 depth tests unchanged:** 125, 126 and 127 nested parentheses: parse, eval, print, row
  insert, sheet rename.
- **Performance:** the repo has no benchmark target. Step 1 reports `make eval` wall time and
  the T-1169 deep-nesting test times before and after, which must be within noise. Formulas
  without parentheses get no new allocation. A `size_of::<Node>()` const assertion.

## Rollout
Each step is one task of size S or M. Each leaves `make check` green and moves only the cells
named in it.

0. **T-1339: Probe group transparency** (research, sonnet, S). Covers the reference-context questions
   under "Unknown". Must land before step 1 merges. If a group is not transparent somewhere, amend
   this spec first.
1. **T-1340: `Group` node for value expressions** (engine, opus, M).
   - Adds the variant, `ungrouped()`, the dispatch peel (the exhaustive `Group` arm left in
     `evaluate_node_in_context` re-peels and continues, never recursing and never panicking), every shape-matcher site, the walkers,
     the printer (stringify and `to_string_moved`), and the size assertion.
   - The parser emits `Group` only where today it drops the group *and records it nowhere*. A
     run already counted by `grouped_root`, a union's or cut's `written`, or the `@` fields (`at_group`,
     `written`, `grouped`) stays with that special case until its fold step. Otherwise it would print
     twice, as `GroupedRootKind{Group{…}}` does. The five special cases are untouched.
   - The T-1110 and T-1169 stack tests (`formulas_at_the_nesting_limit_evaluate_on_a_2_mib_stack`
     and the 125/126/127 tests) stay green unchanged. They gain an alternating `SUM((SUM((…))))`
     shape. The PR reports their debug-build times before and after.
   - Fixes the plain-operator cells of at-operand-nested-groups and the power-chain cells, which
     unblocks T-1331 (it becomes that task's implementation, or T-1331 closes as done by it).
2. **T-1341: Fold `GroupedRootKind`** into a root `Group` (engine, sonnet, S). The root-snap behaviour is
   pinned by a test first.
3. **T-1342: Fold union and cut `written`** into `Group` (engine, opus, M), including `union_text`'s
   stripping rule.
4. **T-1343: Fold the `@` cases** (engine, opus, M). Covers `written`, `lead`/`LeadGroup` and `grouped`,
   plus the parser's `at_*` state.
   - Fixes the remaining at-operand-nested-groups `@` cells, E34/E35, and the T-1167/F50/F51
     assertions.
   - Exports `_xlfn.SINGLE((x))` as Excel stores it.
5. **T-1344: One printer** (engine, opus, M). `to_string_moved` reuses `stringify`'s precedence through a
   reference-rewriting hook instead of its own `moved_operator_text` table. It also deletes the remaining duplicate
   precedence tables (`is_atom` and the per-operator wrap lists in `operator_text` and the unary
   table), leaving one "wrap only if the re-parse differs" rule. It lands after T-1342, so it
   handles `Group` and no `written` fields. Today a cut/paste can
   print a different text from FORMULATEXT, for example a `^` operand. Covered by a moved-text
   round-trip test over the probe corpus.

Until step 4, the existing special cases keep handling their shapes. Steps 1–3 must not change
any `@` text.
