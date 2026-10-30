# RustSheet Synthetic Spreadsheet Generator — Research & Design

> Research snapshot produced at project inception (2026-10-01) by research agents. Treat as background, not as normative spec; normative docs live in `docs/architecture/` and `docs/process/`. Update via a PR when findings change.

Status: design proposal (v0.1, 2026-10-01)
Scope: a stochastic generator that produces `.xlsx` workbooks that exercise the Excel
function landscape in a targeted way. Real Microsoft 365 Excel computes them (the
oracle), and RustSheet's results are diffed against Excel's. When a diff turns up, it is
shrunk to a minimal repro and filed as a task. A second loop needs no oracle: it checks
metamorphic relations and runs continuously against RustSheet alone.

---

## 0. TL;DR

1. **Catalog first.** A machine-readable function signature catalog (YAML, one file per
   category) is the single source of truth. Every generator decision is driven by it:
   argument types, arity, domains, edge pools, version, volatility, milestone.
2. **Type-directed and grammar-based generation** of formulas (Csmith/YARPGen style),
   with depth and size budgets. Weighting comes from three sources: corpus frequency
   priors (Enron, EUSES, FUSE), adaptive gap-seeking (Thompson-sampling bandit over the
   coverage matrix), and a milestone mask. Swarm testing randomly disables feature
   subsets for each workbook.
3. **Values come from typed edge pools**: boundary numbers, FP traps, date serials around
   the 1900 bug, numeric-looking text, unicode, blanks, errors. Formulas are generated
   valid-by-construction by default and invalid on purpose at a controlled rate.
4. **Workbooks are generated in an IR** (canonical JSON), not directly as xlsx. Writers
   emit xlsx from it (XlsxWriter / rust_xlsxwriter, because both handle `_xlfn.` and
   dynamic-array metadata correctly). RustSheet loads the IR directly for the fast
   no-oracle loop.
5. **Oracle economy.** Pack about 1–5k independent probes per workbook, add canary cells,
   and pin and record the Excel build. Subexpression probing records Excel's value for
   every AST node, so most minimization needs no new oracle calls.
6. **Metamorphic relations (MRs)** run without Excel: range additivity, TLP-style
   partitioning, NoREC-style "optimized vs naive" function pairs, insert-row invariance,
   recalc-order, incremental-vs-full, save/load round-trip. Values are generated as
   exactly representable dyadic numbers wherever an MR needs exact arithmetic.
7. **Minimization**: hierarchical (workbook → sheet → cell cone → AST via HDD/Perses →
   literal shrinking), plus Hypothesis-style choice-tape shrinking. Every repro is
   bucketed by a signature so each task is filed once.
8. **Determinism**: a seed tree plus a choice tape. Artifacts are named by content hash
   (sha256 of the canonical IR). The generator and catalog versions are stamped into
   every manifest.
9. **Language**: generator in **Python** (fast iteration, Hypothesis-like strategies),
   xlsx emitted with **XlsxWriter** rather than openpyxl. The MR runner and the
   incremental-recalc checks are in **Rust** inside the RustSheet workspace and consume
   the JSON IR.

---

## 1. Prior art and what we take from each

| Area | Work | Lesson applied |
|---|---|---|
| Random program generation | Csmith (Yang et al., PLDI 2011); YARPGen (Livinskii, Babokin, Regehr, OOPSLA 2020) [1][2] | Generate programs that are **valid by construction** (no UB, which for us means no unintended `#VALUE!` swamps). YARPGen's *generation policies* systematically change the character of the output to **delay saturation**. Our equivalent is "policies": numeric-heavy, text-heavy, lookup-heavy, error-propagation-heavy, array-heavy. |
| Swarm testing | Groce, Zhang, Eide, Chen, Regehr, ISSTA 2012 [3] | Each test uses a random **feature subset**. Some features suppress others (an `IFERROR` wrapper hides errors), and features compete for space. Found 42% more distinct compiler crashes than a tuned default. Implemented as a per-workbook *swarm mask* over functions, operators and value classes. |
| DBMS logic bugs | SQLancer: PQS (OSDI 2020), NoREC, TLP (OOPSLA 2020) [4][5] | **Metamorphic oracles** that need no reference implementation. TLP partitions a predicate into TRUE/FALSE/NULL parts whose union must equal the whole; our analogue partitions ranges and criteria. NoREC compares an "optimized" query with an unoptimizable rewrite; our analogue compares `SUMIFS` with `SUMPRODUCT`, and `XLOOKUP` with `INDEX/MATCH`. |
| Equivalence modulo inputs | EMI (Le, Afshari, Su, PLDI 2014) | Mutating dead code must not change the output. Our analogue mutates the untaken branch of `IF`/`CHOOSE`/`IFERROR` and must see the same result (with care about errors in the untaken branch: Excel short-circuits `IF`). |
| Coverage-guided generator fuzzing | Zest/JQF (Padhye, Lemieux, Sen, ISSTA 2019) [6] | Make the generator a **deterministic function of a parameter byte stream (choice tape)**. Mutating the tape gives structural mutations, and engine code coverage can be used as feedback. |
| Bandit scheduling | EcoFuzz (USENIX Sec 2020); Thompson-sampling greybox fuzzing (Karamcheti et al. 2018; T-Scheduler) [7][8] | Model "which (function, arg-class, edge-class) arm to pull" as a **Beta–Bernoulli bandit** solved with Thompson sampling. Each arm starts at Beta(1,1). |
| Test-case reduction | Delta debugging (Zeller & Hildebrandt, TSE 2002); HDD (Misherghi & Su, ICSE 2006); C-Reduce (Regehr et al., PLDI 2012); Perses (Sun et al., ICSE 2018) [9] | Reduce **along the grammar** (keep candidates syntactically valid). Use hoisting (replace a node with a same-typed descendant) and literal simplification. |
| Internal reduction | Hypothesis (MacIver & Donaldson, ECOOP 2020) [10] | Shrink the **choice sequence** instead of the output, so every shrunk candidate is still a valid generator output that satisfies all constraints. Free with the choice-tape design. |
| Oracle-less reduction | DDMT (Jiang, Tian, Chen, 2026) [11] | Delta debugging can use an **MR violation** as its interestingness test. We do this for MR-found bugs, so minimization costs zero Excel calls. |
| Spreadsheet structural semantics | Kale (2026) [12] | Reference adjustment on row/column insert and delete often differs from user intent. The *engine* must match Excel exactly, so insert/delete transforms are a high-value MR and oracle target, especially at range boundaries. |
| Cross-engine compatibility data | spreadsheet-compat / canispreadsheet.com [13] | Hand-authored JSON cases run on LibreOffice, Google Sheets and Excel-for-web. Two good ideas to copy: (a) **canary cells** (`=1111+2222` must equal 3333; `=NOW()` must differ across runs) that *prove* recalculation happened; (b) a `trusted` flag on each result file. Reported LibreOffice divergences (`ISNUMBER(TRUE)`, `POWER(-8,1/3)`, `MROUND(5,-2)`, the pre-March-1900 epoch) are exactly the edge classes our pools target. |
| Existing Rust engine | IronCalc (`xlsx/tests/calc_tests/**.xlsx`) | Hand-built Excel-saved workbooks per category: the engine recomputes and compares with Excel's cached `<v>` values. This is our oracle format in miniature. Our generator automates authoring these workbooks. |

### 1.1 Corpus statistics used as frequency priors

- **Enron** (Hermans & Murphy-Hill, MSR/ICSE-SEIP 2015) [14]: by share of spreadsheets that
  use each function: SUM 72.0%, `+` 61.8%, `-` 54.0%, `*` 39.1%, `/` 34.5%, IF 20.3%,
  NOW 16.7%, AVERAGE 9.8%, VLOOKUP 8.5%. 76% of spreadsheets use only these nine.
- **Enron vs EUSES** (Jansen, SEMS 2015) [15]. Top 15 functions:
  - Enron: SUM, IF, AVERAGE, VLOOKUP, ROUND, SUBTOTAL, OFFSET, CONCATENATE, NOW,
    DAVERAGE, SUMIF, INDEX, MATCH, LOOKUP, MONTH.
  - EUSES: SUM, IF, ROUND, HYPERLINK, CONCATENATE, AND, COUNTIF, AVERAGE, OR, INDIRECT,
    MIN, ISNUMBER, MAX, VLOOKUP, ISBLANK.
  - Category shares of function calls (Enron / EUSES): operators 71.4/58.5%,
    math & trig 16.4/19.2%, logical 3.6/9.9%, lookup & reference 2.8/3.6%,
    statistical 2.5/4.0%, date & time 1.6/0.5%.
  - **Formula shape** (medians): parse-tree depth 2, preceding cells 2, unique functions
    per formula 1, formula length 18–20 characters, unique formulas per sheet 10–14,
    formulas per sheet 73–128.
- **FUSE** (Barik et al., MSR 2015, 249,376 spreadsheets) [16]: IF is the most frequent
  function, in **17.8% of formula cells**. The most common formula is
  `=SUM(R[-3]C:R[-1]C)`, which supports fill-down and sum-above patterns as a first-class
  structure. Only 7% of web spreadsheets contain any formula.
- **Array formulas, pivots, charts** (Jansen & Hermans 2018) [17]: about 1.2% of sheets
  contain array formulas. 60.5% of those do multi-criteria aggregation (today's
  SUMIFS/FILTER territory), 17.4% call array functions
  (TRANSPOSE/MMULT/LINEST/FREQUENCY/MINVERSE), and 11.5% use SUMPRODUCT.
- **Microsoft's "top 10"** [18]: SUM, IF, LOOKUP, VLOOKUP, MATCH, CHOOSE, DATE, DAYS,
  FIND, INDEX.
- These corpora predate Excel 2010–2024. For modern functions (XLOOKUP, FILTER,
  LET, LAMBDA, TEXTSPLIT…) we have no published frequency data, so we use a **hand-set
  prior** of about 0.3× the matching legacy family. Example: XLOOKUP ≈ 0.3 × VLOOKUP.

**Takeaway.** Real formulas are *shallow* (median depth 2) and dominated by operators,
SUM, IF and lookups. A realism-weighted generator would spend most of its budget there.
That is correct for "does the common stuff work", but bugs live in the long tail. So
priors are **tempered** (`p ∝ freq^α` with α≈0.5) and mixed with gap-seeking weights
(§5.3).

---

## 2. Architecture

```
               +-------------------+        +------------------------+
 catalog/*.yaml|  Function catalog | ----> | Generator (Python)     |<---- coverage.parquet
               +-------------------+        |  seed -> choice tape   |<---- failures.parquet
 scenarios/*.py| Scenario templates| ----> |  -> Workbook IR (JSON) |      (bandit state)
               +-------------------+        +-----------+------------+
                                                        | IR (canonical JSON, content-hashed)
                    +-----------------------------------+-------------------------+
                    v                                   v                         v
          +------------------+              +--------------------+     +----------------------+
          | xlsx writer      |              | RustSheet MR runner|     | RustSheet evaluator  |
          | (XlsxWriter)     |              | (Rust, no oracle)  |     | (loads xlsx or IR)   |
          +--------+---------+              +---------+----------+     +----------+-----------+
                   | .xlsx                            | MR violations             | values
                   v                                  v                           v
          +------------------+   oracle.json   +---------------------------------------+
          | Excel oracle farm|---------------->| Differ / classifier -> Reducer -> Dedup|
          | (pinned build)   |                 |        -> task filer (repro .xlsx+IR) |
          +------------------+                 +---------------------------------------+
```

There are two loops:
- **Oracle loop** (slow, batched): IR → xlsx → Excel → oracle.json → diff.
- **MR loop** (fast, continuous): IR → RustSheet → check MRs → reduce → file. It needs no
  Excel and runs in CI and on idle cores.

---

## 3. Function signature catalog

### 3.1 Type lattice

Argument and return types (used by the generator and the coverage matrix):

```
any
 ├─ scalar
 │   ├─ number      (IEEE double, Excel-normalized: no -0, no NaN/Inf, no denormals)
 │   ├─ text
 │   ├─ logical
 │   ├─ error       (#NULL! #DIV/0! #VALUE! #REF! #NAME? #NUM! #N/A #GETTING_DATA #SPILL! #CALC! #FIELD! #BLOCKED! #CONNECT! #BUSY! #UNKNOWN!)
 │   └─ blank       (only as a cell state / omitted arg, never a literal)
 ├─ reference      (single cell | area | multi-area union | 3D | spill ref A1#)
 │   └─ range       (area reference, rectangular)
 └─ array          (literal {..} or computed; rows x cols)
 lambda            (LAMBDA value; Excel 2024/365)
```

Each parameter also has a **passing mode**, which is what determines Excel's coercion
rules. This is the most important subtlety to encode:

| mode | meaning | Excel behavior example |
|---|---|---|
| `value` | scalar wanted; a range triggers implicit intersection (legacy) or array lifting (DA) | `ABS(A1:A3)` spills in DA Excel |
| `values_agg` | aggregate over values; **direct** scalars are coerced, **in-range** text and bools are skipped | `SUM("1",TRUE)=2` but `SUM(A1:A2)` with `"1"`,`TRUE` is 0 |
| `values_agg_a` | the "A" variants count text as 0 and TRUE as 1 | `AVERAGEA`, `COUNTA` |
| `ref_only` | must be a reference; an array or scalar gives an error or is disallowed at parse | `SUMIF(range,…)`, `OFFSET(ref,…)`, `ROW(ref)` |
| `array_ok` | takes ranges or arrays without lifting | `SUMPRODUCT`, `INDEX` array form |
| `criteria` | criteria-string mini-language (`">=5"`, `"a*"`, `"<>"`, `"=" `) | `COUNTIF`, `SUMIFS`, `D*` functions |
| `lambda` | expects a LAMBDA | `MAP`, `REDUCE`, `BYROW` |

The v0 catalog (eval/catalog/functions.yaml) spells these `aggregate`, `aggregate_a`, `range` and
`array` (for `values_agg`, `values_agg_a`, `ref_only` and `array_ok`); its header is the reference.

### 3.2 Catalog schema (YAML)

> Current state (T-0040): one file, `eval/catalog/functions.yaml`, lists every Excel function;
> its header is the authoritative field list (`category`, `milestone`, `scope`/`reason`, `oracle`,
> and `params` only for functions the generator calls). The per-category files and
> `catalog/schema.json` below are the original design, not yet built.

One file per category: `catalog/math.yaml`, `catalog/lookup.yaml`, and so on. Validated by
a JSON Schema (`catalog/schema.json`). Fields:

```yaml
# catalog/math.yaml
- name: SUM
  category: math_trig
  introduced: "1985"            # Excel version string; scraped from support.microsoft.com
                                # alphabetical list "(2010)/(2013)/(2019)/(2021)/(2024)/(Microsoft 365)"
  xlfn_prefix: null             # null | "_xlfn." | "_xlfn._xlws." (FILTER, SORT) - drives the writer
  volatile: false
  returns: { type: number, shape: scalar }
  dynamic_array: false          # can return a spill
  args:
    - name: number
      type: [number, reference, array]
      mode: values_agg
      repeat: { min: 1, max: 255 }  # repeating group; Excel max 255 args
  edge_classes:                 # hand-curated, merged with the global pools for the arg type
    - { id: sum_cancel, gen: "pair(x, -x) with x in FP.cancellation" }
    - { id: sum_text_direct, gen: "literal text '1' as direct arg" }
    - { id: sum_text_in_range, gen: "cell containing text '1' inside range" }
    - { id: sum_bool_direct }
    - { id: sum_error_in_range }
    - { id: sum_overflow, gen: "1e308 + 1e308 -> #NUM!" }
  freq_prior: 0.72              # share of spreadsheets that use it (Enron); see §1.1
  milestone: M1
  aliases: []
  notes: "Close-to-zero snapping differs from the binary + operator (to verify)."

- name: ROUND
  category: math_trig
  introduced: "1985"
  volatile: false
  returns: { type: number, shape: scalar }
  args:
    - { name: number,     type: number, mode: value }
    - { name: num_digits, type: number, mode: value, domain: { int_trunc: true, range: [-308, 308] } }
  edge_classes:
    - { id: half_even_vs_away, values: [0.5, 1.5, 2.5, -2.5, 2.675, 1.005, 0.285] }
    - { id: neg_digits, values: [-1, -2, -15, -16, -308] }
    - { id: big_digits, values: [15, 16, 17, 30, 308, 309] }
    - { id: frac_digits, values: [1.9, -1.9] }   # truncation of non-integer digits
  freq_prior: 0.06
  milestone: M1

- name: VLOOKUP
  category: lookup_ref
  introduced: "1985"
  volatile: false
  returns: { type: any, shape: scalar }
  args:
    - { name: lookup_value, type: [number, text, logical], mode: value }
    - { name: table_array,  type: [range, array], mode: array_ok,
        constraints: { min_cols_from_arg: col_index_num } }
    - { name: col_index_num, type: number, mode: value, domain: { int_trunc: true, range: [1, 16384] } }
    - { name: range_lookup, type: logical, mode: value, optional: true, default: true }
  edge_classes:
    - { id: approx_unsorted }             # binary search on unsorted data: exact Excel behaviour matters
    - { id: wildcard_text, values: ["a*", "?b", "~*", "~~"] }
    - { id: col_out_of_range }            # -> #REF!
    - { id: col_zero }                    # -> #VALUE!
    - { id: mixed_type_column }           # numbers stored as text vs numbers
    - { id: duplicate_keys }              # first vs last match semantics
  generation_hints:
    table_shape: { rows: [1, 200], cols: [1, 8] }
    key_from_table_prob: 0.7              # most lookups should hit
  freq_prior: 0.085
  milestone: M2

- name: XLOOKUP
  category: lookup_ref
  introduced: "2021"
  xlfn_prefix: "_xlfn."
  dynamic_array: true
  returns: { type: any, shape: scalar_or_array }
  args:
    - { name: lookup_value, type: any, mode: value }
    - { name: lookup_array, type: [range, array], mode: array_ok, constraints: { vector: true } }
    - { name: return_array, type: [range, array], mode: array_ok, constraints: { same_len_as: lookup_array } }
    - { name: if_not_found, type: any, optional: true }
    - { name: match_mode,  type: number, optional: true, domain: { enum: [0, -1, 1, 2] } }
    - { name: search_mode, type: number, optional: true, domain: { enum: [1, -1, 2, -2] } }
  freq_prior: 0.025                       # hand-set: 0.3 x VLOOKUP
  milestone: M4

- name: NOW
  category: date_time
  introduced: "1985"
  volatile: true
  returns: { type: number, shape: scalar }
  args: []
  oracle_policy: exclude_value          # compare only type / ISNUMBER; see §9.1
  milestone: M1

- name: TEXT
  category: text
  introduced: "1985"
  returns: { type: text, shape: scalar }
  args:
    - { name: value,       type: [number, text, logical], mode: value }
    - { name: format_text, type: text, mode: value, pool: number_formats }
  locale_sensitive: true                # pin oracle to en-US
  milestone: M3
```

Key schema elements:
- `args[].type`: a type or a union from §3.1. `mode` comes from §3.1.
- `optional`, `default`, `repeat {min,max,group}`. Repeat groups cover pair-repeats such as
  `SUMIFS(sum_range, crit_range1, crit1, crit_range2, crit2, …)` with
  `repeat: {group: [crit_range, criteria], min: 1, max: 127}`.
- `domain`: `range`, `enum`, `int_trunc`, `positive`, `nonzero`, `prob` (0..1),
  `date_serial` (0..2958465), `same_shape_as`, `vector`, and cross-argument constraints.
  The generator treats domains as **soft**: in-domain with probability `1 − p_violate`
  (default 0.15), otherwise a deliberate violation drawn from the edge pool. The resulting
  errors are themselves test targets.
  A domain marked `hard: true` is never violated: outside it Excel refuses the whole workbook
  (MUNIT's dimension, T-1087), so a violation would cost every other formula of the case.
  `enum` lists the usual text values of a parameter (CELL's info_type; T-1088), soft like `int`.
- `cell: true` (a `range` parameter): the generator passes one cell. ISFORMULA lifts over a range, which would spill into
  the next formula, and Excel refuses a literal (T-1088).
- `vector: true` (a `range` or `array` parameter): the generator passes a one-column range (unless `wide_rate` or `row_rate` below say otherwise). LOOKUP and the XLOOKUP/XMATCH lookup
  vectors give only `#N/A` or `#VALUE!` for a 2-D range, which would hide the search itself (T-1238); WRAPROWS/WRAPCOLS
  wrap one column (T-1083).
  `wide_rate: <share>` makes that share of the draws 2-D and `row_rate: <share>` makes it a one-row range, so MATCH's
  lookup_array is mostly one-dimensional with a few 2-D `#N/A` cases (T-2719).
- `same_shape: true` (`range` parameters of one call): the ranges share one rows x columns shape, except in about 1 call
  in 10, so the *IFS functions compute a real match and the shape-mismatch `#VALUE!` stays covered (T-2719).
- `probe: <param>` (a `value` parameter): drawn from the cells of that `range` parameter 95% of the time, so MATCH's
  lookup_value usually exists in the array (T-2719).
- `full: true` (an `array` parameter): every range spans all data rows, so GROUPBY's, SORTBY's and FILTER's arrays are the
  same height; `mask: true` makes FILTER's include a comparison of one full column (T-1083).
- `mode: lambda` (with `arity`, or `arity: repeat` for MAP's one parameter per array) is a LAMBDA whose body may use its
  parameters, or 30% of the time an eta-reduced name written `_xleta.SUM`; `mode: name` declares a LET variable or LAMBDA
  parameter, visible after its value (LET) or in the `body: true` calculation; `invoke: true` calls the LAMBDA at once.
  `spelled` is the formula spelling where the case builder adds no prefix (`_xlfn.GROUPBY`; T-1083).
- `always: true` (an optional parameter): the generator never omits it (MATCH's match_type, whose default 1 on unsorted
  data is mostly `#N/A`, T-2719). CELL without a reference reads the cell last
  changed, which no oracle value can match.
- `edge_classes`: named, function-specific value or structure generators. They join the
  global type pools (§6) and become coverage-matrix columns (§13).
- `introduced`, `xlfn_prefix`, `volatile`, `dynamic_array`, `locale_sensitive`.
- `freq_prior`: a corpus number when one exists, otherwise hand-set.
- `milestone`: `M1..Mn`, matching the RustSheet roadmap. The generator only samples
  functions whose milestone ≤ the current milestone (plus "canary" lookahead, §5.4).
- `oracle_policy`: `compare` (default), `exclude_value`, `compare_type_only`,
  `tolerance: {ulps: N}`.

**Bootstrap.** Scrape names, categories and versions from Microsoft's
"Excel functions (alphabetical)" page [19], which tags versions as "(2010)", "(2013)",
"(2019)", "(2021)", "(2024)", "(Microsoft 365)". Cross-check names against IronCalc's
`Function` enum. Arg signatures for the top ~150 functions are hand-curated. The rest are
semi-automated from the docs and marked `reviewed: false`. A CI lint checks that every
catalog function parses in RustSheet and appears in the xlfn prefix table of the writer.

---

## 4. Workbook IR

The generator never writes xlsx directly. It emits a canonical JSON IR:

```jsonc
{
  "ir_version": 1,
  "generator": {"version": "0.3.1", "catalog_hash": "sha256:…", "policy": "lookup_heavy",
                "seed": "0x9f3c…", "swarm_mask": ["-INDIRECT", "-text_unicode", …]},
  "settings": {"date_system": 1900, "calc_mode": "auto", "iterative": false, "locale": "en-US"},
  "sheets": [
    {"name": "Data", "cells": {
        "A1": {"v": "id"}, "A2": {"v": 1}, "B2": {"v": 45292, "fmt": "yyyy-mm-dd"},
        "C2": {"v": "  42 "}, "D2": {"v": true}, "E2": {"err": "#N/A"}
     }},
    {"name": "Calc", "cells": {
        "B5": {"f": "SUM(Data!$C$2:$C$101)*1.1", "kind": "probe", "probe_id": "p00017",
               "ast_ref": "ast/p00017.json"},
        "D2": {"f": "SEQUENCE(3,2)", "dyn": true}
     }}
  ],
  "names": [{"name": "Rate", "ref": "Calc!$B$1", "scope": null}],
  "probes": [{"id": "p00017", "cell": "Calc!B5", "functions": ["SUM"],
              "edge_classes": ["sum_text_in_range"], "depth": 2,
              "subexpr_cells": ["Probe!A1", "Probe!A2"]}],
  "mr_instances": [ … see §10 … ]
}
```

- Cell `v` holds a literal constant, `f` a formula (stored *without* `_xlfn.`; the writer
  adds it), `dyn` marks a dynamic-array formula, and `cse` marks a legacy array formula.
- The canonical form has sorted keys, cells in row-major order, numbers as shortest
  round-trip strings, and the content hash taken over this form (§12). xlsx bytes are
  **not** hashed: zip timestamps and writer versions vary.

---

## 5. Type-directed expression generator

### 5.1 Algorithm

```
gen_expr(want: TypeSet, ctx: Ctx, budget: Budget) -> Expr
  if budget.depth == 0 or budget.size <= 1 or coin(p_leaf(ctx)):
      return gen_leaf(want, ctx)                      # literal | cell ref | range ref | name
  prods = productions_returning(want, ctx)            # functions, operators, conditional forms
  prods = filter(prods, milestone_mask, swarm_mask, version_mask)
  p     = choose_weighted(prods, weight(·, ctx))      # §5.3
  args  = []
  for param in expand_arity(p, ctx):                  # samples optional / repeat counts
      want_i = param.types                            # possibly narrowed by domain
      if param.mode == ref_only: args.append(gen_ref(param, ctx)); continue
      if param.domain and not coin(ctx.p_violate):
          args.append(gen_in_domain(param, ctx, budget.split()))  # constraint solving / rejection
      else:
          args.append(gen_expr(want_i ∪ edge_types(param), ctx, budget.split()))
  check_cross_arg_constraints(p, args) or repair(p, args)
  return Call(p, args)
```

- **Grammar coverage.** Productions include all operators: `+ - * / ^ & = <> < > <= >=`,
  unary `-`/`+`, postfix `%`, reference operators `:` (range), ` ` (intersection),
  `,` (union, inside parens), `@` (implicit intersection), `#` (spill ref), array literals
  `{1,2;3,4}`, `LET`/`LAMBDA` binding forms, defined names, structured refs (later
  milestone), and 3D refs `Sheet1:Sheet3!A1`.
- **Budgets.** `depth` is sampled from a mixture: 70% from the realistic distribution
  (geometric, mean ≈ 2, matching the corpus median parse-tree depth of 2) and 30% from a
  stress distribution (uniform 3..12, with occasional 64 = Excel's nesting limit). `size`
  is a node budget (default 25, stress up to 400), split among children with a Dirichlet
  draw. Formula-text length is capped at 8,192 characters and arg count at 255.
- **Type narrowing.** When a parameter accepts `number`, the generator may still produce
  `text` or `logical` (coercion probes) with probability `p_coerce` (default 0.1), labeled
  with edge class `coerce:<from>-><to>`.
- **Leaves.** Each leaf is a literal from the type pools (§6), a reference into an existing
  data table (§7), a reference to another formula cell (this builds DAG edges), or a name.
  `p_ref_leaf ≈ 0.6`, because real formulas mostly reference cells (median 2 preceding
  cells).
- **Valid-by-construction vs. error-seeking.** The YARPGen lesson is that unconstrained
  generation drowns in errors (`#VALUE!` everywhere tests only error propagation). We
  track the expected-error rate per probe with RustSheet as a cheap predictor and
  rejection-sample to keep about 70% non-error results, about 20% intended errors, and
  about 10% "don't care".

### 5.2 Probe shapes

Each probe is a single formula cell, plus optional sidecar cells. Shapes:

1. **Scalar probe**: `=f(args…)` over literals and refs.
2. **Aggregate-over-range probe**: `=AGG(range)`, with the range filled from a
   mixed-type column generator (§7.3).
3. **Lifted probe** (dynamic arrays): `=f(range_or_array)` that spills. The probe region
   reserves a `rows×cols` empty footprint to avoid unintended `#SPILL!`. Spill
   collisions are generated on purpose under edge class `spill_blocked`.
   *Implemented* (T-2224) as `rustsheet-gen --spill`, off by default, so existing suites
   regenerate byte for byte. A formula whose outer call is `dynamic_array: true` in the
   catalog gets its anchor in column A followed by an empty footprint of
   `min(12, data rows) × min(6, data columns)` cells, starting at the anchor
   (`SPILL_ROWS`, `SPILL_COLS` in `gen/generator.py`). Other formulas take one row, and no
   formula sits to the right of column A, so no footprint overlaps a formula or another
   footprint. The case's `check` lists the anchor and every footprint cell. The oracle records
   the cells Excel fills, and those are scored. A footprint cell Excel leaves empty is not in
   the sidecar, so the spill-extent check (T-1660) fails any engine value there against
   empty. A result taller than its footprint is `#SPILL!` on the formula below it in both
   Excel and the engine. A wider one spills right into unchecked cells: the extent check
   fails an engine value where Excel left the cell empty, but Excel's values there are not
   compared. `SPILL_BLOCKED_RATE` (5%) of footprints get a blocker constant just
   below or just right of the anchor. The blocker is an input, so it is not checked, and the
   case is tagged `spill_blocked`. The layout draws from its own seeded stream, so the
   formulas are the same with and without `--spill`.
4. **Implicit intersection probe**: `=@A1:A10` or legacy-style `=A1:A10` written as a
   non-DA formula on row k. Expected to return row k's value or `#VALUE!` outside the
   range.
5. **Chain probe**: a k-deep DAG of formula cells (§7.4).
6. **Fill-down block**: one R1C1 template copied over N rows (§7.5).

### 5.3 Weighting: priors × gaps × scope

For a production `p` in context `ctx`:

```
w(p) = prior(p)^α                       # α≈0.5 tempers corpus skew (SUM would otherwise dominate)
     · gap(p, ctx)                      # adaptive, bandit-derived, ≥ 1
     · policy(p)                        # YARPGen-style generation policy multiplier
     · scope(p)                         # 1 if milestone-enabled, 0 otherwise (ε for lookahead)
     · swarm(p)                         # 0 if disabled in this workbook's swarm mask
```

**Adaptive gap term (Thompson sampling).** The bandit arms are coverage-matrix cells
`(function, arg_position, arg_type_class, edge_class)`, rolled up to function level for
sampling. Each arm keeps `Beta(a, b)`:
- `a += 1` when a probe touching the arm produced a **new** diff signature or an MR
  violation (a bug "reward").
- `b += 1` when it agreed with the oracle or MRs.
- Arms that were never pulled get an exploration bonus: a `Beta(1,1)` prior plus an
  under-coverage multiplier `1 + κ / sqrt(1 + n_pulls)`.
- At workbook start, sample `θ_arm ~ Beta(a,b)` for all arms and set
  `gap(p) = 1 + λ · mean θ` over the arms `p` participates in.
- **Decay.** Multiply `a,b` by γ=0.98 per generation round, so that after RustSheet fixes
  a bug the arm stops being over-sampled (non-stationary bandit). Arms whose bugs are
  filed and still open get **damped** (×0.2), so we don't keep rediscovering known issues.
  Dedup (§9.5) feeds this.

**Failure-rate vs. novelty.** Reward only *new* signatures. Otherwise the bandit
collapses onto one known-broken function. The current failure rate is surfaced
separately in the dashboard.

### 5.4 Milestone scope and swarm

- `scope(p)` is 1 for functions with `milestone ≤ current`. Lookahead functions (next
  milestone) get ε=0.02 so the dashboard can show readiness, and their diffs are labeled
  `lookahead` and never filed as regressions.
- **Version mask**: optionally limit to functions `introduced ≤ X` (e.g. "Excel 2016
  parity").
- **Swarm mask** per workbook: each feature (function, operator, value class, structural
  feature such as cross-sheet refs or names) is independently disabled with probability
  0.5, subject to the mask leaving at least 5 functions enabled [3]. The mask is recorded
  in the IR for reproducibility and coverage attribution.
- **Focus mask** (`rustsheet-gen --focus`, T-2108): every formula's outermost call is a
  generator-ready function whose catalog milestone is exactly the target. The functions take
  turns through a seeded shuffle (reshuffled per case), so a run gives each about the same
  number of outer calls; arguments are still drawn from everything in scope at or below the
  milestone. Without it, the sqrt(prior) weighting lets high-prior earlier-milestone functions
  crowd out the target's own. A milestone with no such function is an error.

### 5.5 Generation policies (anti-saturation)

Each workbook draws one policy. The policy multiplies category weights and pool choices:

| Policy | Emphasis |
|---|---|
| `realistic` | pure priors (α=1), shallow, scenario-driven. Our regression baseline. |
| `numeric_fp` | arithmetic chains, cancellation, ROUND/TRUNC/MOD/INT, 15-digit boundaries |
| `coercion` | text↔number↔bool, blanks in every position, `&` concatenation of numbers |
| `error_prop` | errors in args/ranges; IFERROR/IFNA/ISERROR/ERROR.TYPE/AGGREGATE |
| `lookup` | VLOOKUP/HLOOKUP/MATCH/INDEX/XLOOKUP/XMATCH, sorted/unsorted, wildcards, approx |
| `dates` | DATE/EDATE/EOMONTH/NETWORKDAYS/WEEKDAY/WEEKNUM/DATEDIF/YEARFRAC around serial 0..61 and year edges |
| `text` | LEFT/MID/FIND/SEARCH/SUBSTITUTE/TEXT/VALUE/PROPER, unicode, surrogate pairs |
| `arrays` | DA functions, lifting, broadcasting mismatched shapes, `#` refs, SEQUENCE/FILTER/SORT/UNIQUE |
| `refs` | OFFSET/INDIRECT/INDEX-as-ref/union/intersection/3D refs/names |
| `stats_fin` | statistical and financial functions with domain-edge params |
| `structure` | deep DAGs, wide fan-in/out, cross-sheet chains, insert/delete MRs |

Implemented so far (`rustsheet-gen --policy`): `random` (the catalog-driven P0 generator),
`numeric-rules` (dense floating-point probes, suite `probes-numeric`), `operators-refs` (M1
operators and references only, no functions, suite `gen-m1`; T-0117), `names-3d` (suite `gen-m8a`,
T-1606), `tables` (suite `gen-m8b`, T-1607), `number-formats` (suite `gen-m10`, T-1611) and
`numeric-fp` (suite `gen-m2`, T-3139). All but `random` are whole-suite policies rather than
per-workbook draws.

`numeric-fp` implements the cancellation part of the `numeric_fp` row for M2 and M5: X = base·(1±k·2^-52)
against b = base (bases 1, 2, 0.5, 2^20, 1.5; k = 0..9; both signs; exact doubles built on an
`Inputs` sheet with `numeric_rules`' `BASES` and `fmt_k`) in three positions, each case one
position: `root` (the `+`/`-` at the root, operands read through VLOOKUP, HLOOKUP, INDEX,
MATCH-fed INDEX, MIN, MAX, ABS, ROUND and IF), `wrapper` (the cancellation under IF, IFERROR,
ROUND, ABS, `&`, `=`, AND and COUNTIF) and `sum` (SUM, AVERAGE, SUMIF and SUMIFS over a range of X
and -b, with and without a trailing 0), all in both operand orders: 5,200 cells, each tagged
`numeric-fp/<position>/<form>/<order>/base=../sign=../k=..`. The seed only shuffles the cells.
With `--milestone M5` (suite `gen-m5`, T-3204) it writes three more cases over the same pairs: `root` reads
both operands through TRUNC (with and without 20 digits), INT, `MOD(..,2^21)`, ROUNDUP and ROUNDDOWN (20 digits);
`wrapper` puts `X-b` under TRUNC (both forms), INT, `MOD(..,1)`, ROUNDUP (20 and 0 digits) and ROUNDDOWN (20); and
`digits15` takes, per base and sign, the k one step either side of where X's rounded or truncated 15-significant-digit
cut stops being b (computed exactly with `decimal`), with every picker (plain references included) and wrapper, each
cell tagged `round15=eq|ne/trunc15=eq|ne`: 4,252 cells. Other milestones are refused.

---

## 6. Value generators (per-type edge pools)

Every pool entry has an `edge_class` id. A literal is drawn as
`P(edge pool) = 0.35, P(realistic) = 0.65`, adjusted by policy.

**Implemented (T-2109).** A catalog `value` parameter with `pool: <name>` draws from
`tools/src/rustsheet_tools/gen/pools.py` 35% of the time, after its `domain` draw and never on a `hard`
domain; other parameters still build a free expression. The pools there are `number_edges`, `unit_interval`,
`angle` and `code_point` (T-2109), and `text_edges`, `numeric_text`, `number_formats` (the codes of
`number_formats.py`, quoted) and `errors` (T-2110). T-2286 added `array_shape`, `array_holes`, `lookup_value`,
`lookup_table`, `regex_pattern` and `lambda_body` for the M6 and M7 lookup, array, REGEX and LAMBDA functions, with
two draw extensions: a pooled `range` parameter (LOOKUP's lookup_vector, XMATCH's lookup_array) takes a pool entry
35% of the time in place of a range, and a pooled `lambda` parameter takes an entry with the parameter count the call
needs (MAP's one per array), its `x` and `y` renamed to free names.
T-2247 added `probability`, `shape`, `stat_x`, `gamma_x`, `flag` and `stat_data` for the M9 statistical functions
(probabilities and alphas, degrees of freedom and shapes, distribution x and z, GAMMA's x, the cumulative/const/stats
flags, and the data arrays of the aggregates, order statistics and paired functions) and put FISHER's x on
`unit_interval`; no draw change was needed, as an `array` or `aggregate` parameter's non-range draw already reaches
the pool.
T-2438 added `count` (the integer counts of the binomial, hypergeometric, CONFIDENCE and LARGE functions, with their
soft `int` domain kept), `ets_target`, `ets_seasonality`, `ets_completion`, `ets_aggregation`, `ets_confidence`,
`ets_statistic`, `ets_timeline` and `ets_values` array constants (the first entries are 24 sorted, evenly spaced points
beside 24 values; the two are drawn independently, so a matching pair comes up by chance, the rest are mismatches), `beta_bound`, `fisherinv_y`, `forecast_x` and `lognorm_mean`.

### 6.1 Numbers
- **Zeros & signs**: `0`, `-0` (Excel has no negative zero: it must normalize),
  `1`, `-1`, `0.5`, `-0.5`.
- **Integers at boundaries**: `2^31−1`, `2^31`, `2^32`, `2^53−1`, `2^53`, `2^53+1` (not
  representable), `1e15`, `999999999999999` (15 nines), `9999999999999999` (16 nines,
  display and rounding), `1e21` (display switches to scientific).
- **Extremes** [20]: `1.79769313486232E+308` (Excel's documented max; compute it or store it as a
  cell value: a formula *literal* ≥ 1E+308 makes Excel reject the workbook, see
  `docs/excel-semantics/numbers.md`),
  `2.2250738585072E-308` (min positive normal; Excel does **not** support denormals),
  `1e-307`, `5e-324` (if it can be produced by computation: must flush to 0), and
  overflow producers (`1e307*100` → `#NUM!`).
- **FP traps**: `0.1`, `0.2`, `0.3`, `1.1`, `43.1-43.2` (the Microsoft example: displays
  `0.899999999999999` for `=(43.1-43.2)+1`), `0.000123456789012345` (the 15-significant-
  digit truncation example), `1.333+1.225-1.333-1.225` (close-to-zero compensation since
  Excel 97), `1/3`, `2/3`, `1e-15`, `1+1e-15`, `1+2^-52`.
- **Rounding-half cases**: `0.5, 1.5, 2.5, 2.675, 1.005, 0.285, -2.5` (ROUND is
  away-from-zero on the *decimal* representation, so `ROUND(2.675,2)` is a known
  divergence trap).
- **Close-to-zero snapping families**: generate `a + b` with `b ≈ −a·(1 ± k·2^-52)`
  for k ∈ {0..8}, as top-level ops and parenthesized. Record whether parentheses or
  being the final operation change the result: `=(1+2^-52-1)` keeps the tiny value
  while the unparenthesized form snaps [21].
- **Date serials**: `0` (shows 1900-01-00), `1`, `59`, **`60` (the fictitious
  1900-02-29)**, `61`, `366`, `367`, `2958465` (9999-12-31), `2958466` (out of range),
  negatives, fractional times `0.5`, `0.999988426` (23:59:59), `1/86400`.
- **Realistic**: log-uniform magnitudes 1e-3..1e7, currency (2 decimals), percentages,
  small ints, *dyadic rationals* `m / 2^k` with |m| < 2^20, k ≤ 10. Dyadic values are
  exact in binary, so sums are order-independent. MRs that need exact arithmetic use these
  (§10.1).

### 6.2 Text
- **Empty and whitespace**: `""`, `" "`, `"\t"`, `"\n"`, NBSP `" "`, `"　"`,
  zero-width `"​"`, leading and trailing spaces `" 42 "`.
- **Numeric-looking** (coercion targets for `VALUE`, `+0`, `--`, and comparisons):
  `"42"`, `" 42 "`, `"4.2e1"`, `"1E3"`, `"1,000"`, `"$1,000.50"`, `"(100)"` (accounting
  negative), `"50%"`, `"-0"`, `"0x1F"` (not numeric), `"١٢٣"` (Arabic-Indic digits),
  `"TRUE"` (does not coerce to a number), `"1/2"` (parses as a **date in the current
  year**: clock-dependent, flag `nondeterministic`), `"2024-02-29"`, `"1900-02-29"`,
  `"12:30"`, `"12:30 PM"`, `"25:00"`.
- **Case & collation**: `"a"`, `"A"`, `"ä"`, `"ß"`, `"İ"`, `"-a"`, `"'a"`, `"a-b"` vs
  `"ab"` (Excel's string comparison is case-insensitive and culture-aware, not codepoint
  order).
- **Unicode**: combining marks (`"é"` vs `"é"`), astral characters such as emoji
  `"😀"` (UTF-16 surrogate pair: LEN returns 2), RTL text, very long strings (32,767 is
  the cell character limit; 255 is the limit for string literals inside formulas).
- **Wildcards & criteria syntax** (for `*IF*`, MATCH, SEARCH, XLOOKUP mode 2): `"*"`,
  `"?"`, `"~*"`, `"~?"`, `"~~"`, `">5"`, `">=5"`, `"<>"`, `"="`, `"<>a"`, `">"&TODAY()`.
- **Format strings** (TEXT, number formats): `"0.00"`, `"#,##0"`, `"0%"`, `"0.00E+00"`,
  `"yyyy-mm-dd"`, `"[h]:mm"`, `"mmmm"`, `"@"`, `"General"`, `"0;-0;\"zero\""`,
  `"[Red]0"`, `"# ?/?"`.
- **Error-looking text**: `"#N/A"` as text (it is *not* an error), `"=1+1"` as text.

### 6.3 Logical
`TRUE`, `FALSE`, as literals, in cells, and produced by comparisons. Coercion edge classes:
bool in arithmetic (`TRUE+1=2`), bool in a range under SUM (ignored), bool vs. text
comparison ordering.

### 6.4 Blanks
These are first-class and distinct from `""`:
- a truly empty cell referenced directly (`=A1` returns 0, in text context `""`),
- an empty cell inside ranges (skipped by COUNT/AVERAGE, counted by COUNTBLANK),
- a cell containing a formula that returns `""` (*not* blank for COUNTA and ISBLANK, but
  counted by COUNTBLANK),
- omitted args `f(a,,c)` versus missing trailing args `f(a,b)`,
- `IF(cond,A1)` where A1 is blank.

### 6.5 Errors
All error values, appearing as literals (`#N/A`, `NA()`, `1/0`), in cells, and inside
ranges at the first, last and middle positions. Ranges with **two different errors**
test which error wins (the first in evaluation order). Edge class `error_order` covers
`=#N/A+#DIV/0!` and the reverse. A number- or bool-typed `value` parameter that has no `hard` domain and was
not drawn from its edge pool receives `""` with share `EMPTY_TEXT_RATE` (3%) and an entry of the `errors`
pool with share `ERROR_RATE` (3%), so generated suites reach the `empty_text` and `error` argument kinds
(T-2866; decision 0094).

### 6.6 Arrays and references
- Array literals: `{1,2,3}`, `{1;2;3}`, `{1,2;3,4}`, mixed-type `{1,"a",TRUE,#N/A}`,
  `{}`-adjacent invalid forms (parse errors, used only in parser fuzzing).
- **Broadcast shapes**: `(1×n op n×1)` → n×n; `(m×n op m×1)` broadcast; mismatched
  `(2×3 op 3×2)` → `#N/A` padding; scalar op array.
- References: single cell, rows `1:1`, columns `A:A`, full-sheet (huge: watch the cost),
  single-row and single-column ranges, ranges with a reversed corner `B5:A1` (normalized),
  mixed absolute/relative `$A1`, `A$1`, `$A$1`, unions `(A1:A3,C1:C3)`, intersections
  `A1:C3 B2:D4`, an empty intersection → `#NULL!`, 3D `Sheet1:Sheet3!A1`, refs to empty
  sheets, `A1#` spill refs, references at sheet limits (`XFD1048576`).

---

## 7. Workbook-level structure

### 7.1 Layout

Default: `Data` sheet(s) with realistic tables, `Calc` sheet(s) with probes and chains,
an optional `Probe` sheet for subexpression sidecars (§9.3), and an optional `Lookup`
sheet. The sheet count is sampled from {1: 50%, 2–3: 40%, 4–8: 10%}, skewed to small,
matching the corpus median of 1 worksheet. Sheet names include spaces, quotes, unicode and
names that look like cell refs (`'A1'`, `'R1C1'`, `'TRUE'`) to exercise quoting.

As built (T-2396): a case that calls GETPIVOTDATA (catalog `pivot: true`) gets a third sheet, `Pivot`, with a seeded
source block and one compact-form pivot over it (spec `pivot_tables`); every GETPIVOTDATA call in the case queries
that pivot, with deliberate `#REF!` misses (tools/src/rustsheet_tools/gen/pivots.py).

### 7.2 Realistic data tables

A table is a list of **column archetypes**, each a generator:

| Archetype | Generator |
|---|---|
| `id` | sequential int, sometimes with gaps or duplicates (lookup traps), sometimes text ids `"INV-00042"` |
| `date` | business days within a range, sorted or shuffled; occasional text-dates (`"2024-03-01"` as text) |
| `amount` | log-normal currency with 2 decimals; negatives (refunds); occasional 0; dyadic option |
| `qty` | small Poisson ints |
| `category` | Zipf over 3–20 labels with case/whitespace variants (`"East"`, `"east"`, `"East "`) |
| `flag` | bool, or "Y"/"N" text |
| `pct` | 0..1 with 4 decimals |
| `name` | synthetic person or product names with unicode |
| `code` | numeric-looking text with leading zeros (`"00123"`) |

**Dirt injection** (per table, probability-controlled) sets the mixed-type ratio:
blanks, numbers-as-text, an error cell, a stray header row repeated mid-table, and a
total row at the bottom (classic double-count trap for whole-column refs). Tables are
1–200 rows typically, occasionally 10k for performance and big-range semantics.

### 7.3 Ranges with mixed types

A `mixed_range(n, mix)` generator samples cell kinds from a Dirichlet over
{number, text, numeric-text, bool, blank, error, formula-returning-""}. It is applied
specifically to aggregate probes (SUM, AVERAGE, COUNT*, MIN/MAX, MEDIAN, SUMPRODUCT,
CONCAT, TEXTJOIN). This is where `values_agg` vs `values_agg_a` vs direct-arg coercion
differences show up.

### 7.4 Dependency chains (DAG testing)

- **Chain**: `C1 = f(data)`, `C2 = g(C1)`, … with depth `k ~ {1..5 typical, 50, 500, 5000}`.
  Deep chains stress recursion limits and evaluation order.
- **Diamond / fan-in / fan-out**: fan-in up to 255 refs; fan-out where one input feeds
  100 formulas.
- **Cross-sheet chains**: hops alternate sheets; include 3D refs and refs into sheets
  whose names need quoting.
- **Conditional dependencies**: `IF(flag, A, B)`, `CHOOSE`, `INDIRECT("A"&n)` and `OFFSET`
  create *dynamic* dependencies that a static dependency graph can get wrong. This is
  essential for incremental-recalc MRs.
- **Cycles**: rare (≤1% of workbooks) and only with iterative-calc settings
  generated deliberately. The oracle checks Excel's behavior (circular-reference warning,
  0 values) versus ours.
- **Topology control**: generate the DAG first (random layered DAG with parameters
  depth, width, edge density and cross-sheet probability), then fill nodes with
  type-compatible expressions whose leaf refs equal the DAG predecessors.

### 7.5 Fill-down and relative/absolute mixes

FUSE's most common formula, `=SUM(R[-3]C:R[-1]C)`, is a fill pattern [16]. The generator
makes **R1C1 templates** and copies them over regions:
- `=B2*$F$1` filled down 50 rows (relative row, absolute rate);
- running totals `=C2+D3` (anchored previous row);
- `=SUM($B$2:B2)` (expanding range);
- `=VLOOKUP($A2, Lookup!$A$1:$D$100, COLUMN()-1, FALSE)` filled across and down;
- mixed `$A1` / `A$1` multiplication tables.

The IR stores the expanded A1 formulas (xlsx stores A1 formulas, and shared formulas are
optional; the writer may emit `<f t="shared">` to test our shared-formula reader).

### 7.6 Other structural features (milestone-gated)
Defined names (workbook and sheet scope, names shadowing functions), merged cells
(no effect on values), hidden rows (SUBTOTAL 1xx vs 0xx, AGGREGATE), number formats
(affect `TEXT`/`CELL` but not values), Excel tables with structured references,
LET/LAMBDA names, spills referencing spills, `#SPILL!` collisions.

---

## 8. Scenario templates

Templates are parameterized "realistic workbook" generators: Python functions that compose
tables (§7.2) and fill-down formula blocks (§7.5) with **holes** that the type-directed
generator fills. Realism comes from the template; stress comes from the holes.

1. **Financial model (3-statement-lite)**: monthly columns × line items; growth
   `=B5*(1+$B$2)`; `NPV`, `IRR`, `XNPV`, `PMT`, `IPMT`, `PPMT`, `FV`, `RATE` on generated
   cashflows (including all-negative cashflows → `IRR` `#NUM!`); `ROUND` everywhere;
   `SUMPRODUCT` weighting; scenario switch via `CHOOSE($B$1,…)`.
2. **Gradebook**: students × assignments with blanks (missing work), text such as
   `"excused"`, weights; `AVERAGE` vs `AVERAGEA`, `SUMPRODUCT/SUM`, `RANK.EQ`,
   `PERCENTILE`, `IFS` letter-grade ladders, `COUNTIF(">=90")`, `LOOKUP` approximate
   bands, `MEDIAN`, `STDEV.S`.
3. **Inventory lookup**: SKU table (text ids with leading zeros, numbers stored as text),
   orders sheet with `VLOOKUP`/`INDEX-MATCH`/`XLOOKUP`, `IFERROR(...,"missing")`,
   `SUMIFS` stock by warehouse/date, reorder flags with `AND/OR`, wildcard SKU search.
4. **Date schedules**: project plan with `WORKDAY`/`NETWORKDAYS` (+ holiday range),
   `EDATE`/`EOMONTH` amortization dates, `WEEKDAY` with each `return_type`, `WEEKNUM` vs
   `ISOWEEKNUM`, `DATEDIF` all units, `YEARFRAC` all bases, dates around 1900-02-29,
   leap years 2000/2100, end-of-month rollovers (`EDATE("2024-01-31",1)`).
5. **Sales dashboard**: transaction log → `SUMIFS`/`COUNTIFS`/`AVERAGEIFS` by
   category × month, `UNIQUE`/`SORT`/`FILTER` spills, `SUBTOTAL` with hidden rows,
   `TEXT(date,"mmm")` grouping, percentage of total.
6. **Loan amortization**: `PMT` row plus 360-row fill-down of interest and principal,
   cumulative `CUMIPMT`, balance checks. A natural MR is that the ending balance ≈ 0, and
   `SUM(principal)` = loan within tolerance.
7. **Text cleanup sheet**: `TRIM`/`CLEAN`/`SUBSTITUTE`/`PROPER`/`TEXTSPLIT`/`TEXTJOIN`/
   `REGEXREPLACE` on dirty names and codes.

Template selection weight: about 40% of workbooks are scenario-based (realism) and 60%
are probe grids (density). Scenario workbooks are also the regression suite that gets
exact-match gating in CI.

---

## 9. Oracle pipeline and diffing

### 9.1 Packing
- One workbook holds **many independent probes** (1–5k formula cells). Probes are
  independent unless they are part of a chain, so one probe's failure never poisons
  another. Spill probes get reserved empty footprints.
- **Canaries** on every sheet [13]: `=1111+2222` (must be 3333) proves recalculation
  happened. `=NOW()` is recorded and must be fresh. `=INFO("release")` or
  `=INFO("osversion")` records the environment. The Excel build string is also captured
  from the automation API.
- Volatile functions (`NOW`, `TODAY`, `RAND`, `RANDBETWEEN`, `RANDARRAY`, `OFFSET`,
  `INDIRECT`, `CELL`, `INFO`) follow `oracle_policy`. NOW/TODAY/RAND results are compared
  by type and range only. `OFFSET`/`INDIRECT` are volatile but deterministic, so they are
  compared exactly.

### 9.2 Running Excel
Options, all recording `excel_build`:
1. **Excel desktop on a Windows VM** via COM (xlwings / pywin32): open the file,
   `Application.CalculateFull()` (or `CalculateFullRebuild()`), read `Range.Value2` and
   `Range.Formula2`, save as xlsx. Formula2 is captured to detect any rewriting Excel did,
   for example an inserted `@`. This is the gold standard. Pin the build and channel
   (e.g. Monthly Enterprise) and **re-baseline** on upgrades.
2. **Excel for the web via Microsoft Graph**: upload, then
   `POST /workbook/application/calculate {calculationType: "FullRebuild"}` [22], then read
   used ranges. This scales horizontally, but is a different engine build. Use it only for
   pre-screening, and label results `engine=excel-web`.
3. Read back cached values directly from the saved xlsx `<v>` elements (as IronCalc's
   tests do). This is cheapest and gives values at full double precision, but loses
   spill-range metadata unless we parse `metadata.xml`.

### 9.3 Subexpression probing (oracle amortization)

For each probe, optionally (default for depth ≥ 3) emit **sidecar cells** on a `Probe`
sheet, one per non-leaf AST node, each containing exactly that subexpression (with
references made absolute). One oracle run then gives Excel's value for **every node**.
When RustSheet disagrees at the root, the culprit is the **deepest node where RustSheet ≠
Excel while all its children agree**, found with no extra oracle calls. Caveats: a
subexpression evaluated in its own cell can change semantics (implicit intersection,
array lifting, `@`, reference-returning functions such as `INDEX`/`OFFSET` used as refs).
The catalog's `mode` tells us when a node is reference-valued. Those are probed as
`=ISREF(…)`, `=ROWS(…)`, `=COLUMNS(…)`, `=CELL("address",…)` instead of by value.

### 9.4 Diff classification
Compare per cell in order:
1. **Type** (number/text/bool/error/blank/array-shape). A mismatch is `type_mismatch`.
2. **Error code**: an exact match is required (`#VALUE!` vs `#NUM!` is a real bug).
3. **Text**: exact codepoint equality.
4. **Number**:
   - bitwise equal → pass;
   - ≤ 4 ULP apart → `fp_ulp` (low severity, tracked; Excel parity is the goal, so not ignored);
   - equal when both are rendered at 15 significant digits → `fp_15digit` (medium; often
     summation order or a snapping difference);
   - otherwise → `value_mismatch` (high).
5. **Spill**: shape mismatch is `spill_shape`. `#SPILL!` vs a value counts as a
   `type_mismatch`.
6. Formula text normalization: if Excel's `Formula2` differs from the written formula
   (an `@` was inserted, or a function was renamed), log `writer_or_semantics` and check
   our writer first.

### 9.5 Bucketing (dedup)
A **signature** is `(diff_class, culprit_function, culprit_arg_type_classes,
culprit_edge_classes, excel_value_kind, ours_value_kind)`, taken from the culprit node in
§9.3. All diffs with the same signature form one bucket, and one task is filed per bucket
with the smallest repro. The signature feeds the bandit's novelty reward (§5.3).

---

## 10. Metamorphic relations (no oracle needed)

Each MR instance is generated with the workbook and stored in `mr_instances`. Both sides
are formula cells, plus a check cell, or the check runs in the Rust MR runner, which
evaluates both and compares. MRs are typed by **exactness**:
- `exact`: requires dyadic or integer data so FP order cannot matter;
- `fp_tol(k ulps)`;
- `same_type_and_error`.

### 10.1 Algebraic and aggregate MRs
| ID | Relation | Notes |
|---|---|---|
| MR-ADD | `SUM(A1:A10) = SUM(A1:A5)+SUM(A6:A10)` | exact on dyadic data. The same split works for COUNT, COUNTA, COUNTBLANK, MAX(max of parts), MIN, PRODUCT |
| MR-UNION | `SUM(A1:A10) = SUM((A1:A5,A6:A10))` = `SUM(A1:A5,A6:A10)` | union-ref handling |
| MR-PERM | aggregates invariant under row permutation of the data (SUM, AVERAGE, MEDIAN, MODE.MULT set, PERCENTILE, STDEV, LARGE/SMALL, COUNTIF) | exact on dyadic data only; on general data `fp_tol` |
| MR-SCALE | `SUM(k*A1:A10) = k*SUM(A1:A10)` with k a power of 2 | exact; also exercises array lifting |
| MR-TLP | `COUNTIF(r,">"&t) + COUNTIF(r,"<="&t) = COUNT(r)` (numeric). For mixed ranges add `COUNTIF(r,"<>"&…)` and the non-numeric parts so the partitions sum to `COUNTA(r)+COUNTBLANK(r)=ROWS*COLS` | Ternary partitioning analogue [5]: numeric / non-numeric / blank |
| MR-TLP-SUMIFS | `SUMIFS(v,c,"A") + SUMIFS(v,c,"<>A") = SUM(v)` (ignoring text) | criteria engine |
| MR-AVG | `AVERAGE(r) = SUM(r)/COUNT(r)` (fp_tol) and `AVERAGEA(r)=…/COUNTA(r)` with bool→1, text→0 | aggregation modes |
| MR-ROUND | `ROUND(x,n) = ROUND(x,n)` across equivalent spellings: `MROUND`, `ROUNDUP`/`ROUNDDOWN` bracketing, `FIXED` → `VALUE` | `exact` |
| MR-DATE | `DATE(YEAR(d),MONTH(d),DAY(d)) = INT(d)` for d ∈ [61, 2958465]; `EDATE(d,12)=DATE(YEAR(d)+1,…)` except Feb 29; `DATEVALUE(TEXT(d,"yyyy-mm-dd"))=INT(d)` | 1900 bug: d ∈ [1,60] treated separately |
| MR-TEXT | `LEN(a&b)=LEN(a)+LEN(b)`; `LEFT(s,n)&MID(s,n+1,LEN(s))=s`; `UPPER(LOWER(s))=UPPER(s)`; `SUBSTITUTE(s,x,x)=s` | unicode pools |
| MR-BOOL | `AND(a,b)=NOT(OR(NOT(a),NOT(b)))`; `IF(c,x,y)=IF(NOT(c),y,x)` | |

MR-UNION is implemented (T-1149) as a seeded Rust unit test in the engine
(`crates/rustsheet-engine/src/test/test_union_sum_metamorphic.rs`), not as a gen-m1 policy: it
needs no oracle and no xlsx (gen-m1 policies emit xlsx cases scored through the eval pipeline against Excel values), so it runs in `make check` (`cargo test`) with 120 pairs
`SUM((r1,r2[,r3]))` / `SUM(r1,r2[,r3])` over single cells, ranges, whole columns and duplicated
(overlapping) operands. Skipping an area in `for_each_position_in_areas` fails it.

### 10.2 NoREC-style "two implementations" MRs
These compare an optimized or specialized function with a naive formulation that probably
goes through a different engine code path:
- `SUMIFS(v,c,k)` ⇔ `SUMPRODUCT((c=k)*v)` (careful: criteria strings use wildcard and
  coercion semantics that differ from `=`, so the generator limits `k` to non-wildcard
  exact numbers or text and accounts for case-insensitivity)
- `VLOOKUP(k,T,j,FALSE)` ⇔ `INDEX(T, MATCH(k, INDEX(T,0,1), 0), j)` ⇔ `XLOOKUP(k, col1, colj)`
- `COUNTIF(r,k)` ⇔ `SUM(--(r=k))` (same caveats)
- `SUMPRODUCT(a,b)` ⇔ `SUM(a*b)` (dynamic-array lifting)
- `MAX(r)` ⇔ `LARGE(r,1)` ⇔ `INDEX(SORT(r,,-1),1)`; `MEDIAN` ⇔ `PERCENTILE.INC(r,0.5)` ⇔ `QUARTILE.INC(r,2)`
- `MATCH(k,r,1)` on sorted data ⇔ `XMATCH(k,r,-1)` ⇔ linear scan via `COUNTIF(r,"<="&k)`
- `TEXTJOIN(",",TRUE,r)` ⇔ `CONCAT` with manual separators skipping blanks
- **Literal vs reference** (expected-difference MR): `SUM("1",2)` vs `SUM(A1,A2)` with
  A1=`"1"`. The catalog `mode` says these *should* differ (3 vs 2). The MR checks that
  RustSheet gives the expected difference, which encodes Excel's documented coercion
  rules as executable properties.

### 10.3 EMI-style dead-code mutation
For `IF(c, X, Y)` where `c` is constant-evaluable, mutate the untaken branch (replace it
with a random same-type expression, including one that errors) and expect an unchanged
result. Excel's `IF`/`IFS`/`CHOOSE`/`SWITCH`/`IFERROR` are lazy, so an error in the
untaken branch must **not** propagate. Note that `CHOOSE` with an array index evaluates
multiple branches, which is a good edge class.

### 10.4 Structural MRs (engine-level, Rust runner)
| ID | Relation |
|---|---|
| MR-INSROW | Insert k rows/cols at position p (above, inside, at the boundary of, and below referenced ranges). Every formula cell's *value* is unchanged and every reference shifted per Excel's rules (range grows when inserting strictly inside; edge insertions do not grow the range). Combine with oracle runs at boundaries, because the expected *formula text* is Excel-specific [12]. |
| MR-DELROW | Delete rows not referenced → values unchanged. Delete referenced rows → `#REF!` in the exact places Excel puts it. |
| MR-MOVE | Cut/paste a formula cell → value unchanged, refs unchanged. Copy/paste → relative refs translated (compare against the R1C1 template). |
| MR-RENAME | Rename a sheet (including to names that need quoting) → values unchanged, and formula text updated. |
| MR-ORDER | Evaluate the dependency graph in k random valid topological orders, plus with randomized hash seeds or thread counts → identical values (bitwise). |
| MR-INCR | Incremental vs full: apply a random sequence of cell edits, recalculate incrementally after each, then compare with a fresh full recalc of the final state, bitwise. Dynamic dependencies (`INDIRECT`, `OFFSET`, `INDEX` as ref, `CHOOSE`) and spills are oversampled here. **Highest-value MR for engine correctness.** |
| MR-ROUNDTRIP | Load xlsx → save → load → values and formulas identical. Also IR → xlsx → RustSheet ≡ IR → RustSheet directly. |
| MR-R1C1 | Formula text A1 → R1C1 → A1 is the identity. Copy over a region ≡ shared-formula expansion. |
| MR-PARENS | Adding redundant parentheses around non-top-level subexpressions does not change values, **except** at top-level `+`/`-` where Excel's close-to-zero compensation applies [21]. That exception is itself encoded as an expected-difference MR, and its exact rule is learned from the oracle (§15.2 phase P1 probes). |
| MR-LETINLINE | `LET(x, e, f(x,x))` ≡ `f(e,e)` for non-volatile e. LAMBDA beta-reduction likewise. |
| MR-NAME | Replacing a reference with a defined name pointing to it doesn't change values. |

### 10.5 Using MRs for triage
An MR violation is a bug in RustSheet for sure **only** if the MR holds in Excel. Each MR
family is validated against the oracle once, on a few hundred instances (a "MR
certification" run), and families that Excel itself violates are reclassified as
expected-difference or dropped. After certification, MR violations are filed without
any oracle call, and minimized with the MR as the interestingness test (DDMT) [11].

---

## 11. Minimization (automatic test-case reduction)

Interestingness predicate `I(wb)`. Either "RustSheet ≠ Excel on cell X with the same diff
signature", which needs the oracle, or "MR-k violated with the same signature", which does
not. The reducer is **signature-preserving**, so it does not slip to a different bug.

**Stages** (cheapest and biggest reductions first):
1. **Oracle-free localization** (§9.3): if sidecars exist, jump straight to the culprit
   node. Build the candidate repro `=F(v1,…,vn)`, where `vi` are Excel's values for the
   culprit's children (materialized as literals, or as cells holding those values when the
   argument mode is `ref_only` or `values_agg`, since literal vs cell matters).
   **Verify with one oracle call.** If it reproduces, we are almost done.
2. **Cone of influence**: keep only the target cell and its transitive precedents. Drop
   everything else (other probes, sheets, names). This is free because probes are
   independent.
3. **Workbook ddmin**: delta-debug over sheets → rows/columns of data tables → individual
   precedent cells (replace with blank, or with the oracle value if known).
4. **AST reduction (HDD/Perses)** on the culprit formula:
   - *Hoisting*: replace a node with a type-compatible descendant.
   - *Evaluation substitution*: replace a subtree with its Excel value (known from the
     sidecars). This is the strongest step.
   - *Arity reduction*: drop optional or repeat args.
   - *Ref simplification*: shrink ranges (ddmin over range rows/cols), convert cross-sheet
     to same-sheet, absolute to relative.
5. **Literal shrinking** (Hypothesis-style ordering): numbers toward `0, 1, -1`, small
   integers, fewer significant digits, and dyadic forms; strings toward shorter and ASCII;
   dates toward the year 2000. Each candidate must keep the signature.
6. **Choice-tape shrinking** (alternative path): because the generator is a deterministic
   function of its choice tape (Zest/Hypothesis design [6][10]), run Hypothesis-style tape
   shrinking (delete spans, zero blocks, sort, binary-search integers). Every candidate
   is a valid generator output.

**Oracle budget control.** Batch all candidates of a ddmin round into **one** workbook
(each candidate is an independent probe region), so one Excel call evaluates about 100
candidates. The reducer is parallel ddmin (complement tests in one batch). Expected
cost: 2–6 oracle calls per bug versus hundreds for naive ddmin. For MR-found bugs the
cost is 0.

**Output**: `repro.xlsx` and `repro.ir.json` with ≤ 5 cells where possible. The task body
holds: the formula, input cell values, Excel's value (raw hex double + 17-digit decimal +
15-digit display), RustSheet's value, diff class, signature, catalog entry, and seed and
generator version for regeneration. Example:
`=ROUND(2.675,2)` → Excel `2.68` vs RustSheet `2.67` · `value_mismatch` · `ROUND/num:fp_half/edge:half_even_vs_away`.

---

## 12. Determinism, naming, corpus layout

### 12.1 Seeds and reproducibility
- **Seed tree**: `campaign_seed` → `workbook_seed = H(campaign_seed, index)` →
  per-component substreams (`layout`, `tables`, `probes[i]`, `mr`) via a counter-based
  PRNG (Philox or ChaCha20 via `numpy.random.Generator(PCG64DXSM)` with `SeedSequence.spawn`).
  Regenerating one probe never requires regenerating the others.
- **Choice tape**: every random decision goes through `tape.draw(kind, bounds)`, and the
  tape is saved (compressed). Replay = regenerate from the tape. That gives exact
  reproduction even if PRNG internals change.
- **Pinned inputs**: `generator_version` (semver + git sha), `catalog_hash` (sha256 of
  the normalized catalog), `bandit_snapshot_id` (bandit weights affect choices, so they
  are part of the reproducibility key; the tape makes replay independent of them).
- **No ambient nondeterminism**: no wall clock, locale or environment in generation.
  `"1/2"`-style text and `NOW`/`TODAY` are flagged `nondeterministic` and handled by
  oracle policy.

### 12.2 Content-hash naming
`wb_id = "wb-" + base32(sha256(canonical_ir_json))[:20]`. Identical workbooks dedupe
automatically. Oracle results are keyed by `(wb_id, excel_build)`, so an Excel upgrade
naturally creates new oracle records without overwriting history.

### 12.3 Corpus layout

```
corpus/
  catalog/                      # copy of catalog at campaign time (or symlink + hash)
  campaigns/<campaign_id>/
    campaign.json               # seed, generator version, milestone, policies mix, budgets
    bandit/state-<round>.json   # Beta(a,b) per arm (snapshot per round)
  workbooks/<wb_id[0:2]>/<wb_id>/
    ir.json.zst                 # canonical IR (hash input)
    tape.bin.zst                # choice tape
    book.xlsx                   # as written (writer + version in manifest)
    manifest.json               # see below
    oracle/<excel_build>.json   # Excel results
    rustsheet/<rs_git_sha>.json # our results (same schema)
    diff/<rs_git_sha>__<excel_build>.json
  repros/<signature_hash>/
    repro.xlsx  repro.ir.json  oracle.json  task.md(generated)  history.jsonl
  coverage/
    matrix-<rs_git_sha>.parquet
  index.sqlite                  # wb_id -> campaign, policy, functions, status (queryable)
```

`manifest.json`:
```json
{
  "wb_id": "wb-7Q3K…", "ir_sha256": "…", "xlsx_sha256": "…",
  "campaign_id": "c-2026-10-01-a", "workbook_seed": "0x…",
  "generator": {"version": "0.3.1", "git": "abc123", "catalog_hash": "…", "writer": "xlsxwriter-3.2.x"},
  "milestone": "M2", "policy": "coercion", "swarm_mask_hash": "…", "scenario": null,
  "stats": {"sheets": 2, "cells": 1840, "probes": 1200, "max_depth": 7, "mr_instances": 85},
  "functions": {"SUM": 210, "VLOOKUP": 33},
  "edge_classes": {"fp_cancellation": 41, "blank_in_range": 97},
  "oracle_status": {"excel-16.0.19127.20000": "done"}
}
```

**Oracle JSON** (`oracle/<build>.json`):
```json
{
  "engine": "excel-desktop", "build": "16.0.19127.20000", "channel": "MonthlyEnterprise",
  "locale": "en-US", "calc": "FullRebuild", "trusted": true,
  "canaries": {"Calc!Z1": {"expect": 3333, "got": 3333}, "now": [46296.51, 46296.52]},
  "cells": {
    "Calc!B5": {"t": "n", "v": "0x4004b851eb851eb8", "d": "2.59", "formula2": "=SUM(Data!$C$2:$C$101)*1.1"},
    "Calc!B6": {"t": "e", "v": "#VALUE!"},
    "Calc!B7": {"t": "s", "v": "abc"},
    "Calc!B8": {"t": "b", "v": true},
    "Calc!D2": {"t": "spill", "rows": 3, "cols": 2, "values": [[…]]}
  }
}
```
Numbers are stored as the hex of the IEEE bits (lossless) plus a decimal for humans. The
RustSheet results use the same schema, so the differ is engine-agnostic.

---

## 13. Coverage measurement

### 13.1 The matrix
Dimensions: `function × arg_position × arg_type_class × edge_class` plus a
`result_class` axis (number/text/bool/each error/blank/spill). Cells hold counts of probes
that **evaluated** that combination, from AST annotations captured at generation time.
Runtime classes are confirmed from RustSheet's evaluation trace, since a cell ref may have
held text at runtime. Derived views:
- **Function × arg-type** heatmap: are `SUM` probes getting text, bool, blank and error
  inputs in both direct and in-range positions?
- **Function × edge-class** completion (% of catalog-declared edge classes hit ≥ N times,
  N=20 by default).
- **Function × result-class**: did we ever see each documented error for this function?
  (The catalog can list `expected_errors: ["#NUM!","#VALUE!"]`.)
- **Pairwise interaction coverage** for (outer function, inner function) at depth 2, to
  find untested compositions. Weighted by priors so we don't chase combinatorial dust.
- **Structural coverage**: chain depth buckets, cross-sheet hops, ref kinds (union,
  intersection, 3D, spill-ref, full-column), fill patterns, and dynamic dependencies.

### 13.2 Engine code coverage (optional Zest-style feedback)
Instrument RustSheet with `cargo llvm-cov` (or sancov) in the MR loop. Workbooks that hit
new branches in `functions/**` get their choice tapes kept as **seeds** for tape mutation
(coverage-guided structured fuzzing) [6]. This complements the spec-level matrix: the
spec says what Excel can do, and code coverage says what our implementation branches on.

### 13.3 Feedback
`gap(p)` (§5.3) combines bandit novelty with matrix under-coverage. The dashboard shows,
per milestone, the % of matrix cells at ≥N with 0 open diffs, which serves as an "Excel
parity score" per function.

---

## 14. Excel semantic edge cases to target (checklist → edge classes)

Each item is an edge class with a dedicated generator. Items marked *(probe)* are
behaviors whose exact rule must be **learned from the oracle in phase 1**, before MRs or
expectations depend on them.

1. **15-significant-digit behavior** [20]: inputs are truncated to 15 significant digits
   (`0.000123456789012345 + 1` → `1.00012345678901`); xlsx `<v>` stores up to 17 digits.
   Test the number parser for literals with more than 15 digits *(probe: are formula
   literals rounded to 15 digits at parse time?)*.
2. **Close-to-zero compensation** on `+`/`-` (since Excel 97) [20][21]: final-operation
   and parentheses dependence; whether SUM applies it *(probe)*; whether comparisons
   `=0.1+0.2=0.3` use a tolerance *(probe)*.
3. **No −0, no NaN/Inf, no denormals**: overflow → `#NUM!`, `x/0` → `#DIV/0!`, underflow
   → 0 [20].
4. **Date serials and the 1900 bug**: serial 60 = 1900-02-29. `WEEKDAY`, `TEXT` and
   `DATEVALUE("1900-02-29")` are wrong-but-required for serials < 61. Serial 0 =
   "1900-01-00". `DATE(0..1899,…)` adds 1900. `DATE` normalizes month and day overflow
   (`DATE(2024,14,-1)`). Max serial 2958465. The 1904 date system is a workbook setting
   (generated rarely).
5. **Text↔number coercion**: `"1"+1=2`, `" 1 "+1=2`, `"1,000"+0`, `"$1"`, `"50%"`,
   `"(1)"`, date and time strings → serial; `"TRUE"+0` → `#VALUE!`; `TRUE+0=1`; `=""+0` →
   `#VALUE!`; `--` vs `VALUE` vs `N()` vs `T()` differ.
6. **Aggregation coercion asymmetry**: direct args coerced, range contents skipped (SUM,
   AVERAGE, MIN, MAX, COUNT); `*A` variants differ; `COUNT("1")=1` but text "1" in a range
   isn't counted.
7. **Empty-cell semantics**: `=A1` (blank) → 0. `=A1&""` → `""`. `ISBLANK` vs `=""` vs
   `LEN()=0`. Formula-returned `""` is not blank. A blank as a lookup value matches what?
   *(probe)*. `IF(TRUE,)` (missing arg) → 0.
8. **Comparison ordering**: numbers < text < FALSE < TRUE in sort and comparison;
   case-insensitive `=`; culture-aware `<`; errors propagate through comparisons.
9. **Error propagation order**: the first error in evaluation order wins
   (`=#N/A+#DIV/0!`). Functions that *consume* errors: IFERROR, IFNA, ISERROR, ISERR,
   ERROR.TYPE, AGGREGATE options, COUNT ignores errors in ranges.
10. **Implicit intersection** (legacy formulas and `@`): same row or column, else
    `#VALUE!`. Applies to functions expecting a scalar when given a range in non-DA
    context [23].
11. **Array broadcasting and lifting**: scalar⊗array, row⊗column → matrix, mismatched
    shapes → `#N/A` fill, functions with `ref_only` params not lifting, `SUMPRODUCT`
    treating non-numeric as 0.
12. **Spills**: `#SPILL!` on collision, merged cell or table; `A1#` refs; spill of a
    spill; `@` on a spilling function.
13. **Criteria mini-language**: operators in strings, wildcards and `~` escape, numeric
    text matching numbers (`COUNTIF(r,"1")` matches number 1 *and* text "1"),
    `"<>"` = non-blank, `"="` = blank, dates as criteria strings.
14. **Lookup approximate-mode semantics on unsorted data**: Excel's binary-search
    behavior must be bit-for-bit identical. Duplicates, mixed types and blanks in the
    lookup column.
15. **Rounding functions**: decimal-correct ROUND (2.675, 1.005), `MROUND` sign rules,
    `CEILING`/`FLOOR` legacy vs `.MATH`/`.PRECISE`, `INT` vs `TRUNC` on negatives, `MOD`
    with a negative divisor and large quotients (`MOD(1e15,3)`, `MOD(2^53,…)` → `#NUM!`
    limits).
16. **Power & roots**: `0^0` → `#NUM!`, `(-8)^(1/3)` → `#NUM!`, `-2^2` = 4 (unary minus
    binds tighter than `^`), `2^-1074`.
17. **Text functions on UTF-16**: LEN counts UTF-16 code units (an emoji counts 2);
    MID/LEFT can split surrogate pairs; `CHAR`/`CODE` code-page vs `UNICHAR`/`UNICODE`.
18. **Locale**: pin en-US for the oracle. `TEXT` formats, decimal separators, and
    `DATEVALUE` parsing are locale-dependent and stay excluded from other locales until a
    later milestone.

---

## 15. Implementation plan

### 15.1 Language and library choice

| Concern | Python (+XlsxWriter / openpyxl) | Rust (+rust_xlsxwriter) |
|---|---|---|
| Iteration speed on generator logic, strategies, templates | **Best**. Hypothesis-like code, notebooks for analysis | slower to iterate |
| Ecosystem for analysis (coverage matrix, bandit, dashboards) | **pandas/polars, numpy** | adequate |
| xlsx writing correctness for modern Excel | **XlsxWriter**: auto `_xlfn.` via `use_future_functions`, `_xlws.` for FILTER/SORT, `_xlpm.` for LAMBDA params, `write_dynamic_array_formula` emits the DA metadata (no `@` insertion) [24] | **rust_xlsxwriter**: same author and design; auto-prefixes future functions; `write_dynamic_formula` [25] |
| openpyxl | Reads and writes. **No cached values** written (fine, since Excel computes, and it guarantees no stale `<v>` can fool a reader, as [13] notes). **But** it doesn't auto-add `_xlfn.` prefixes and has no first-class dynamic-array (`cm`/metadata.xml) support, so DA formulas get `@`-ified or `#NAME?` in Excel unless hand-patched. Good as a **reader** of oracle-saved files and for edge-case writer tests. | — |
| Speed for the MR loop (millions of evaluations) | too slow if it round-trips xlsx | **Best**: in-process RustSheet calls on the IR, no xlsx |
| Integration with the RustSheet test suite | via files | native (`cargo test`, proptest, cargo-fuzz) |

**Recommendation (hybrid):**
- **Generator (Python)**: catalog loader, typed generator, value pools, templates, bandit,
  coverage analytics. Output is canonical IR JSON.
- **xlsx writer (Python, XlsxWriter)**, isolated behind a `Writer` interface. Keep an
  openpyxl writer only for "writer diversity" tests of RustSheet's reader (shared
  formulas, inline strings, odd XML) and use openpyxl to read oracle files. Excel computes
  everything, so nothing needs cached values. Set `fullCalcOnLoad`.
- **MR runner and reducer core (Rust)**: crate `rustsheet-fuzz` in the workspace. It
  loads the IR directly into the engine, runs MR-INCR, MR-ORDER and the structural MRs,
  and can use `proptest`/`cargo-fuzz` for the choice-tape-driven variants. It writes
  violations as IR and reuses the same corpus layout.
- **Oracle farm (Python)**: a Windows VM pool with Excel desktop (xlwings/pywin32 COM)
  as primary, and Graph/Excel-web as an optional secondary. A queue of `wb_id`s;
  idempotent per `(wb_id, build)`.
- **Port to Rust later?** Only the hot path (generation for the MR loop) if Python becomes
  a bottleneck. The IR boundary makes this a contained change. Optionally generate a Rust
  version of the catalog via `build.rs` from the same YAML.

### 15.2 Phased build order

| Phase | Deliverable | Exit criterion |
|---|---|---|
| **P0: Foundations** (1–2 wk) | Catalog schema + JSON Schema validation; bootstrap scrape of names/versions [19]; hand-curate the top ~60 (Enron/EUSES top 15 ∪ MS top 10 ∪ M1/M2 scope); IR spec + canonicalizer + hasher; XlsxWriter writer; oracle harness with canaries; oracle JSON; RustSheet result dumper in the same schema; differ | A hand-written IR round-trips through Excel and RustSheet and diffs |
| **P1: Edge-behavior probes** (1 wk) | Hand-enumerated probe grids (not random) for every §14 item marked *(probe)*, run once through the oracle | Rules for snapping, comparison tolerance, literal parsing and blank lookup documented as catalog notes and expected-difference MRs |
| **P2: Typed scalar generator** | `gen_expr` with budgets, value pools, priors, milestone mask, probe packing, sidecars; coverage matrix v1 | ≥ 1k probes/workbook; matrix shows ≥ 80% of declared edge classes hit for M1 functions |
| **P3: MR engine (Rust)** | `rustsheet-fuzz` crate: algebraic MRs, TLP, NoREC pairs, MR-INCR, MR-ORDER, MR-ROUNDTRIP; MR certification run against the oracle | Runs nightly in CI with no oracle; zero uncertified MRs filing tasks |
| **P4: Reduction + dedup + filing** | Sidecar localization, cone-of-influence, batched parallel ddmin, AST HDD + evaluation substitution, literal shrinking, signatures, task template | Median repro ≤ 5 cells; ≤ 6 oracle calls per bug |
| **P5: Workbook structure + scenarios** | Data-table archetypes, mixed ranges, DAG generator, fill-down templates, cross-sheet, names, 7 scenario templates | Scenario suite becomes a CI gate (exact match on pinned oracle results) |
| **P6: Adaptivity** | Thompson-sampling bandit, decay and damping, swarm masks, generation policies, dashboards; optional llvm-cov feedback and tape mutation | New-signature discovery rate per oracle-hour beats a uniform-random baseline (A/B over 2 weeks) |
| **P7: Modern Excel** | Dynamic arrays and spills, LET/LAMBDA/MAP/REDUCE, structured refs, TEXTSPLIT/REGEX*, GROUPBY/PIVOTBY (365) | Matrix coverage for the M-dyn milestone |

### 15.3 Risks
- **Oracle throughput** is the bottleneck. Mitigations: dense packing, sidecars, MR
  certification, Excel-web pre-screen, and caching by content hash.
- **Excel version drift**: pin the build, key results by build, and re-baseline the
  scenario suite on upgrade, diffing Excel-vs-Excel across builds (which also finds
  Excel's own changes).
- **Generator bias** toward what we already know: swarm, policies, tempered priors, and
  code-coverage feedback counteract it.
- **Subexpression-cell semantics change** (implicit intersection, ref-valued nodes):
  handled through catalog `mode`, plus final oracle confirmation of every repro.

---

## 16. Sources

1. Yang, Chen, Eide, Regehr. *Finding and Understanding Bugs in C Compilers* (Csmith), PLDI 2011. https://www.researchgate.net/publication/220752409_Finding_and_Understanding_Bugs_in_C_Compilers
2. Livinskii, Babokin, Regehr. *Random Testing for C and C++ Compilers with YARPGen*, OOPSLA 2020. https://users.cs.utah.edu/~regehr/yarpgen-oopsla20.pdf · https://blog.sigplan.org/2021/01/14/finding-bugs-in-c-and-c-compilers-using-yarpgen/
3. Groce, Zhang, Eide, Chen, Regehr. *Swarm Testing*, ISSTA 2012. https://www.flux.utah.edu/paper/groce-issta12
4. Rigger, Su. *Testing Database Engines via Pivoted Query Synthesis*, OSDI 2020. https://www.usenix.org/system/files/osdi20-rigger.pdf
5. Rigger, Su. *Finding Bugs in Database Systems via Query Partitioning* (TLP), OOPSLA 2020. https://www.manuelrigger.at/preprints/TLP.pdf · SQLancer papers list: https://github.com/sqlancer/sqlancer/blob/main/docs/PAPERS.md
6. Padhye et al. *Semantic Fuzzing with Zest*, ISSTA 2019. https://arxiv.org/pdf/1812.00078 · JQF: https://github.com/rohanpadhye/JQF
7. Yue et al. *EcoFuzz: Adaptive Energy-Saving Greybox Fuzzing as a Variant of the Adversarial Multi-Armed Bandit*, USENIX Security 2020. https://www.usenix.org/conference/usenixsecurity20/presentation/yue
8. Karamcheti, Mann, Rosenberg. *Adaptive Grey-Box Fuzz-Testing with Thompson Sampling*, 2018. https://arxiv.org/pdf/1808.08256 · *Make out like a (Multi-Armed) Bandit* (T-Scheduler). https://arxiv.org/pdf/2312.04749
9. Zeller & Hildebrandt, *Simplifying and Isolating Failure-Inducing Input*, TSE 2002. Misherghi & Su, *HDD*, ICSE 2006. Regehr et al., *Test-Case Reduction for C Compiler Bugs* (C-Reduce), PLDI 2012. Sun et al., *Perses*, ICSE 2018. https://dl.acm.org/doi/10.1145/3180155.3180236 · HDD with hoisting: https://arxiv.org/pdf/2104.03637
10. MacIver & Donaldson. *Test-Case Reduction via Test-Case Generation: Insights from the Hypothesis Reducer*, ECOOP 2020. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2020.13
11. Jiang, Tian, Chen. *Delta Debugging in the Absence of Test Oracles Through Metamorphic Testing* (DDMT), 2026. https://arxiv.org/abs/2607.00929
12. *Kale: A Transformation-Safe Spreadsheet System*, 2026. https://arxiv.org/abs/2608.26345
13. spreadsheet-compat / canispreadsheet.com (cross-engine executed function database, canaries, trusted flag). https://github.com/xxaflabsxx/spreadsheet-compat
14. Hermans & Murphy-Hill. *Enron's Spreadsheets and Related Emails: A Dataset and Analysis*, ICSE 2015. https://repository.tudelft.nl/record/uuid:085374fa-06a3-48c9-a022-aa02c32507e8 · https://www.researchgate.net/publication/304552688_Enron's_Spreadsheets_and_Related_Emails_A_Dataset_and_Analysis
15. Jansen. *Enron versus EUSES: A Comparison of Two Spreadsheet Corpora*, SEMS 2015. https://arxiv.org/html/1503.04055
16. Barik, Lubick, Smith, Slankas, Murphy-Hill. *FUSE: A Reproducible, Extendable, Internet-scale Corpus of Spreadsheets*, MSR 2015. https://static.barik.net/barik/publications/msr2015/PID3640389.pdf
17. Jansen & Hermans. *The use of Charts, Pivot Tables, and Array Formulas in two Popular Spreadsheet Corpora*, 2018. https://www.alphaxiv.org/abs/1808.10642
18. Microsoft top-10 functions (via Microsoft Tech Community discussion). https://techcommunity.microsoft.com/discussions/excelgeneral/how-useful-are-the-top-10-most-popular-excel-functions/4145750
19. Microsoft. *Excel functions (alphabetical)* (version tags). https://support.microsoft.com/en-us/office/excel-functions-alphabetical-b3944572-255d-4efb-bb96-c6d90033e188 · by category: https://support.microsoft.com/en-us/excel/excel-functions-by-category
20. Microsoft Learn. *Floating-point arithmetic may give inaccurate result in Excel*. https://learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result
21. Wikipedia. *Numeric precision in Microsoft Excel* (parentheses defeat near-zero rounding, e.g. `=(1+2^-52-1)`). https://en.wikipedia.org/wiki/Numeric_precision_in_Microsoft_Excel
22. Microsoft Graph. *workbookApplication: calculate* (Recalculate / Full / FullRebuild). https://learn.microsoft.com/en-us/graph/api/workbookapplication-calculate?view=graph-rest-1.0
23. Microsoft. *Implicit intersection operator: @*. https://support.microsoft.com/en-us/excel/implicit-intersection-operator · *Dynamic array formulas in non-dynamic aware Excel*: https://support.microsoft.com/en-us/office/dynamic-array-formulas-in-non-dynamic-aware-excel-696e164e-306b-4282-ae9d-aa88f5502fa2
24. XlsxWriter docs. *Working with Formulas* (future functions `_xlfn.`, `_xlws.`, `_xlpm.`, dynamic arrays, cached results). https://xlsxwriter.readthedocs.io/working_with_formulas.html
25. rust_xlsxwriter docs. `Formula`. https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/struct.Formula.html
26. Microsoft. *Calculation operators and precedence in Excel*. https://support.microsoft.com/en-us/office/calculation-operators-and-precedence-36de9366-46fe-43a3-bfa8-cf6d8068eacc
27. Excel 1900 leap-year bug (serial 60, WEEKDAY before 1900-03-01). https://en.wikipedia.org/wiki/Leap_year_problem · https://www.sethserver.com/programming/the-insane-history-of-dates-in-excel.html
28. openpyxl and dynamic arrays (manual `cm`/metadata needed). https://groups.google.com/g/openpyxl-users/c/aacD5eRiP7w
29. IronCalc test corpus (Excel-saved workbooks per category; engine recompute vs cached values): `IronCalc/xlsx/tests/calc_tests/**` (local checkout in scratchpad).
