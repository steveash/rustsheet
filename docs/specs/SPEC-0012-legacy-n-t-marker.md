# SPEC-0012 — A legacy formula's N and T keep their legacy reading

- **Status:** accepted (on merge of T-1727)
- **Milestone / lane:** M2 / engine
- **Tasks:** T-1727 (this spec); T-1851 (the marker), T-1713 (placing it: the eval cells flip),
  T-1852 (row and column edits, moves, fill and paste keep it), T-1853 (probe: does a cell stay
  legacy after those edits in Excel). T-1768 and T-1847 widen the rule once their probes land. The
  order is under Rollout.

## Problem
A legacy formula is a plain `<f>` in xlsx, or `Input::LegacyFormula` in the API. The engine stores
it as its dynamic twin plus the `@`s the legacy rewrite adds. The rewrite is
`add_implicit_intersection` (expressions/parser/static_analysis.rs and static_analysis/legacy.rs,
last changed by T-1717, PR #1046). The xlsx import calls it in `from_a1_to_rc`. The API calls it in
`Model::legacy_formula_text`. Nothing else records that a formula is legacy. The export
re-derives the plain `<f>` vs `cm="1"` choice from the text: `legacy_reading_intersects`
(rustsheet-xlsx export/worksheets.rs) saves a formula as a one-cell dynamic array when a legacy
reading would add an `@` to it.

That works while every legacy difference shows as an `@`. One difference does not. N and T over
an array-valued IF, CHOOSE, INDEX or IFS show no `@` in Excel, yet the legacy cell is `#VALUE!`
while the dynamic cell spills or gives the first element:

| Formula (legacy cell) | Excel FORMULATEXT | Excel legacy | Excel dynamic | Ours today (legacy) |
|---|---|---|---|---|
| `=N(IF(1,{2;3}))` | `=N(IF(1,{2;3}))` | `#VALUE!` | spills 2, 3 | spills 2, 3 |
| `=T(IF(1,{"a";"b"}))` | `=T(IF(1,{"a";"b"}))` | `#VALUE!` | spills "a", "b" | spills "a", "b" |
| `=N(IF(1,{2,3}))` | no `@` | `#VALUE!` | `#SPILL!` (J is filled) | `#SPILL!` |
| `=N(CHOOSE(1,{2;3}))` | `=N(CHOOSE(1,{2;3}))` | `#VALUE!` | — | 2, shown `=N(@CHOOSE(1,{2;3}))` |
| `=N(INDEX({2;3},0,1))` | no `@` | `#VALUE!` | — | 2, shown with `@` |
| `=N(IFS(1,{2;3}))` | no `@` | `#VALUE!` | — | 2, shown with `@` |
| `=ABS(N(IF(1,{2;3})))`, `=N(IF(1,{2;3}))+0`, `=N(IF(1,{2;3}))&""` | no `@` | `#VALUE!` | — | spills 2, 3 |
| `=T(IF(1,{2;3}))`, `=N(IF(1,{"x";3}))` | no `@` | `#VALUE!` | — | spills "" / 0 |
| `=N(IF(1,{2}))` | `=N(IF(1,{2}))` | `#VALUE!` | — | 2, shown `=N(IF(1,@{2}))` |
| `=SUM(N(IF(1,{2;3})))` | no `@` | 5 | 5 | 5 |
| `=IF(1,N(IF(1,{2;3})))` | `=@IF(1,N(IF(1,{2;3})))` | 2 | — | 2 |

The legacy text is the dynamic text, so evaluation cannot tell the two cells apart. The export
has no matching hole. A dynamic `=N(IF(1,{2;3}))` has a non-scalar static result, so it is stored
as a dynamic anchor (`Cell::ArrayFormula`) and written with `cm="1"`. It never reaches
`legacy_reading_intersects`.

Snapshot (2026-10-18, main at d1efc49): 32 known failures in eval/baseline/probes.txt need this.
They are T-1713's acceptance cells: probes/legacy-n-t-reference-args A10, A20, B10, B20, E10:E11,
G10:I11, K10, K20, L10, L20, N10, N20, O10, R10:T11; probes/legacy-array-result A10:A11, G10:G11, I10;
probes/n-array-lift D4:D5.

## Goals / non-goals
Goals:
1. The engine keeps, in the stored formula, the one fact a legacy formula has and its text does
   not show. That fact is which N and T calls read an array-valued argument as `#VALUE!`. It
   survives storage, reload, sheet renames and moves, defined-name renames, undo, and xlsx export
   and import.
2. FORMULATEXT, the API's formula text and the exported `<f>` text never show the marker.
3. A dynamic formula with the same text keeps its dynamic result, and its export does not
   change. A spilling dynamic formula stays `cm="1"`. A statically scalar one (`=N(IF(A1,1,2))`)
   stays a plain `<f>` (T-1552), because it gets no marker (rule 2).
4. The marker is placed by the T-1717 rewrite, in the same walk and with the same `arrays` flag.
   No second analysis of "value position" is added.

Non-goals:
- A general "this cell is legacy" flag. No other evidenced legacy difference is invisible in the
  text. Every other one shows as an `@` (references.md "Legacy formulas put `@` on array-valued
  calls and array constants", "Legacy formulas intersect the ranges in value arguments"). If
  T-1847 or T-1768 finds another one, it gets its own marker in the same style (Design, "Extending").
- CSE array formulas. They get no legacy rewrite (`from_a1_to_rc` with `is_array_formula`). N
  inside them lifts, as Excel's array context does.
- Defined-name formulas, conditional formats and data validation. The rewrite does not run on
  them today.
- A Python `LegacyFormula` input class. The binding gets legacy cells only through xlsx open and
  save, which carry the marker in the model. No eval case or tool needs to type one from Python.

## Excel behavior to match
Evidence: probes/legacy-n-t-reference-args (all cells), probes/legacy-array-result A, G, I (rows 1,
10, 11), probes/n-array-lift A3, A4, D4, A10, A11 (all excel-graph, 2026-10-16, T-1713, T-1616).
These are recorded in docs/excel-semantics/references.md, "Legacy N and T over a
reference-returning call" and "Legacy formulas put `@` on array-valued calls and array constants",
and in information.md, "N over an array lifts element-wise".

Rules:
1. **Where:** an N or T call in a value position of a legacy cell. "Value position" is T-1717's
   `arrays` flag. These positions are probed for values:
   - Value positions (`#VALUE!`): the top of the cell (A, B, H, I, K, L, N, O, S, T), ABS's
     argument (E), and an operand of `+0` (G) or of `&""` (R).
   - Not value positions: a SUM argument (F, 5), IF's arms (P, 2), and the SUMPRODUCT array
     context (n-array-lift A10: `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` is 5).

   The rest of the flag is extrapolated from T-1717's FORMULATEXT evidence and is unprobed for
   values (T-1847). That covers other scalar functions, the arguments of IFERROR, IFNA, CHOOSE,
   IFS, SWITCH, LET and LAMBDA, and other vector arguments.
2. **Which argument:** the argument is a bare call (no `@`, no parentheses) of IF, CHOOSE, INDEX or
   IFS whose static result is not `StaticResult::Scalar`. A scalar one such as `=N(IF(A1,1,2))`
   reads the same either way, so it gets no marker and its export does not change.
   - CHOOSE, INDEX and IFS: every argument the call can return must be an array constant. That is
     every CHOOSE value argument, INDEX's `array` and every IFS value. It is the only shape
     probed (A, B, K, L). With a mixed or a reference arm, these calls keep today's `@` until
     T-1768.
   - IF has no such restriction (H, I, N, O, S, T and legacy-array-result A, G, I), because a
     reference result reads the same either way (rule 3). Not SWITCH: `=N(SWITCH(1,1,{2;3}))` spills 2, 3 in a legacy cell
   (M). Not IFERROR: it gets the `@`, `=N(@IFERROR({2;3},0))` is 2 (D, already implemented). T over
   SWITCH is unprobed (T-1847) and stays as it is.
3. **What it does:** the argument is evaluated as N and T evaluate it today, reading a reference
   as its top-left cell (`evaluate_node_top_left`). An array result is `#VALUE!`, whatever its
   size, `{2}` included (N10). A reference result is N/T of its top-left cell, as `=N(IF(1,Z1:Z2))`
   is 7 (J10). A scalar is as it was (`=N(IF(1,2))` 2, legacy-array-result J10). The error is the
   cell's value and propagates through the enclosing value positions (E, G, R).
4. **Shown:** never. FORMULATEXT of every cell above shows no `@` and plain N/T (row 20). So the
   text loses the `@`s ours prints today. That is the `@` on the CHOOSE, INDEX and IFS call (A20,
   B20, K20, L20), and the one-element array constant's `@` in IF's arm, `=N(IF(1,@{2}))` (N20;
   that `@` comes from `shows_scalar_array_at`, T-1633).
5. **Stored:** Excel keeps legacy-ness per cell, as the absence of `cm="1"` on a `<c>` with a plain
   `<f>` (xlsx-storage.md, "A one-cell dynamic formula is stored as a one-cell array with
   `cm="1"`"). So a legacy cell is written back as a plain `<f>`, and a dynamic cell whose legacy
   reading would differ is written as `cm="1"`. This is today's rule (T-1552), applied to the
   marker too.
6. **Unprobed: re-entry (T-1853 probes all of it).** We assume that a legacy cell becomes dynamic
   in Excel when its formula is typed again. That is the usual account of dynamic-array Excel, but
   it is not probed here. We also assume the cell stays legacy when rows are inserted above it, when it is moved, and when it is
   copied or filled. Excel adjusts the stored formula in those edits and does not re-enter it. No
   probe exists, and the eval harness cannot express an edit sequence. T-1853 checks whether the
   oracle can (Graph range insert/copy). Until then, T-1852 implements the assumption, and it is
   marked as one in references.md.

## Design

### The marker: two internal function kinds
`Function` gets two variants, `NLegacy` and `TLegacy`, used only as the `kind` of a
`Node::FunctionKind`. A legacy N call is `FunctionKind { kind: NLegacy, args }`. Nothing else in
the tree changes. Most code treats them as N and T. These are the sites that must know about them:

| Site | `NLegacy` / `TLegacy` |
|---|---|
| `to_localized_name`, `to_xlsx_string` (FORMULATEXT, UI text, `<f>`) | N's and T's names: the marker is never printed |
| `stringify` with no context, i.e. `to_rc_format` (internal storage) | the reserved spelling (below) |
| dispatch (functions/mod.rs) | thin wrappers `fn_n_legacy` / `fn_t_legacy` with the shared dispatch signature. They call the N/T kernel with `legacy: true`, so an `Array` result of the argument is `#VALUE!` (rule 3) |
| signature (`get_function_args_signature`) | N's and T's (one scalar argument), so the rewrite walks the argument the same way |
| static result (`static_analysis_on_function`) | `StaticResult::Scalar`, so the legacy cell is a plain `CellFormula`, never a dynamic anchor (`set_cell_with_formula`'s `is_dynamic`) |
| `reads_reference_whole`, `leaves_call_whole` (static_analysis/legacy.rs) | match N and T and their legacy kinds (one helper, `Function::is_n_or_t`), so a second pass of the rewrite over a marked tree adds nothing (idempotence) |

These are the only sites that match `Function::N` or `Function::T` today (10 lines, `rg
"Function::(N|T)\b"`). The four tables are exhaustive matches, so the compiler finds every one of
them.

The variants are not functions a user can call. They are not in `impl_function_lookup!` (no
language field), so the parser's name lookup never returns them. `Function::into_iter` (the
catalog, autocomplete and `function_iterator`) does not list them. `function_iterator` is extended,
not weakened: it reads a second, explicit list, `Function::INTERNAL`, and checks that the enum's
variants equal the lookup table plus that list.

Why a function kind rather than a node: see Alternatives B and D. In short, a new `Node` variant
would touch about 15 files of exhaustive `Node` matches. It would also risk a silent miss in every
`_ =>` tree walker, and a missed reference is a wrong recalc. A variant of `Function` touches 10
lines.

### Storage: an R1C1-only spelling
`Worksheet::shared_formulas` holds the formula in R1C1 text (`to_rc_format`). Every reload, sheet
rename and defined-name rename parses that text in `LexerMode::R1C1` (new_empty.rs,
defined_names.rs), mutates the tree, and prints it again. So the marker needs a spelling there. It
needs no field anywhere else.

- `to_rc_format` prints `NLegacy` as a reserved call name: `NLG1(` and `TLG1(` since T-1936
  (`_RS_LEGACY_N(` and `_RS_LEGACY_T(` in T-1851, still read from stored text while the workbook
  defines no name of that spelling). The spelling is an A1 cell, so no A1 text holds it as a call
  name and Excel never calls a name of it (references.md "Legacy N and T marker spelling").
- `parse_named_call` (expressions/parser/operands.rs) recognizes the spelling before the function
  lookup, next to `_xlfn.SINGLE`. In `LexerMode::R1C1` it gives the internal kind. In
  `LexerMode::A1` (typed formulas, xlsx `<f>` text) it gave plain `Function::N` / `Function::T`
  until T-1936; now it is not consulted there (the spelling is a cell). One matcher,
  `Function::from_legacy_spelling`, decides, and no A1-parsed formula can print as the reserved
  spelling. Case and `_xlfn.` prefix variants follow that matcher exactly, and a test checks that
  no other variant reparses to the internal kind. Today R1C1 mode parses only stored text (the
  three call sites above).
- **No way to type it.** Since T-1805 a formula that does not parse is stored as a marked parse
  error (expressions/parser/unparsed.rs) that the R1C1 parser hands back unparsed. So a broken
  formula holding the spelling, e.g. `=_RS_LEGACY_N(IF(1,{2;3}))+R[1]C`, comes back as the same
  parse error after a reload, never marked. T-1851's test pins this.
- **A defined name with the same spelling** (superseded by T-1936: the spelling is now a cell in
  A1 mode, a call of it `#REF!` as in Excel). In A1 mode the spelling was plain N/T, so a workbook
  LAMBDA name with that spelling is shadowed: `_RS_LEGACY_N(1)` calls N, not the name. T-1851 may
  find, with the excel-probe skill, a spelling that Excel refuses as a defined name. If it does
  not, it records the shadowing in references.md, and its test states the actual behavior. The
  entry is a known gap, not a `DEVIATION`: decision 0052 allows deviations only for directive 5
  (T-1851 recorded it as UNVERIFIED, probe pending in T-1912).
- The shared-formula dedup (`shared_formula_index`, keyed by the RC text) keeps a legacy
  `=N(IF(1,{2;3}))` and a dynamic one on the same sheet apart, as two entries, with no extra key.
- Model files saved before T-1851 have no spelling and load as today. A model file with the
  spelling, opened by an older engine, reads it as an unknown name (`#NAME?`). xlsx, the
  interchange format, never contains the spelling.

### Placement: in the T-1717 rewrite (T-1713)
This builds on PR #1046's design and keeps it. `add_legacy_intersection(node, add, arrays)`
already knows, for every node, whether it is in a value position: that is `arrays`, computed by
`legacy_array_context`. The marker is set at the end of its `Node::FunctionKind` arm, after the
arguments are rewritten:

```rust
// static_analysis.rs, FunctionKind arm, after the argument loop:
if arrays {
    mark_legacy_reading(kind, args); // N -> NLegacy, T -> TLegacy where rule 2 holds
}
```

`mark_legacy_reading` (static_analysis/legacy.rs, beside `leaves_call_whole`) is rule 2:
`kind.is_n_or_t()`, and `args[0]` is one of these bare calls, with a static result other than
`StaticResult::Scalar`:
- an IF call;
- a CHOOSE, INDEX or IFS call whose every returnable argument is an array constant.

It decides on the rewritten argument, as this snippet's place after the loop says. That matters:
`=N(IF(W1:W2,Z1,Z2))` is an array before the rewrite and a scalar after it (`IF(@W1:W2,..)`), so
deciding on the argument as written would not be idempotent. The predicate looks through the
automatic one-element `@`s that `shows_scalar_array_at` put on the returned arms (IF's arms,
CHOOSE's and IFS's values). It takes them off in place, and puts them back if the call is not marked
(item 6).

Where this spec constrains or extends #1046:

1. **Extends `leaves_call_whole`** from IF and SWITCH to the CHOOSE, INDEX and IFS shapes of rule 2.
   That drops the `@` T-1717 and T-1728 keep on them (`N(@CHOOSE(1,{2;3}))`), which they kept only
   because the marker did not exist. Their reference shapes keep their `@` until T-1768.
2. **SWITCH stays as #1046 made it:** no `@` and no marker, so it spills.
3. **IFERROR and IFNA are unchanged:** they keep the `@` on the call.
4. **Constrains `arrays`.** Until now the flag only decided where an `@` goes. From T-1713 on, it
   also decides an evaluation result. A change to `legacy_array_context` or `returns_an_argument`
   now changes values as well as text, so it needs evaluation evidence (a probe's legacy row), not
   only FORMULATEXT evidence. legacy.rs's module doc says so.
5. **No export mirror step.** `remove_redundant_legacy_intersection` (static_analysis/redundant.rs)
   needs none. The export prints N/T for the internal kinds anyway. The add pass is idempotent on a
   marked node, because the rule of `mark_legacy_reading` holds again where the marker is.
   `legacy_reading_intersects` keeps its code, and the three cases come out right:
   - A legacy marked cell is a statically scalar `CellFormula`. Remove, then add, gives the same RC
     text, so it is saved as a plain `<f>` printing `N(...)`. Excel and our import read it back as
     legacy.
   - A dynamic N/T over a non-scalar call is a dynamic anchor. It is exported with `cm="1"`, as
     today.
   - A dynamic, statically scalar formula whose legacy reading would place a marker is saved as
     `cm="1"` by the existing T-1552 rule. That is right, because its legacy reading differs.
     Rule 2's `Scalar` exclusion keeps `=N(IF(A1,1,2))` out of this case.

6. **No one-element `@` under the marker.** `shows_scalar_array_at` (T-1633) puts an `@` on a
   one-element array constant in any scalar argument (`=ABS(@{5})`). It must not do so in the
   returned arms of a call the marker covers: `=N(IF(1,{2}))` is shown with no `@` (N20), and so are
   `=T(IF(1,{"a"}))`, `=N(CHOOSE(1,{2}))`, `=T(CHOOSE(1,{"a"}))` and `=N(IFS(1,{2}))`, all `#VALUE!`
   (probes/legacy-n-t-marker-shapes A-D, T-1713). `mark_legacy_reading` drops those `@`s when it
   marks the call. The export mirror then has no such `@` to drop either.

The marker never depends on a cell position or a reference's target, only on the shape of the
tree. So structural edits that only rewrite references (insert rows, rename or move a sheet,
rename a name) cannot make a marker wrong.

### Entry points
| Path | Today | With the marker |
|---|---|---|
| xlsx import, plain `<f>` | A1 parse → rewrite → `to_rc_format` | the same; the rewrite now places the marker and RC prints it (no code change beyond the rewrite) |
| `Input::LegacyFormula` (rustsheet edit.rs) | `legacy_formula_text` prints the rewritten tree as **display** text, then `set_user_input` parses it again as a dynamic formula | the display text would drop the marker. T-1851 makes the changes listed below the table |
| eval harness (`legacy_formulas` of a case) | `Input::LegacyFormula` | unchanged, carried by the row above |
| reload, sheet rename/move/delete, defined-name rename (`reset_parsed_structures`, new_empty.rs, defined_names.rs) | parse RC → mutate → `to_rc_format` | carried by the RC spelling, no change |
| facade undo (`Undo::Cells`) | restores the `Cell`; its `f` index into `shared_formulas` stays valid, because the renames rewrite entries in place | carried, no change |
| `Input::Formula` / typed text / `set_user_input` | dynamic | dynamic, as in Excel when a formula is typed again; the A1 spelling is plain N |
| row/column insert and delete (`displace_cells`, actions.rs), `move_cell`, `move_cell_value_to_area`, `extend_to` / `extend_copied_value` (fill), UserModel clipboard and autofill | print **display** text, then re-enter it through `update_cell_with_formula` / `set_user_input` | would drop the marker. T-1852: when the source node holds a marker (`Node::has_legacy_marker`), the text goes through the legacy entry, not the dynamic one. Cost: `has_legacy_marker` is one O(nodes) walk per re-entered cell, the same order as the text print these paths already do, and T-1852 may cache it per shared-formula index. None of these is reachable from the `rustsheet` facade today; they are engine `UserModel` paths |
| Python binding | facade and raw model; legacy cells only via xlsx open | no change: the model carries the marker; `cell().formula` shows N |

T-1851's changes for `Input::LegacyFormula`:
- It splits `set_cell_with_formula` into `parse_cell_formula` plus one shared
  `set_cell_with_formula_node(sheet, row, column, node, style) -> Result<i32, String>`. There is no
  copy (directive 3).
- It adds `pub fn set_cell_with_legacy_formula(&mut self, sheet: u32, row: i32, column: i32,
  formula: &str) -> Result<(), String>`. It parses, rewrites, then calls the node setter. Its error
  convention is `set_user_input`'s, which `apply` maps through `EditError::from_engine_message`.
- `apply` calls it after `prepare_cell_for_user_input`.
- The pub `legacy_formula_text` stays. It gives the display text and three tests call it, so
  nothing breaks.

T-1852's re-entry relies on the rewrite being idempotent over its own display text:
`rewrite(parse_a1(display(rewrite(x))))` equals `rewrite(x)` in RC. The export already relies on
the same property (a remove-then-add of an imported formula is the identity), and #1046's unit
tests cover it for the `@`s. T-1713 adds the property test for the marker (Test plan).

### Evaluation
The kernels of `fn_n` (functions/information/mod.rs) and `fn_t` (functions/text/t_function.rs)
take `legacy: bool`. `fn_n` and `fn_t` call them with `false`. The dispatch entries of the
internal kinds, `fn_n_legacy` and `fn_t_legacy`, call them with `true`. The dispatch signature
is unchanged. The `CalcResult::Array` arm, which maps the
elements today (T-1616, T-1729), returns `CalcResult::Error { error: Error::VALUE, origin: cell,
.. }` when `legacy` is set. The other arms are shared. No other evaluator code changes. The rule
lives in the N/T kernels, where the argument's result type is already known, and nowhere else.

### Extending
If a later probe finds another legacy difference that the text does not show, it follows the same
pattern. It gets an internal kind (or a `legacy: true` dispatch flag) for the one function
involved, and a rule in or beside `mark_legacy_reading`. T-1847's
`T(SWITCH(..))` and T-1768's `N(CHOOSE(1,Z1:Z2))` would only widen the predicate. If more than two
or three functions ever need markers, the internal kinds should become a `legacy` bit on
`FunctionKind`. That needs a spec amendment, and is not done ahead of need.

### Cost
- **Memory per cell: zero.** `Cell` is unchanged. Only the formulas that contain a marked N/T
  call change, and they grow by the spelling's extra length (about 12 bytes per marked call) in
  their one shared RC string per distinct formula. `Node` is unchanged: `Function` is a
  fieldless enum and two more variants keep its size; `NODE_SIZE`'s const assert still pins 80.
- **Import:** the rewrite already visits every node once. Per `FunctionKind` in a value position,
  the marker check costs one match on N/T. Per N/T call over IF, CHOOSE, INDEX or IFS, it adds one
  scan of that call's arguments and one static analysis of it: O(args) beyond the analysis, no
  recursion of its own, no clone. The kind and the one-element `@`s are changed in place. The only
  allocation is a small index list, and only when there is a one-element `@` to take off.
- **Export:** `legacy_reading_intersects` already clones each shared formula and runs remove,
  then add. The marker adds O(1) per N/T call to the add pass.
- **Recalc:** the internal kinds dispatch to their own thin wrappers. The kernel reads its
  `bool` only in the array arm. No other evaluation path
  changes.
- **Load:** R1C1 parsing tries the reserved spelling in `parse_named_call`, one string comparison
  per call (beside `_xlfn.SINGLE`'s).

No benchmark gate is needed: no hot path changes. T-1851 and T-1713 run `make bench` if the import
benchmark covers legacy workbooks, and report it in the PR.

## Alternatives considered

**A. A per-cell legacy flag, as Excel stores it (`cm` on the `<c>`).** A `legacy: bool` on
`Cell::CellFormula`, set by the import and by `Input::LegacyFormula`. A replica of the `Cell`
layout measures 80 bytes with and without the field (`ArrayFormula` is the largest variant, and the
field fits in `CellFormula`'s spare space). The bincode `.ic` encoding would add a byte per formula
cell. It was rejected for three reasons:
1. *Evaluation cannot use it alone.* N's legacy reading depends on whether that N call is in a
   value position (rule 1). The flag would make the whole cell legacy, and the evaluator would have
   to recompute T-1717's `arrays` flag at every N/T call. That is a second, runtime copy of the
   rewrite's position analysis, against directive 3. Caching it means a marker per formula anyway.
2. *Shared formulas.* A legacy and a dynamic cell with the same RC text share one entry and one
   parsed tree, so the tree cannot differ between them. The dedup key would have to become
   (text, flag) in `shared_formula_index`, the import's `find_or_add_formula` and their lookups.
3. *More paths to carry it.* Every cell move, copy and structural path builds `Cell` values
   (`set_cell_with_formula`, the import's sheet_data.rs, the clipboard). Each would need the flag.
   The marker rides on the RC text, which these paths already preserve, or re-enter (T-1852).

What it does well is match Excel's model for the export. The marker gets the same result from the
existing T-1552 comparison (Placement, item 5).

**B. A hidden `Node` wrapper on N's argument** (`Node::LegacyRead { child }`), meaning "a
reference passes, an array is `#VALUE!`". It was rejected. A `Node` variant touches the
exhaustive matches in about 15 files (`rg -c EmptyArgKind`), and every `_ =>` walker would skip
it silently. The wrapper would also have to return references through
`evaluate_node_with_reference`. And its semantics are not general: `=ABS(IF(1,{-2;3}))` takes an
`@` and gives 2 (legacy-array-result B), so "array is `#VALUE!`" belongs to N/T, not to a position.

**C. A per-formula flag** beside `Worksheet::shared_formulas` (a parallel `Vec<bool>`, or a
sentinel prefix on the string), with the marker re-derived by running the rewrite when the formula
is parsed. It was rejected. Every reader and writer of `shared_formulas` would have to keep the
flag in step: load, the two sheet-rename paths, defined-name rename, the import and
`shared_formula_index`. The RC spelling is carried by the parse → mutate → print round trip all of
them already do.

**D. A third mode of the `@` node** (`automatic` becomes typed, automatic, or legacy-array). It
was rejected. It is not an intersection: a reference must pass whole, and an array must be an
error. Static analysis types every `ImplicitIntersection` as `Scalar`, `stringify` prints `@`, and
the mirror reasons about `@` redundancy. Each of these would need an exception.

**E. Do nothing until a general "legacy cell" model is needed.** It was rejected, because 32
failing probe cells already need it.

## Test plan
- **T-1851 (unit, engine):**
  - The RC spelling parses to the internal kind in R1C1 mode and to plain N/T in A1 mode.
  - `to_rc_format` prints it. `to_localized_string`, `to_english_string`, `to_excel_string`,
    `to_excel_array_formula_string` and FORMULATEXT print `N(` / `T(`.
  - The edge behavior is pinned:
    - An A1 call of the spelling is plain N/T, also when a LAMBDA name with that spelling exists.
      This is the known gap above, unless a spelling Excel refuses as a name is found.
    - A parse-error formula holding the spelling comes back as the same parse error after a
      reload (T-1805), never marked.
    - Case and `_xlfn.` variants that `is_legacy_marker_spelling` does not accept never reparse
      to the internal kind.
  - `function_iterator` with `Function::INTERNAL`.
  - Evaluation of the marked kinds from a stored RC formula: an array argument (2x1, 1x2, 1x1,
    mixed types) is `#VALUE!`; a range is its top-left; a scalar is unchanged.
  - The static result is `Scalar`, so a marked formula's cell is a `CellFormula`.
  - `Input::LegacyFormula` stores the rewritten node, not its display text: a test with a marked
    RC string written by hand, until T-1713 places markers.
- **T-1713 (eval and unit):**
  - Its acceptance cells (Problem) leave eval/baseline/probes.txt, with 0 regressions in probes,
    ironcalc-calc, poi and fixtures.
  - Placement tests, one per row of the Problem table, including the negative ones: SUM, `@IF`,
    SWITCH, IFERROR, N over an array constant (`=N(@{2;3})`, legacy-array-result E10), and a
    statically scalar IF: legacy and dynamic `=N(IF(A1,1,2))` get no marker, and the dynamic one
    still exports as a plain `<f>`.
  - Idempotence: `rewrite(rewrite(x)) == rewrite(x)`, and the display round trip above, as RC
    text, over every `legacy_formulas` cell of every probes case.
  - Export round trip, as a regression guard: a legacy `=N(IF(1,{2;3}))` gives a plain `<f>`, and reimport gives the
    marker and `#VALUE!`. A dynamic one gives `cm="1"`, and reimport gives no marker and a spill.
    The same tests for T, CHOOSE, INDEX and IFS. A legacy and a dynamic copy on one sheet get two
    shared-formula entries.
- **T-1852 (unit):** after a row insert above it, a column delete beside it, a move, a fill and a
  paste, a marked cell (`=N(IF(1,{2;3}))+A5`, whose text the edit rewrites) still has its marker
  and is still `#VALUE!`. An unmarked dynamic twin stays dynamic.
- **Fuzzing:** fuzz/fuzz_targets/formula_parse.rs parses A1 text only. T-1851 extends it in two
  ways. It feeds the raw fuzz input to the parser in `LexerMode::R1C1` as well, because a
  hand-made model file can hold any stored text. It also re-parses its `to_rc_format` output in
  R1C1 mode. No new target is needed.

## Rollout
1. **T-1851** (engine, M): the internal kinds, the R1C1 spelling, names, signature, static
   result, `fn_n`/`fn_t` with `legacy`, `Function::INTERNAL`, and
   `Model::set_cell_with_legacy_formula` with `Input::LegacyFormula` switched to it. No marker is
   placed yet, so no eval result changes. That is checked: probes known failures unchanged.
2. **T-1713** (functions-logic → engine lane, since the change is in static_analysis/; M):
   `mark_legacy_reading`, the `leaves_call_whole` extension, the one-element `@` rule, the
   idempotence and export round-trip tests, the eval flip, and the references.md and
   information.md updates ("Not implemented" removed). Depends on T-1851.
3. **T-1852** (engine, S): the UserModel re-entry paths. Depends on T-1851; independent of T-1713,
   but only observable after it.
4. **T-1853** (research): probe rule 6 if the oracle can express the edit; it updates
   references.md either way.
5. T-1768 and T-1847 (existing): when their probes land, they widen `mark_legacy_reading`
   (CHOOSE/INDEX over references, T over SWITCH), with eval evidence, as constraint 4 requires.
